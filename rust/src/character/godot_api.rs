#![allow(clippy::needless_pass_by_value)]
use std::{f32::consts::FRAC_PI_2, ops::Rem};

use godot::{builtin::math::ApproxEq, engine::RectangleShape2D, prelude::*};
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
    Hurt,
}

impl State {
    pub(super) fn is_ball(self) -> bool {
        self == Self::JumpBall || self == Self::RollingBall
    }

    /// Returns `true` if the state is [`JumpBall`].
    ///
    /// [`JumpBall`]: State::JumpBall
    #[must_use]
    pub(super) const fn is_jumping(self) -> bool {
        matches!(self, Self::JumpBall)
    }

    /// Returns `true` if the state is [`RollingBall`].
    ///
    /// [`RollingBall`]: State::RollingBall
    #[must_use]
    pub(super) const fn is_rolling(self) -> bool {
        matches!(self, Self::RollingBall)
    }

    /// Returns `true` if the state is [`Hurt`].
    ///
    /// [`Hurt`]: State::Hurt
    #[must_use]
    pub(super) const fn is_hurt(self) -> bool {
        matches!(self, Self::Hurt)
    }
}
use crate::{
    character::Character,
    sensor::DetectionResult,
    solid_object::{sloped_solid_object::SlopedSolidObject, SolidObject},
};

pub enum SolidObjectKind {
    Simple(Gd<SolidObject>),
    Sloped(Gd<SlopedSolidObject>),
}

#[godot_api]
impl Character {
    #[signal]
    fn rings_changed(value: i32);
    #[func]
    pub(super) fn set_rings(&mut self, value: i32) {
        self.rings = value;
        self.base_mut()
            .emit_signal(c"rings_changed".into(), &[Variant::from(value)]);
    }
    #[func]
    pub fn set_stand_on_object(&mut self, object: Gd<SolidObject>) {
        self.solid_object_to_stand_on = Some(SolidObjectKind::Simple(object));
    }
    #[func]
    pub fn set_stand_on_sloped_object(&mut self, object: Gd<SlopedSolidObject>) {
        self.solid_object_to_stand_on = Some(SolidObjectKind::Sloped(object));
    }
    #[func]
    fn on_attacking(&mut self, badnik: Gd<Node2D>, is_boss: bool) {
        if self.is_grounded {
            return;
        }
        let position = self.global_position();
        let badnik_position = badnik.get_global_position();
        if is_boss {
            self.velocity *= -0.5;
        } else if position.y > badnik_position.y || self.velocity.y < 0.0 {
            // No rebound
            self.velocity.y -= self.velocity.y.signum();
        } else {
            // Rebound
            self.velocity.y *= -1.0;
        }
    }
    #[func]
    fn on_hurt(&mut self, hazard: Gd<Node2D>) {
        if self.is_invulnerable() {
            return;
        }
        if self.rings <= 0 {
            // Death
            self.die();
            return;
        }
        self.regather_rings_timer = 64;
        self.scatter_rings();
        let hazard_position = hazard.get_global_position();
        let sign = (self.global_position().x - hazard_position.x).signum();
        self.velocity = Vector2::new(self.hurt_x_force * sign, self.hurt_y_force);
        self.set_state(State::Hurt);
        self.set_grounded(false);
        self.clear_objects();
    }
    #[func]
    #[allow(clippy::missing_const_for_fn)]
    pub(super) fn is_invulnerable(&self) -> bool {
        self.invulnerability_timer > 0 || self.state.is_hurt()
    }
    #[func]
    #[allow(clippy::missing_const_for_fn)]
    pub(super) fn can_gather_rings(&self) -> bool {
        (!self.state.is_hurt() || self.invulnerability_timer < 64) && self.regather_rings_timer <= 0
    }

    fn scatter_rings(&mut self) {
        if let Some(scattered_ring_scene) = &self.scattered_ring_scene.clone() {
            let ring_starting_angle = f32::to_radians(101.25);
            let mut ring_angle = ring_starting_angle;
            let mut ring_flip = false;
            let mut ring_speed = 4.0;

            for i in 0..(self.rings).min(32) {
                // If we are halfway, start second "circle" of rings with lower speed
                if i == 16 {
                    ring_speed = 2.0;
                    ring_angle = ring_starting_angle;
                }

                let mut x_speed = ring_angle.cos() * ring_speed;
                let y_speed = -ring_angle.sin() * ring_speed;
                // Every ring created will moving be at the same angle as the other in the current pair,
                // but flipped the other side of the circle
                if ring_flip {
                    x_speed *= -1.0; // Reverse ring's X Speed
                    ring_angle += f32::to_radians(22.5); // We increment angle on every other ring which makes 2 even rings either side
                }
                // Toggle flip
                ring_flip = !ring_flip;
                // Create a scattered ring object at the Player's X and Y Position;
                let mut scattered_ring = scattered_ring_scene.instantiate_as::<Node2D>();
                scattered_ring.set_as_top_level(true);
                scattered_ring.set_global_position(self.global_position());
                scattered_ring.set(
                    c"velocity".into(),
                    Vector2::new(x_speed, y_speed).to_variant(),
                );
                self.base_mut().add_child(scattered_ring.upcast());
            }
        }
        self.set_rings(0);
    }

    pub fn die(&self) {
        if let Some(mut tree) = self.base().get_tree() {
            tree.call_deferred("reload_current_scene".into(), &[]);
        }
    }
    #[func]
    pub fn set_collision_layer(&mut self, value: u32) {
        self.collision_layer = value;
        if let Some(sensor_floor_left) = &mut self.sensor_floor_left {
            sensor_floor_left.bind_mut().set_collision_mask(value);
        };
        if let Some(sensor_floor_right) = &mut self.sensor_floor_right {
            sensor_floor_right.bind_mut().set_collision_mask(value);
        };
        if let Some(sensor_ceiling_left) = &mut self.sensor_ceiling_left {
            sensor_ceiling_left.bind_mut().set_collision_mask(value);
        };
        if let Some(sensor_ceiling_right) = &mut self.sensor_ceiling_right {
            sensor_ceiling_right.bind_mut().set_collision_mask(value);
        };
        if let Some(sensor_push_left) = &mut self.sensor_push_left {
            sensor_push_left.bind_mut().set_collision_mask(value);
        };
        if let Some(sensor_push_right) = &mut self.sensor_push_right {
            sensor_push_right.bind_mut().set_collision_mask(value);
        };
    }
    #[func]
    pub fn set_grounded(&mut self, value: bool) {
        self.is_grounded = value;
        self.update_sensors();
    }
    #[func]
    pub fn set_ground_angle(&mut self, mut angle: f32) {
        self.ground_angle = angle;
        if angle < PI {
            angle += TAU;
        }
        self.base_mut().set_rotation(TAU - angle);
        self.update_sensors();
    }

    pub fn set_ground_angle_from_result(&mut self, result: DetectionResult) {
        let angle = if result.snap {
            (result.angle / FRAC_PI_2).round().rem(4.0) * FRAC_PI_2
        } else {
            result.angle
        };
        self.set_ground_angle(angle);
    }
    #[func]
    pub(super) fn set_width_radius(&mut self, value: f32) {
        self.width_radius = value;
        self.update_sensors();
    }

    #[func]
    pub(super) fn set_height_radius(&mut self, value: f32) {
        self.height_radius = value;
        self.update_sensors();
    }
    #[func]
    pub(super) fn set_push_radius(&mut self, value: f32) {
        self.push_radius = value;
        self.update_sensors();
    }
    #[allow(dead_code)]
    fn update_y_position(&mut self, delta: f32) {
        let mut position = self.global_position();
        let down = self.current_mode().down();
        position += down * (delta - 2.0);
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
        if self.state.is_hurt() && !value.is_hurt() {
            self.invulnerability_timer = 120;
        }
        self.state = value;
        if was_ball && !is_ball {
            self.set_character(self.character);
        } else if is_ball && !was_ball {
            self.set_width_radius(7.0);
            self.set_height_radius(14.0);
        }

        if let Some(sprites) = &mut self.sprites {
            match self.state {
                State::Idle => sprites.play_ex().name(c"idle".into()).done(),
                State::StartMotion => sprites.play_ex().name(c"start_motion".into()).done(),
                State::FullMotion => sprites.play_ex().name(c"full_motion".into()).done(),
                State::JumpBall | State::RollingBall => {
                    sprites.play_ex().name(c"rolling".into()).done();
                } // TODO: add the hurt animation
                State::Hurt => {
                    sprites.play_ex().name(c"idle".into()).done();
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
            };
            if let Some(sensor_floor_right) = &mut self.sensor_floor_right {
                sensor_floor_right.set_position(bottom_right);
                sensor_floor_right.bind_mut().set_direction(down_direction);
            };
            if let Some(sensor_ceiling_left) = &mut self.sensor_ceiling_left {
                sensor_ceiling_left.set_position(top_left);
                sensor_ceiling_left.bind_mut().set_direction(up_direction);
            };
            if let Some(sensor_ceiling_right) = &mut self.sensor_ceiling_right {
                sensor_ceiling_right.set_position(top_right);
                sensor_ceiling_right.bind_mut().set_direction(up_direction);
            };
        }
        {
            // Push Sensors
            let half_width = self.push_radius;
            let mode = self.current_mode_walls();
            let half_height = if self.is_grounded
                && (self.ground_angle == 0.0 || self.ground_angle.approx_eq(&TAU))
            {
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
            };
            if let Some(sensor_push_right) = &mut self.sensor_push_right {
                sensor_push_right.set_position(center_right);
                sensor_push_right.bind_mut().set_direction(right_direction);
            };
        }
        self.update_shapes();
    }
    pub(super) fn update_shapes(&mut self) {
        let width = self.push_radius * 2.0;
        let height = self.height_radius * 2.0;
        let mode = self.current_mode();
        self.attacking = self.state.is_ball();

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
        if let Some(collision_shape) = self.hitbox_shape.as_deref_mut() {
            if let Some(mut rect) = collision_shape
                .get_shape()
                .and_then(|shape| shape.try_cast::<RectangleShape2D>().ok())
            {
                let mut size = Vector2::new(15.0, height - 3.0);
                if mode.is_sideways() {
                    size = Vector2::new(size.y, size.x);
                }
                rect.set_size(size);
            }
            collision_shape.set_debug_color(if self.attacking {
                Color::RED.with_alpha(0.2)
            } else {
                Color::BLUE.with_alpha(0.2)
            });
        }
    }
}
