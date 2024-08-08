use godot::prelude::*;

use crate::{character::Character, sensor::DetectionResult};

impl Character {
    pub(super) fn grounded_right_wall_collision(&mut self, distance: f32) {
        let right = self.current_mode().right();
        self.velocity += right * distance;
        self.ground_speed = 0.0;
        godot_print!("Grounded right wall collision, dv:{} ", right * distance);
    }
    pub(super) fn grounded_left_wall_collision(&mut self, distance: f32) {
        let left = self.current_mode().left();
        self.velocity += left * distance;
        self.ground_speed = 0.0;

        godot_print!("Grounded left wall collision, dv:{} ", left * distance);
    }
    pub(super) fn airborne_left_wall_collision(&mut self, distance: f32) {
        let mut position = self.global_position();
        position.x -= distance;
        self.set_global_position(position);

        self.velocity.x = 0.0;

        godot_print!("Airborne left wall collision dx:{}", -distance);
    }
    pub(super) fn airborne_right_wall_collision(&mut self, distance: f32) {
        let mut position = self.global_position();
        godot_print!("{}", position);
        position.x += distance;
        self.set_global_position(position);

        self.velocity.x = 0.0;

        godot_print!("Airborne right wall collision dx:{}", distance);
    }

    pub(super) fn snap_to_floor(&mut self, distance: f32) {
        let mut position = self.global_position();
        let down = self.current_mode().down();
        position += down * distance;

        self.set_global_position(position);
        godot_print!("Snap to floor dp: {}", down * distance);
    }

    pub(super) fn ground_sensor_results(&mut self) -> Vec<DetectionResult> {
        let mut results = vec![];

        if let Some(sensor_floor_left) = &mut self.sensor_floor_left {
            if let Ok(r) = sensor_floor_left
                .bind_mut()
                .sense()
                .try_to::<DetectionResult>()
            {
                results.push(r);
            }
        };
        if let Some(sensor_floor_right) = &mut self.sensor_floor_right {
            if let Ok(r) = sensor_floor_right
                .bind_mut()
                .sense()
                .try_to::<DetectionResult>()
            {
                results.push(r);
            }
        }
        results
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
                .sense()
                .try_to::<DetectionResult>()
            {
                results.push(r);
            }
        };
        if let Some(sensor_ceiling_right) = &mut self.sensor_ceiling_right {
            if let Ok(r) = sensor_ceiling_right
                .bind_mut()
                .sense()
                .try_to::<DetectionResult>()
            {
                results.push(r);
            }
        };
        results
    }

    pub(super) fn wall_left_sensor_check(&mut self) -> Option<DetectionResult> {
        if let Some(sensor_push_left) = &mut self.sensor_push_left {
            let old_position = sensor_push_left.get_global_position();
            let new_position = old_position + self.velocity;
            sensor_push_left.set_global_position(new_position);
            let result = if let Ok(result) = sensor_push_left
                .bind_mut()
                .sense()
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
        if let Some(sensor_push_right) = &mut self.sensor_push_right {
            let old_position = sensor_push_right.get_global_position();
            let new_position = old_position + self.velocity;
            sensor_push_right.set_global_position(new_position);
            let result = if let Ok(result) = sensor_push_right
                .bind_mut()
                .sense()
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
                .sense()
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
                .sense()
                .try_to::<DetectionResult>()
            {
                return Some(result);
            }
        }
        None
    }
}
