#![allow(clippy::just_underscores_and_digits)]

use std::f32::consts::TAU;

use crate::sensor::{DetectionResult, Direction};

use super::Character;
use godot::{engine::RectangleShape2D, prelude::*};
use real_consts::{FRAC_PI_2, PI};

pub fn inverse_lerp(a: f32, b: f32, v: f32) -> f32 {
    (v - a) / (b - a)
}
/// From : <https://info.sonicretro.org/SPG:Slope_Collision#360_Degree_Collision>
#[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
pub(super) enum Mode {
    #[default]
    Floor,
    RightWall,
    Ceiling,
    LeftWall,
}

impl Mode {
    pub(super) fn angle(self) -> f32 {
        match self {
            Self::Floor => 0.0,
            Self::LeftWall => FRAC_PI_2,
            Self::Ceiling => PI,
            Self::RightWall => PI + FRAC_PI_2,
        }
    }
    pub(super) const fn down_direction(self) -> Direction {
        match self {
            Self::Floor => Direction::Down,
            Self::RightWall => Direction::Right,
            Self::Ceiling => Direction::Up,
            Self::LeftWall => Direction::Left,
        }
    }
    pub(super) const fn right_direction(self) -> Direction {
        match self {
            Self::Floor => Direction::Right,
            Self::RightWall => Direction::Up,
            Self::Ceiling => Direction::Left,
            Self::LeftWall => Direction::Down,
        }
    }
    pub(super) const fn left_direction(self) -> Direction {
        match self {
            Self::Floor => Direction::Left,
            Self::RightWall => Direction::Down,
            Self::Ceiling => Direction::Right,
            Self::LeftWall => Direction::Up,
        }
    }

    pub(super) const fn up_direction(self) -> Direction {
        match self {
            Self::Floor => Direction::Up,
            Self::RightWall => Direction::Left,
            Self::Ceiling => Direction::Down,
            Self::LeftWall => Direction::Right,
        }
    }
    pub(super) const fn down(self) -> Vector2 {
        match self {
            Self::Floor => Vector2::DOWN,
            Self::RightWall => Vector2::RIGHT,
            Self::Ceiling => Vector2::UP,
            Self::LeftWall => Vector2::LEFT,
        }
    }
    pub(super) const fn left(self) -> Vector2 {
        match self {
            Self::Floor => Vector2::LEFT,
            Self::RightWall => Vector2::DOWN,
            Self::Ceiling => Vector2::RIGHT,
            Self::LeftWall => Vector2::UP,
        }
    }
    pub(super) const fn right(self) -> Vector2 {
        match self {
            Self::Floor => Vector2::RIGHT,
            Self::RightWall => Vector2::UP,
            Self::Ceiling => Vector2::LEFT,
            Self::LeftWall => Vector2::DOWN,
        }
    }
}

impl Mode {
    #[allow(clippy::just_underscores_and_digits)]
    pub(super) fn from_ground_angle(angle: f32) -> Self {
        let _46 = f32::to_radians(45.0);
        let _135 = f32::to_radians(135.0);
        let _226 = f32::to_radians(226.0);
        let _315 = f32::to_radians(315.0);
        let _360 = f32::to_radians(360.0);

        if (0.0.._46).contains(&angle) || (_315..=_360).contains(&angle) {
            Self::Floor
        } else if (_46.._135).contains(&angle) {
            Self::RightWall
        } else if (_135.._226).contains(&angle) {
            Self::Ceiling
        } else if (_226.._315).contains(&angle) {
            Self::LeftWall
        } else {
            godot_warn!("out of range 0-360 angle {angle}");
            Self::default()
        }
    }
    #[allow(clippy::just_underscores_and_digits)]
    pub(super) fn from_wall_angle(angle: f32) -> Self {
        let _45 = f32::to_radians(45.0);
        let _136 = f32::to_radians(136.0);
        let _225 = f32::to_radians(225.0);
        let _316 = f32::to_radians(316.0);
        let _360 = f32::to_radians(360.0);
        if (0.0.._45).contains(&angle) || (_316..=_360).contains(&angle) {
            Self::Floor
        } else if (_45.._136).contains(&angle) {
            Self::RightWall
        } else if (_136.._225).contains(&angle) {
            Self::Ceiling
        } else if (_225.._316).contains(&angle) {
            Self::LeftWall
        } else {
            godot_warn!("out of range 0-360 angle {angle}");
            Self::default()
        }
    }

    pub(super) fn is_sideways(self) -> bool {
        self == Self::RightWall || self == Self::LeftWall
    }
}
/// From: <https://info.sonicretro.org/SPG:Slope_Collision#Airborne_Sensor_Activation>
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum MotionDirection {
    Right,
    Up,
    Left,
    Down,
}

impl MotionDirection {
    pub(super) fn from_velocity(velocity: Vector2) -> Self {
        if velocity.x.abs() > velocity.y.abs() {
            if velocity.x > 0.0 {
                Self::Right
            } else {
                Self::Left
            }
        } else if velocity.y > 0.0 {
            Self::Down
        } else {
            Self::Up
        }
    }

    pub(super) fn is_horizontal(self) -> bool {
        self == Self::Right || self == Self::Left
    }
}
impl Character {
    pub(super) fn set_sensor_size(&mut self, size: Vector2) {
        if let Some(mut shape) = self
            .sensor_shape
            .as_deref_mut()
            .and_then(|cs| cs.get_shape())
            .and_then(|shape| shape.try_cast::<RectangleShape2D>().ok())
        {
            shape.set_size(size);
        }
    }
    pub(super) fn set_hitbox_size(&mut self, size: Vector2) {
        if let Some(collision_shape) = self.hitbox_shape.as_deref_mut() {
            if let Some(mut rect) = collision_shape
                .get_shape()
                .and_then(|shape| shape.try_cast::<RectangleShape2D>().ok())
            {
                rect.set_size(size);
            }
        }
    }
    pub(super) fn play_animation(&mut self, animation: impl Into<StringName>) {
        if let Some(sprites) = &mut self.sprites {
            sprites.play_ex().name(animation.into()).done();
        }
    }
    pub(super) fn facing_left(&self) -> bool {
        if let Some(sprites) = &self.sprites {
            return sprites.is_flipped_h();
        }
        false
    }

    pub(super) fn global_position(&self) -> Vector2 {
        self.base().get_global_position()
    }
    pub(super) fn set_global_position(&mut self, value: Vector2) {
        self.base_mut().set_global_position(value);
    }
    pub(super) fn is_uphill(&self) -> bool {
        self.ground_speed.signum() == self.ground_angle.sin().signum()
    }
    /// From <https://info.sonicretro.org/SPG:Slope_Collision#Jump_Check>
    pub(super) fn can_jump(&self) -> bool {
        if let Some(result) = self.ceiling_check(false) {
            return result.distance >= 6.0;
        }
        true
    }
    /// From: <https://info.sonicretro.org/SPG:Rolling#Criteria>
    pub(super) fn can_roll(&self) -> bool {
        self.ground_speed.abs() > 1.0
    }

    /// From: <https://info.sonicretro.org/SPG:Slope_Physics#Falling_and_Slipping_Down_Slopes>
    #[allow(clippy::just_underscores_and_digits)]
    pub(super) fn is_slipping(&self) -> bool {
        // let _46 = f32::to_radians(46.0);
        // let _315 = f32::to_radians(315.0);
        // Sonic 1 , 2 and CD
        // (_46..=_315).contains(&self.ground_angle)

        let _35 = f32::to_radians(35.0);
        let _326 = f32::to_radians(326.0);
        // Sonic 3
        (_35..=_326).contains(&self.ground_angle)
    }
    /// From: <https://info.sonicretro.org/SPG:Slope_Physics#Falling_and_Slipping_Down_Slopes>
    pub(super) fn is_falling(&self) -> bool {
        let _69 = f32::to_radians(69.0);
        let _293 = f32::to_radians(293.0);
        // Sonic 3
        (_69..=_293).contains(&self.ground_angle)
    }
    /// From: <https://info.sonicretro.org/SPG:Slope_Collision#Ground_Sensors_.28Grounded.29>
    pub(super) fn should_snap_to_floor(&self, result: DetectionResult) -> bool {
        // Sonic 1
        // return result.distance > -14.0 && result.distance < 14.0;
        // Sonic 2 and onwards
        let mode = Mode::from_ground_angle(result.angle);
        let distance = result.distance;
        if mode.is_sideways() {
            distance <= (self.velocity.y.abs() + 4.0).min(14.0) && distance >= -14.0
        } else {
            distance <= (self.velocity.x.abs() + 4.0).min(14.0) && distance >= -14.0
        }
    }
    /// From: <https://info.sonicretro.org/SPG:Slope_Collision#Process_3>
    pub(super) fn is_landed(&self, result: DetectionResult) -> bool {
        if result.distance >= 0.0 {
            return false;
        }
        let direction = self.current_motion_direction();
        match direction {
            MotionDirection::Down => self
                .ground_sensor_results(false)
                .iter()
                .any(|r| r.distance >= -(self.velocity.y + 8.0)),
            MotionDirection::Right | MotionDirection::Left => self.velocity.y >= 0.0,
            MotionDirection::Up => false,
        }
    }
    /// From: <https://info.sonicretro.org/SPG:Slope_Collision#Process_4>
    pub(super) fn should_land_on_ceiling(&self) -> bool {
        let _91 = f32::to_radians(91.0);
        let _225 = f32::to_radians(225.0);
        let motion_direction = MotionDirection::from_velocity(self.velocity);
        (_91..=_225).contains(&self.ground_angle) && motion_direction == MotionDirection::Up
    }

    /// From: <https://info.sonicretro.org/SPG:Slope_Collision#Push_Sensors_.28Grounded.29>
    pub(super) fn should_activate_wall_sensors(&self) -> bool {
        let _270 = f32::to_radians(270.0);
        (0.0..=FRAC_PI_2).contains(&self.ground_angle)
            || (_270..=TAU).contains(&self.ground_angle)
            || self.ground_angle % FRAC_PI_2 == 0.0
    }

    pub(super) fn current_slope_factor(&self) -> f32 {
        if self.state.is_rolling() {
            if self.is_uphill() {
                godot_print!("uphill");
                self.slope_factor_rollup
            } else {
                godot_print!("rolldown");
                self.slope_factor_rolldown
            }
        } else {
            self.slope_factor_normal
        }
    }
    pub(super) const fn current_friction(&self) -> f32 {
        if self.state.is_rolling() {
            self.roll_friction
        } else {
            self.friction
        }
    }
    pub(super) const fn current_deceleration(&self) -> f32 {
        if self.state.is_rolling() {
            self.roll_deceleration
        } else {
            self.deceleration
        }
    }
    pub(super) fn current_motion_direction(&self) -> MotionDirection {
        MotionDirection::from_velocity(self.velocity)
    }
    pub(super) fn current_mode(&self) -> Mode {
        if self.is_grounded {
            Mode::from_ground_angle(self.ground_angle)
        } else {
            Mode::Floor
        }
    }
    pub(super) fn current_mode_walls(&self) -> Mode {
        if self.is_grounded {
            Mode::from_wall_angle(self.ground_angle)
        } else {
            Mode::Floor
        }
    }
}
