use crate::{
    sensor::{DetectionResult, Direction},
    vec3_ext::Vector2Ext,
};

use super::Character;
use godot::prelude::*;
use real_consts::{FRAC_PI_2, PI};

#[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
pub(super) enum Mode {
    #[default]
    Floor,
    RightWall,
    Ceiling,
    LeftWall,
}

impl Mode {
    pub(super) fn angle(&self) -> f32 {
        match self {
            Mode::Floor => 0.0,
            Mode::LeftWall => FRAC_PI_2,
            Mode::Ceiling => PI,
            Mode::RightWall => PI + FRAC_PI_2,
        }
    }
    pub(super) fn down_direction(&self) -> Direction {
        match self {
            Mode::Floor => Direction::Down,
            Mode::RightWall => Direction::Right,
            Mode::Ceiling => Direction::Up,
            Mode::LeftWall => Direction::Left,
        }
    }
    pub(super) fn right_direction(&self) -> Direction {
        match self {
            Mode::Floor => Direction::Right,
            Mode::RightWall => Direction::Up,
            Mode::Ceiling => Direction::Left,
            Mode::LeftWall => Direction::Down,
        }
    }
    pub(super) fn left_direction(&self) -> Direction {
        match self {
            Mode::Floor => Direction::Left,
            Mode::RightWall => Direction::Down,
            Mode::Ceiling => Direction::Right,
            Mode::LeftWall => Direction::Up,
        }
    }

    pub(super) fn up_direction(&self) -> Direction {
        match self {
            Mode::Floor => Direction::Up,
            Mode::RightWall => Direction::Left,
            Mode::Ceiling => Direction::Down,
            Mode::LeftWall => Direction::Right,
        }
    }
    pub(super) fn down(&self) -> Vector2 {
        match self {
            Mode::Floor => Vector2::DOWN,
            Mode::RightWall => Vector2::RIGHT,
            Mode::Ceiling => Vector2::UP,
            Mode::LeftWall => Vector2::LEFT,
        }
    }
    pub(super) fn left(&self) -> Vector2 {
        match self {
            Mode::Floor => Vector2::LEFT,
            Mode::RightWall => Vector2::UP,
            Mode::Ceiling => Vector2::RIGHT,
            Mode::LeftWall => Vector2::DOWN,
        }
    }
    pub(super) fn right(&self) -> Vector2 {
        match self {
            Mode::Floor => Vector2::RIGHT,
            Mode::RightWall => Vector2::DOWN,
            Mode::Ceiling => Vector2::LEFT,
            Mode::LeftWall => Vector2::UP,
        }
    }
}

impl Mode {
    #[allow(clippy::just_underscores_and_digits)]
    pub(super) fn from_ground_angle(angle: f32) -> Self {
        let _46 = f32::to_radians(46.0);
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
    pub(super) fn from_normal(normal: Vector2) -> Mode {
        Mode::from_ground_angle(normal.plane_angle())
    }

    pub(super) fn is_sideways(&self) -> bool {
        *self == Self::RightWall || *self == Self::LeftWall
    }
}
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

    pub(super) fn is_horizontal(&self) -> bool {
        *self == Self::Right || *self == Self::Left
    }
}
impl Character {
    pub(super) fn velocity(&self) -> Vector2 {
        self.base().get_velocity()
    }
    pub(super) fn set_velocity(&mut self, value: Vector2) {
        self.base_mut().set_velocity(value)
    }

    pub(super) fn global_position(&self) -> Vector2 {
        self.base().get_global_position()
    }
    pub(super) fn set_global_position(&mut self, value: Vector2) {
        self.base_mut().set_global_position(value)
    }
    pub(super) fn is_uphill(&self) -> bool {
        self.ground_speed.signum() == self.ground_angle.sin().signum()
    }

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
    #[allow(clippy::just_underscores_and_digits)]
    pub(super) fn is_falling(&self) -> bool {
        let _69 = f32::to_radians(69.0);
        let _293 = f32::to_radians(293.0);
        // Sonic 3
        (_69..=_293).contains(&self.ground_angle)
    }
    pub(super) fn should_snap_to_floor(&self, result: DetectionResult) -> bool {
        // Sonic 1
        // result.distance > -14.0 && result.distance < 14.0
        // Sonic 2 and onwards
        let mode = Mode::from_normal(result.normal);
        let velocity = self.velocity();
        let distance = result.distance;
        if mode.is_sideways() {
            distance <= (velocity.x.abs() + 4.0).min(14.0) && distance >= -14.0
        } else {
            distance <= (velocity.y.abs() + 4.0).min(14.0) && distance >= -14.0
        }
    }
    pub(super) fn is_landed(&mut self, result: DetectionResult) -> bool {
        if result.distance >= 0.0 {
            return false;
        }
        let velocity = self.velocity();

        let direction = MotionDirection::from_velocity(velocity);
        godot_print!("{direction:?}");
        match direction {
            MotionDirection::Down => self
                .ground_sensor_results()
                .iter()
                .any(|r| r.distance >= -(velocity.y + 8.0)),
            MotionDirection::Right | MotionDirection::Left => velocity.y >= 0.0,
            MotionDirection::Up => false,
        }
    }
    #[allow(clippy::just_underscores_and_digits)]
    pub(super) fn should_land_on_ceiling(&self) -> bool {
        let _91 = f32::to_radians(91.0);
        let _225 = f32::to_radians(225.0);
        let velocity = self.velocity();
        let motion_direction = MotionDirection::from_velocity(velocity);
        (_91..=_225).contains(&self.ground_angle) && motion_direction == MotionDirection::Up
    }
    #[allow(clippy::just_underscores_and_digits)]
    pub(super) fn should_activate_wall_sensors(&self) -> bool {
        let _90 = f32::to_radians(90.0);
        let _270 = f32::to_radians(270.0);
        let _360 = f32::to_radians(360.0);
        (0.0..=_90).contains(&self.ground_angle)
            || (_270..=_360).contains(&self.ground_angle)
            || self.ground_angle % _90 == 0.0
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
    pub(super) fn current_friction(&self) -> f32 {
        if self.state.is_rolling() {
            self.roll_friction
        } else {
            self.friction
        }
    }
    pub(super) fn current_motion_direction(&self) -> MotionDirection {
        MotionDirection::from_velocity(self.base().get_velocity())
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
