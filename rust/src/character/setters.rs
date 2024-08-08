use std::{f32::consts::FRAC_PI_2, ops::Rem};

use godot::{engine::RectangleShape2D, prelude::*};
use real_consts::{PI, TAU};

#[derive(GodotConvert, Var, Export, Default, Debug, PartialEq, Eq, Clone, Copy)]
#[godot(via = GString)]
pub(super) enum Kind {
    #[default]
    Sonic,
    Tails,
    Knuckles,
}
#[derive(GodotConvert, Var, Export, Default, Debug, PartialEq, Eq, Clone, Copy)]
#[godot(via = GString)]
pub(super) enum State {
    #[default]
    Idle,
    StartMotion,
    FullMotion,
    JumpBall,
    RollingBall,
}

impl State {
    pub(super) fn is_ball(&self) -> bool {
        *self == Self::JumpBall || *self == Self::RollingBall
    }

    /// Returns `true` if the state is [`JumpBall`].
    ///
    /// [`JumpBall`]: State::JumpBall
    #[must_use]
    pub(super) fn is_jumping(&self) -> bool {
        matches!(self, Self::JumpBall)
    }

    /// Returns `true` if the state is [`RollingBall`].
    ///
    /// [`RollingBall`]: State::RollingBall
    #[must_use]
    pub(super) fn is_rolling(&self) -> bool {
        matches!(self, Self::RollingBall)
    }
}
use crate::{character::Character, sensor::DetectionResult};
#[godot_api]
impl Character {
    #[func]
    pub(super) fn set_grounded(&mut self, value: bool) {
        self.is_grounded = value;
        self.update_sensors();
    }
    #[func]
    pub(super) fn set_ground_angle(&mut self, result: DetectionResult) {
        let mut angle = if result.snap {
            (result.angle / FRAC_PI_2).round().rem(4.0) * FRAC_PI_2
        } else {
            result.angle
        };
        self.ground_angle = angle;
        if angle < PI {
            angle += TAU;
        }
        self.base_mut().set_rotation(TAU - angle);
        self.update_sensors();
    }
    #[func]
    pub(super) fn set_width_radius(&mut self, value: f32) {
        self.width_radius = value;
        self.update_sensors();
    }

    #[func]
    pub(super) fn set_height_radius(&mut self, value: f32) {
        let delta = self.height_radius - value;
        self.height_radius = value;
        self.update_sensors();
        self.update_y_position(delta);
    }
    #[func]
    pub(super) fn set_push_radius(&mut self, value: f32) {
        self.push_radius = value;
        self.update_sensors();
    }
    fn update_y_position(&mut self, delta: f32) {
        let mut position = self.global_position();
        let down = self.current_mode().down();
        position += down * delta;
        self.set_global_position(position);
    }

    #[func]
    pub(super) fn set_character(&mut self, value: Kind) {
        match value {
            Kind::Sonic => {
                self.set_width_radius(9.0);
                self.set_height_radius(19.0);
                self.jump_force = 6.5;
            }
            Kind::Tails => {
                self.set_width_radius(9.0);
                self.set_height_radius(15.0);
                self.jump_force = 6.5;
            }
            Kind::Knuckles => {
                self.set_width_radius(9.0);
                self.set_height_radius(19.0);
                self.jump_force = 6.0;
            }
        }

        self.character = value;
    }
    #[func]
    pub(super) fn set_state(&mut self, value: State) {
        let was_ball = self.state.is_ball();
        let is_ball = value.is_ball();

        if was_ball && !is_ball {
            self.set_character(self.character);
        } else if is_ball && !was_ball {
            self.set_width_radius(7.0);
            self.set_height_radius(14.0);
        }
        self.state = value;
        if let Some(sprites) = &mut self.sprites {
            match self.state {
                State::Idle => sprites.play_ex().name(c"idle".into()).done(),
                State::StartMotion => sprites.play_ex().name(c"start_motion".into()).done(),
                State::FullMotion => sprites.play_ex().name(c"full_motion".into()).done(),
                State::JumpBall | State::RollingBall => {
                    sprites.play_ex().name(c"rolling".into()).done()
                }
            }
        }
    }
    pub(super) fn set_flip_h(&mut self, value: bool) {
        if let Some(sprites) = &mut self.sprites {
            sprites.set_flip_h(value);
        }
    }

    #[func]
    pub fn update_sensors(&mut self) {
        let mask = self.get_collision_layer();
        {
            let half_width = self.width_radius;
            let half_height = self.height_radius;
            let mode = self.current_mode();

            // Floor and ceiling sensors
            let down_direction = mode.down_direction();
            let up_direction = mode.up_direction();

            let angle = mode.angle();

            let bottom_left = Vector2::new(-half_width, half_height).rotated(angle);
            let bottom_right = Vector2::new(half_width, half_height).rotated(angle);
            let top_left = Vector2::new(-half_width, -half_height).rotated(angle);
            let top_right = Vector2::new(half_width, -half_height).rotated(angle);

            if let Some(sensor_floor_left) = &mut self.sensor_floor_left {
                sensor_floor_left.set_position(bottom_left);
                sensor_floor_left.bind_mut().set_direction(down_direction);
                sensor_floor_left.bind_mut().set_collision_mask(mask);
            };
            if let Some(sensor_floor_right) = &mut self.sensor_floor_right {
                sensor_floor_right.set_position(bottom_right);
                sensor_floor_right.bind_mut().set_direction(down_direction);
                sensor_floor_right.bind_mut().set_collision_mask(mask);
            };
            if let Some(sensor_ceiling_left) = &mut self.sensor_ceiling_left {
                sensor_ceiling_left.set_position(top_left);
                sensor_ceiling_left.bind_mut().set_direction(up_direction);
                sensor_ceiling_left.bind_mut().set_collision_mask(mask);
            };
            if let Some(sensor_ceiling_right) = &mut self.sensor_ceiling_right {
                sensor_ceiling_right.set_position(top_right);
                sensor_ceiling_right.bind_mut().set_direction(up_direction);
                sensor_ceiling_right.bind_mut().set_collision_mask(mask);
            };
        }
        {
            // Push Sensors
            let half_width = self.push_radius;
            let mode = self.current_mode_walls();
            let half_height =
                if self.is_grounded && (self.ground_angle == 0.0 || self.ground_angle == TAU) {
                    8.0
                } else {
                    0.0
                };
            let right_direction = mode.right_direction();
            let left_direction = mode.left_direction();
            let angle = mode.angle();

            let center_left = Vector2::new(-half_width, half_height).rotated(angle);
            let center_right = Vector2::new(half_width, half_height).rotated(angle);
            if let Some(sensor_push_left) = &mut self.sensor_push_left {
                sensor_push_left.set_position(center_left);
                sensor_push_left.bind_mut().set_direction(left_direction);
                sensor_push_left.bind_mut().set_collision_mask(mask);
            };
            if let Some(sensor_push_right) = &mut self.sensor_push_right {
                sensor_push_right.set_position(center_right);
                sensor_push_right.bind_mut().set_direction(right_direction);
                sensor_push_right.bind_mut().set_collision_mask(mask);
            };
        }
        self.update_shapes();
    }
    pub(super) fn update_shapes(&mut self) {
        let width = self.width_radius * 2.0;
        let height = self.height_radius * 2.0;
        let mode = self.current_mode();

        if let Some(mut shape) = self
            .sensor_shape
            .as_deref_mut()
            .and_then(|cs| cs.get_shape())
            .and_then(|shape| shape.try_cast::<RectangleShape2D>().ok())
        {
            let mut size = Vector2::new(width, height);
            if mode.is_sideways() {
                size = Vector2::new(size.y, size.x);
            }
            shape.set_size(size);
        }
        if let Some(mut hitbox) = self
            .hitbox_shape
            .as_deref_mut()
            .and_then(|cs| cs.get_shape())
            .and_then(|shape| shape.try_cast::<RectangleShape2D>().ok())
        {
            let mut size = Vector2::new(15.0, height - 3.0);
            if mode.is_sideways() {
                size = Vector2::new(size.y, size.x);
            }
            hitbox.set_size(size);
        }
    }
}
