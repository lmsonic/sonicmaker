use godot::prelude::*;

use crate::{
    character::{Character, State},
    sensor::DetectionResult,
};

use super::utils::Mode;

impl Character {
    pub(super) fn grounded_right_wall_collision(&mut self, distance: f32) {
        godot_print!("Right wall collision");
        let mut velocity = self.velocity();
        let right = self.current_mode().right();
        velocity += right * distance;
        self.ground_speed = 0.0;
        self.set_state(State::Idle);
        self.set_velocity(velocity);
    }
    pub(super) fn grounded_left_wall_collision(&mut self, distance: f32) {
        godot_print!("Left wall collision");

        let mut velocity = self.velocity();
        let left = self.current_mode().left();
        velocity += left * distance;
        self.ground_speed = 0.0;
        self.set_velocity(velocity);
    }
    pub(super) fn airborne_left_wall_collision(&mut self, distance: f32) {
        godot_print!("Left wall collision");
        let mut position = self.global_position();
        position.x -= distance;
        self.set_global_position(position);

        let velocity = self.velocity();
        self.set_velocity(Vector2::new(0.0, velocity.y));
    }
    pub(super) fn airborne_right_wall_collision(&mut self, distance: f32) {
        godot_print!("Right wall collision");
        let mut position = self.global_position();
        position.x += distance;
        self.set_global_position(position);

        let velocity = self.velocity();
        self.set_velocity(Vector2::new(0.0, velocity.y));
    }

    pub(super) fn snap_to_floor(&mut self, distance: f32) {
        let mode = self.current_mode();
        let mut position = self.global_position();
        match mode {
            Mode::Floor => position.y += distance,
            Mode::RightWall => position.x += distance,
            Mode::Ceiling => position.y -= distance,
            Mode::LeftWall => position.x += distance,
        }

        self.set_global_position(position);
    }

    pub(super) fn ground_sensor_results(&mut self) -> Vec<DetectionResult> {
        let mut results = vec![];

        if let Some(sensor_floor_left) = &mut self.sensor_floor_left {
            if let Ok(r) = sensor_floor_left
                .bind_mut()
                .detect_solid()
                .try_to::<DetectionResult>()
            {
                results.push(r);
            }
        };
        if let Some(sensor_floor_right) = &mut self.sensor_floor_right {
            if let Ok(r) = sensor_floor_right
                .bind_mut()
                .detect_solid()
                .try_to::<DetectionResult>()
            {
                results.push(r);
            }
        }
        results
    }
    pub(super) fn can_jump(&mut self) -> bool {
        if let Some(result) = self.ceiling_check() {
            return result.distance >= 6.0;
        }
        true
    }
    pub(super) fn ground_check(&mut self) -> Option<DetectionResult> {
        self.ground_sensor_results()
            .into_iter()
            .min_by(|a, b| a.distance.total_cmp(&b.distance))
    }
    pub(super) fn ceiling_check(&mut self) -> Option<DetectionResult> {
        self.ceiling_sensor_results()
            .into_iter()
            .min_by(|a, b| a.distance.total_cmp(&b.distance))
    }

    fn ceiling_sensor_results(&mut self) -> Vec<DetectionResult> {
        let mut results = vec![];
        if let Some(sensor_ceiling_left) = &mut self.sensor_ceiling_left {
            if let Ok(r) = sensor_ceiling_left
                .bind_mut()
                .detect_solid()
                .try_to::<DetectionResult>()
            {
                results.push(r);
            }
        };
        if let Some(sensor_ceiling_right) = &mut self.sensor_ceiling_right {
            if let Ok(r) = sensor_ceiling_right
                .bind_mut()
                .detect_solid()
                .try_to::<DetectionResult>()
            {
                results.push(r);
            }
        };
        results
    }

    pub(super) fn wall_left_sensor_check(&mut self) -> Option<DetectionResult> {
        let velocity = self.velocity();

        if let Some(sensor_push_left) = &mut self.sensor_push_left {
            let old_position = sensor_push_left.get_global_position();
            let new_position = old_position + velocity;
            sensor_push_left.set_global_position(new_position);
            let result = if let Ok(result) = sensor_push_left
                .bind_mut()
                .detect_solid()
                .try_to::<DetectionResult>()
            {
                Some(result)
            } else {
                None
            };
            sensor_push_left.set_global_position(old_position);
            result
        } else {
            None
        }
    }
    pub(super) fn wall_right_sensor_check(&mut self) -> Option<DetectionResult> {
        let velocity = self.velocity();
        if let Some(sensor_push_right) = &mut self.sensor_push_right {
            let old_position = sensor_push_right.get_global_position();
            let new_position = old_position + velocity;
            sensor_push_right.set_global_position(new_position);
            let result = if let Ok(result) = sensor_push_right
                .bind_mut()
                .detect_solid()
                .try_to::<DetectionResult>()
            {
                Some(result)
            } else {
                None
            };
            sensor_push_right.set_global_position(old_position);
            result
        } else {
            None
        }
    }
    pub(super) fn airborne_wall_left_sensor_check(&mut self) -> Option<DetectionResult> {
        if let Some(sensor_push_left) = &mut self.sensor_push_left {
            if let Ok(result) = sensor_push_left
                .bind_mut()
                .detect_solid()
                .try_to::<DetectionResult>()
            {
                return Some(result);
            }
        }
        None
    }
    pub(super) fn airborne_wall_right_sensor_check(&mut self) -> Option<DetectionResult> {
        if let Some(sensor_push_right) = &mut self.sensor_push_right {
            if let Ok(result) = sensor_push_right
                .bind_mut()
                .detect_solid()
                .try_to::<DetectionResult>()
            {
                return Some(result);
            }
        }
        None
    }
}
