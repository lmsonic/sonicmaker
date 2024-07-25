use std::f32::consts::{FRAC_PI_2, PI};

use godot::{engine::RectangleShape2D, prelude::*};

use crate::{
    character::Character,
    sensor::{DetectionResult, Direction},
    vec3_ext::Vector2Ext,
};

#[derive(GodotConvert, Var, Export, Default, Debug, PartialEq, Eq, Clone, Copy)]
#[godot(via = GString)]
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

    pub(super) fn up_direction(&self) -> Direction {
        match self {
            Mode::Floor => Direction::Up,
            Mode::RightWall => Direction::Left,
            Mode::Ceiling => Direction::Down,
            Mode::LeftWall => Direction::Right,
        }
    }
}

impl Mode {
    #[allow(clippy::just_underscores_and_digits)]
    pub(super) fn from_ground_angle(angle: f32) -> Self {
        let _0 = f32::to_radians(0.0);
        let _46 = f32::to_radians(46.0);
        let _135 = f32::to_radians(135.0);
        let _226 = f32::to_radians(226.0);
        let _315 = f32::to_radians(315.0);
        let _360 = f32::to_radians(360.0);
        if (_0.._46).contains(&angle) || (_315..=_360).contains(&angle) {
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
        let _0 = f32::to_radians(0.0);
        let _45 = f32::to_radians(45.0);
        let _136 = f32::to_radians(136.0);
        let _225 = f32::to_radians(225.0);
        let _316 = f32::to_radians(316.0);
        let _360 = f32::to_radians(360.0);
        if (_0.._45).contains(&angle) || (_316..=_360).contains(&angle) {
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

impl Character {
    pub(super) fn current_mode(&self) -> Mode {
        if self.is_grounded {
            Mode::from_ground_angle(self.last_ground_angle)
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
        let mut position = self.base().get_global_position();
        if mode.is_sideways() {
            position.x += distance
        } else {
            position.y += distance;
        }
        self.base_mut().set_global_position(position);
    }
    pub(super) fn should_snap_to_floor(&self, result: DetectionResult) -> bool {
        // Sonic 1
        result.distance > -14.0 && result.distance < 14.0
        // TODO: use this accounting for speed
        // Sonic 2 and onwards
        // let mode = Mode::from_normal(result.normal);
        // let velocity = self.base().get_velocity();
        // let distance = result.distance;
        // if mode.is_sideways() {
        //     distance <= (velocity.x.abs() + 4.0).min(14.0) && distance >= -14.0
        // } else {
        //     distance <= (velocity.y.abs() + 4.0).min(14.0) && distance >= -14.0
        // }
    }
    pub(super) fn is_landed(&mut self, result: DetectionResult) -> bool {
        enum MotionDirection {
            Right,
            Up,
            Left,
            Down,
        }

        impl MotionDirection {
            #[allow(clippy::just_underscores_and_digits)]
            fn from_velocity_angle(angle: f32) -> Self {
                let _0 = f32::to_radians(0.0);
                let _46 = f32::to_radians(46.0);
                let _136 = f32::to_radians(136.0);
                let _226 = f32::to_radians(226.0);
                let _316 = f32::to_radians(316.0);
                let _360 = f32::to_radians(360.0);
                if (_0.._46).contains(&angle) || (_316.._360).contains(&angle) {
                    Self::Right
                } else if (_46.._136).contains(&angle) {
                    Self::Up
                } else if (_136.._226).contains(&angle) {
                    Self::Down
                } else if (_226.._316).contains(&angle) {
                    Self::Left
                } else {
                    godot_warn!("out of range 0-360 angle {angle}");
                    Self::Down
                }
            }
        }
        result.distance.abs() < 4.0
        // TODO: use this once we have a controller
        // if result.distance > 0.0 {
        //     return false;
        // }
        // let velocity = self.base().get_velocity();
        // let motion_angle = velocity.angle_0_360();
        // let direction = MotionDirection::from_velocity_angle(motion_angle);
        // match direction {
        //     MotionDirection::Right | MotionDirection::Left => self
        //         .ground_sensor_results()
        //         .iter()
        //         .any(|r| r.distance >= -velocity.y + 8.0),
        //     MotionDirection::Down => velocity.y >= 0.0,
        //     MotionDirection::Up => false,
        // }
    }
    pub(super) fn ground_sensor_results(&mut self) -> Vec<DetectionResult> {
        let mut results = vec![];
        if let Some(sensor_a) = &mut self.sensor_floor_left {
            if let Ok(r) = sensor_a
                .bind_mut()
                .detect_solid()
                .try_to::<DetectionResult>()
            {
                results.push(r);
            }
        };
        if let Some(sensor_b) = &mut self.sensor_floor_right {
            if let Ok(r) = sensor_b
                .bind_mut()
                .detect_solid()
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
            .min_by(|a, b| a.distance.abs().total_cmp(&b.distance.abs()))
    }
    pub(super) fn ceiling_check(&mut self) -> Option<DetectionResult> {
        let mut results = vec![];
        if let Some(sensor_c) = &mut self.sensor_ceiling_left {
            if let Ok(r) = sensor_c
                .bind_mut()
                .detect_solid()
                .try_to::<DetectionResult>()
            {
                results.push(r);
            }
        };
        if let Some(sensor_d) = &mut self.sensor_ceiling_right {
            if let Ok(r) = sensor_d
                .bind_mut()
                .detect_solid()
                .try_to::<DetectionResult>()
            {
                results.push(r);
            }
        };
        results
            .into_iter()
            .min_by(|a, b| a.distance.abs().total_cmp(&b.distance.abs()))
    }
}
