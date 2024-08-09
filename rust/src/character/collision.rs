use godot::prelude::*;

use crate::{
    character::Character,
    sensor::{DetectionResult, Sensor},
};

impl Character {
    pub(super) fn grounded_right_wall_collision(&mut self, distance: f32) {
        let right = self.current_mode().right();
        let mut position = self.global_position();
        position += right * distance;
        self.ground_speed = 0.0;
        self.set_global_position(position);
        godot_print!("Grounded right wall collision, d:{} ", right * distance);
    }
    pub(super) fn grounded_left_wall_collision(&mut self, distance: f32) {
        let left = self.current_mode().left();
        let mut position = self.global_position();
        position += left * (distance);
        self.ground_speed = 0.0;
        self.set_global_position(position);

        godot_print!("Grounded left wall collision, dp:{} ", left * distance);
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

    fn sensor_results(
        &mut self,
        sensors: &mut [Option<Gd<Sensor>>],
        apply_velocity: bool,
    ) -> Vec<DetectionResult> {
        let mut results = vec![];
        for sensor in sensors.iter_mut().flatten() {
            let position = sensor.get_position();
            if apply_velocity {
                let next_position = position + self.velocity;
                sensor.set_position(next_position);
            }
            if let Ok(r) = sensor.bind_mut().sense().try_to::<DetectionResult>() {
                results.push(r);
            }
            if apply_velocity {
                sensor.set_position(position);
            }
        }
        results
    }
    fn sensor_result(
        &mut self,
        sensor: &mut Option<Gd<Sensor>>,
        apply_velocity: bool,
    ) -> Option<DetectionResult> {
        let mut result = None;

        if self.velocity.length() > 15.0 {};
        if let Some(sensor) = sensor {
            let position = sensor.get_position();
            if apply_velocity {
                let next_position = position + self.velocity;
                sensor.set_position(next_position);
            }
            if let Ok(r) = sensor.bind_mut().sense().try_to::<DetectionResult>() {
                result = Some(r);
            }
            if apply_velocity {
                sensor.set_position(position);
            }
        }
        result
    }

    pub(super) fn snap_to_floor(&mut self, distance: f32) {
        let mut position = self.global_position();
        let down = self.current_mode().down();
        position += down * distance;

        self.set_global_position(position);
        godot_print!("Snap to floor dp: {}", down * distance);
    }

    pub(super) fn ground_sensor_results(&mut self, apply_velocity: bool) -> Vec<DetectionResult> {
        let left = self.sensor_floor_right.clone();
        let right = self.sensor_floor_left.clone();
        self.sensor_results(&mut [left, right], apply_velocity)
    }

    pub(super) fn ground_check(&mut self, apply_velocity: bool) -> Option<DetectionResult> {
        self.ground_sensor_results(apply_velocity)
            .into_iter()
            .min_by(|a, b| a.distance.total_cmp(&b.distance))
    }
    pub(super) fn ceiling_check(&mut self, apply_velocity: bool) -> Option<DetectionResult> {
        self.ceiling_sensor_results(apply_velocity)
            .into_iter()
            .min_by(|a, b| a.distance.total_cmp(&b.distance))
    }

    fn ceiling_sensor_results(&mut self, apply_velocity: bool) -> Vec<DetectionResult> {
        let left = self.sensor_ceiling_right.clone();
        let right = self.sensor_ceiling_left.clone();
        self.sensor_results(&mut [left, right], apply_velocity)
    }

    pub(super) fn wall_left_sensor_check(
        &mut self,
        apply_velocity: bool,
    ) -> Option<DetectionResult> {
        self.sensor_result(&mut self.sensor_push_left.clone(), apply_velocity)
    }
    pub(super) fn wall_right_sensor_check(
        &mut self,
        apply_velocity: bool,
    ) -> Option<DetectionResult> {
        self.sensor_result(&mut self.sensor_push_right.clone(), apply_velocity)
    }
}
