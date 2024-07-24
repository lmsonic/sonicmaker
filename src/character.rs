use godot::engine::{CharacterBody2D, Engine, ICharacterBody2D};
use godot::prelude::*;
use real_consts::{FRAC_PI_2, FRAC_PI_4, PI};

use crate::sensor::{DetectionResult, Direction, Sensor};
use crate::vec3_ext::Vector2Ext;

#[derive(GodotConvert, Var, Export, Default, Debug, PartialEq, Eq, Clone, Copy)]
#[godot(via = GString)]
enum Kind {
    #[default]
    Sonic,
    Tails,
    Knuckles,
}

#[derive(GodotConvert, Var, Export, Default, Debug, PartialEq, Eq, Clone, Copy)]
#[godot(via = GString)]
enum State {
    #[default]
    Standing,
    Ball,
}

#[derive(GodotConvert, Var, Export, Default, Debug, PartialEq, Eq, Clone, Copy)]
#[godot(via = GString)]
enum Mode {
    #[default]
    Floor,
    RightWall,
    Ceiling,
    LeftWall,
}

impl Mode {
    #[allow(clippy::just_underscores_and_digits)]
    fn from_ground_angle(angle: f32) -> Self {
        let _0 = f32::to_radians(0.0);
        let _45 = f32::to_radians(45.0);
        let _135 = f32::to_radians(135.0);
        let _224 = f32::to_radians(224.0);
        let _315 = f32::to_radians(315.0);
        let _360 = f32::to_radians(360.0);
        if (_0.._45).contains(&angle) || (_315.._360).contains(&angle) {
            Self::Floor
        } else if (_45.._135).contains(&angle) {
            Self::RightWall
        } else if (_135.._224).contains(&angle) {
            Self::Ceiling
        } else if (_224.._360).contains(&angle) {
            Self::LeftWall
        } else {
            godot_warn!("out of range 0-360 angle {angle}");
            Self::default()
        }
    }
    fn from_normal(normal: Vector2) -> Mode {
        Mode::from_ground_angle(normal.angle_0_360())
    }

    fn is_sideways(&self) -> bool {
        *self == Self::RightWall || *self == Self::LeftWall
    }
}

#[derive(GodotClass)]
#[class(tool,init, base=CharacterBody2D)]
pub struct Character {
    #[export]
    #[var(get, set = set_character)]
    character: Kind,
    #[export]
    #[var(get, set = set_state)]
    state: State,
    #[export(range = (0.0,100.0, 1.0))]
    #[var(get, set = set_width_radius)]
    #[init(default = 19.0)]
    width: f32,
    #[export(range = (0.0,100.0, 1.0))]
    #[var(get, set = set_height_radius)]
    #[init(default = 39.0)]
    height: f32,
    #[export]
    #[init(default = 20.0)]
    push_radius: f32,
    #[export]
    #[init(default = 6.5)]
    jump_force: f32,
    #[export]
    sensors: Option<Gd<Node2D>>,
    #[export]
    sensor_a: Option<Gd<Sensor>>,
    #[export]
    sensor_b: Option<Gd<Sensor>>,
    #[export]
    sensor_c: Option<Gd<Sensor>>,
    #[export]
    sensor_d: Option<Gd<Sensor>>,
    base: Base<CharacterBody2D>,
    #[export]
    is_grounded: bool,
    #[export]
    last_ground_normal: Vector2,
}

#[godot_api]
impl ICharacterBody2D for Character {
    fn physics_process(&mut self, delta: f64) {
        if Engine::singleton().is_editor_hint() {
            return;
        }

        if let Some(result) = self.get_ground_result() {
            if self.collides_with_floor(result) {
                self.last_ground_normal = result.normal;
                if self.is_grounded {
                    self.snap_to_floor(result.distance);
                }
                self.is_grounded = true;
                let ground_angle = self.last_ground_normal.angle_0_360();
                self.base_mut().set_rotation(ground_angle);
                if let Some(sensors) = &mut self.sensors {
                    let mode = Mode::from_ground_angle(ground_angle);
                    match mode {
                        Mode::Floor => sensors.set_rotation(0.0),
                        Mode::RightWall => sensors.set_rotation(FRAC_PI_2),
                        Mode::Ceiling => sensors.set_rotation(PI),
                        Mode::LeftWall => sensors.set_rotation(-FRAC_PI_2),
                    }
                }
            } else {
                self.is_grounded = false;
            }
        } else {
            self.is_grounded = false;
        }
        self.get_ceiling_result();
    }
}

#[godot_api]
impl Character {
    #[func]
    fn set_width_radius(&mut self, value: f32) {
        self.width = value;

        self.update_sensors();
    }

    #[func]
    fn set_height_radius(&mut self, value: f32) {
        self.height = value;
        self.update_sensors();
    }

    #[func]
    fn set_character(&mut self, value: Kind) {
        match value {
            Kind::Sonic => {
                self.set_width_radius(19.0);
                self.set_height_radius(39.0);
                self.jump_force = 6.5;
            }
            Kind::Tails => {
                self.set_width_radius(19.0);
                self.set_height_radius(31.0);
                self.jump_force = 6.5;
            }
            Kind::Knuckles => {
                self.set_width_radius(19.0);
                self.set_height_radius(39.0);
                self.jump_force = 6.0;
            }
        }

        self.character = value;
    }
    #[func]
    fn set_state(&mut self, value: State) {
        match (self.state, value) {
            (State::Standing, State::Ball) => {
                self.set_height_radius(36.0);
                self.set_width_radius(16.0);
            }
            (State::Ball, State::Standing) => {
                self.set_character(self.character);
            }
            _ => {}
        }
        self.state = value;
    }
    #[func]
    pub fn update_sensors(&mut self) {
        let half_width = (self.width / 2.0).floor();
        let half_height = (self.height / 2.0).floor();
        let mask = self.base().get_collision_layer();

        if let Some(sensor_a) = &mut self.sensor_a {
            sensor_a.set_position(Vector2::new(-half_width, half_height));
            sensor_a.bind_mut().set_direction(Direction::Down);
            sensor_a.set_collision_mask(mask);
        };
        if let Some(sensor_b) = &mut self.sensor_b {
            sensor_b.set_position(Vector2::new(half_width, half_height));
            sensor_b.bind_mut().set_direction(Direction::Down);
            sensor_b.set_collision_mask(mask);
        };
        if let Some(sensor_c) = &mut self.sensor_c {
            sensor_c.set_position(Vector2::new(-half_width, -half_height));
            sensor_c.bind_mut().set_direction(Direction::Up);
            sensor_c.set_collision_mask(mask);
        };
        if let Some(sensor_d) = &mut self.sensor_d {
            sensor_d.set_position(Vector2::new(half_width, -half_height));
            sensor_d.bind_mut().set_direction(Direction::Up);
            sensor_d.set_collision_mask(mask);
        };
    }
}

impl Character {
    fn snap_to_floor(&mut self, distance: f32) {
        let mode = Mode::from_normal(self.last_ground_normal);
        let mut position = self.base().get_global_position();
        if mode.is_sideways() {
            position.x -= distance;
        } else {
            position.y += distance;
        }
        self.base_mut().set_global_position(position);
    }
    fn collides_with_floor(&self, result: DetectionResult) -> bool {
        // Sonic 1
        return result.distance > -14.0 && result.distance < 14.0;
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
    fn get_ground_result(&mut self) -> Option<DetectionResult> {
        let mut results = vec![];
        if let Some(sensor_a) = &mut self.sensor_a {
            if let Ok(r) = sensor_a
                .bind_mut()
                .detect_solid()
                .try_to::<DetectionResult>()
            {
                results.push(r);
            }
        };
        if let Some(sensor_b) = &mut self.sensor_b {
            if let Ok(r) = sensor_b
                .bind_mut()
                .detect_solid()
                .try_to::<DetectionResult>()
            {
                results.push(r);
            }
        };
        results
            .into_iter()
            .min_by(|a, b| a.distance.total_cmp(&b.distance))
    }
    fn get_ceiling_result(&mut self) -> Option<DetectionResult> {
        let mut results = vec![];
        if let Some(sensor_c) = &mut self.sensor_c {
            if let Ok(r) = sensor_c
                .bind_mut()
                .detect_solid()
                .try_to::<DetectionResult>()
            {
                results.push(r);
            }
        };
        if let Some(sensor_d) = &mut self.sensor_d {
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
            .max_by(|a, b| a.distance.total_cmp(&b.distance))
    }
}
