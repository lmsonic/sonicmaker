use std::f32::consts::{FRAC_PI_2, PI};

use godot::{engine::RectangleShape2D, prelude::*};

use crate::{
    character::Character,
    sensor::{DetectionResult, Direction},
    vec3_ext::Vector2Ext,
};

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
        if velocity.x > velocity.y {
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
    pub(super) fn grounded_right_wall_collision(&mut self, distance: f32) {
        godot_print!("Right wall collision");

        let mut velocity = self.velocity();
        let right = self.current_mode().right();
        velocity += right * distance;
        godot_print!("{}", right * distance);
        self.ground_speed = 0.0;
        self.set_velocity(velocity);
    }
    pub(super) fn grounded_left_wall_collision(&mut self, distance: f32) {
        godot_print!("Left wall collision");

        let mut velocity = self.velocity();
        let left = self.current_mode().left();
        velocity += left * distance;
        godot_print!("{}", left * distance);
        self.ground_speed = 0.0;
        self.set_velocity(velocity);
    }
    pub(super) fn airborne_left_wall_collision(&mut self, distance: f32) {
        godot_print!("Left wall collision");
        let mut position = self.global_position();
        position.x += distance;
        self.set_global_position(position);

        let velocity = self.velocity();
        self.set_velocity(Vector2::new(0.0, velocity.y));
    }
    pub(super) fn airborne_right_wall_collision(&mut self, distance: f32) {
        godot_print!("Right wall collision");
        let mut position = self.global_position();
        position.x -= distance;
        self.set_global_position(position);

        let velocity = self.velocity();
        self.set_velocity(Vector2::new(0.0, velocity.y));
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
    pub(super) fn snap_to_floor(&mut self, distance: f32) {
        let mode = self.current_mode();
        let mut position = self.global_position();
        if mode.is_sideways() {
            position.x += distance;
        } else {
            position.y += distance;
        }
        self.set_global_position(position);
    }
    pub(super) fn should_snap_to_floor(&self, result: DetectionResult) -> bool {
        // Sonic 1
        // result.distance > -14.0 && result.distance < 14.0
        // Sonic 2 and onwards
        let mode = Mode::from_normal(result.normal);
        let velocity = self.base().get_velocity();
        let distance = result.distance;
        if mode.is_sideways() {
            distance <= (velocity.x.abs() + 4.0).min(14.0) && distance >= -14.0
        } else {
            distance <= (velocity.y.abs() + 4.0).min(14.0) && distance >= -14.0
        }
    }
    pub(super) fn is_landed(&mut self, result: DetectionResult) -> bool {
        if result.distance < -14.0 || result.distance > 14.0 {
            return false;
        }
        let velocity = self.velocity();
        let direction = MotionDirection::from_velocity(velocity);
        match direction {
            MotionDirection::Right | MotionDirection::Left => self
                .ground_sensor_results()
                .iter()
                .any(|r| r.distance >= -(velocity.y + 8.0)),
            MotionDirection::Down => velocity.y >= 0.0,
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
            return result.distance > 6.0;
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
            if let Ok(result) = sensor_push_left
                .bind_mut()
                .detect_solid()
                .try_to::<DetectionResult>()
            {
                return Some(result);
            }
            sensor_push_left.set_global_position(old_position);
        };
        None
    }
    pub(super) fn wall_right_sensor_check(&mut self) -> Option<DetectionResult> {
        let velocity = self.velocity();
        if let Some(sensor_push_right) = &mut self.sensor_push_right {
            let old_position = sensor_push_right.get_global_position();
            let new_position = old_position + velocity;
            sensor_push_right.set_global_position(new_position);
            if let Ok(result) = sensor_push_right
                .bind_mut()
                .detect_solid()
                .try_to::<DetectionResult>()
            {
                return Some(result);
            }
            sensor_push_right.set_global_position(old_position);
        };
        None
    }
}
