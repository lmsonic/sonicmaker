use godot::engine::{
    AnimatedSprite2D, CharacterBody2D, CollisionShape2D, Engine, ICharacterBody2D, RectangleShape2D,
};
use godot::prelude::*;
use real_consts::{FRAC_PI_2, PI};

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
    Idle,
    StartMotion,
    FullMotion,
    AirBall,
    RollingBall,
}

impl State {
    fn is_ball(&self) -> bool {
        *self == Self::AirBall || *self == Self::RollingBall
    }
    fn is_grounded(&self) -> bool {
        *self != Self::AirBall
    }
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
    fn angle(&self) -> f32 {
        match self {
            Mode::Floor => 0.0,
            Mode::RightWall => -FRAC_PI_2,
            Mode::Ceiling => PI,
            Mode::LeftWall => FRAC_PI_2,
        }
    }
    fn down_direction(&self) -> Direction {
        match self {
            Mode::Floor => Direction::Down,
            Mode::RightWall => Direction::Right,
            Mode::Ceiling => Direction::Up,
            Mode::LeftWall => Direction::Left,
        }
    }
    fn up_direction(&self) -> Direction {
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
        Mode::from_ground_angle(normal.plane_angle())
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
    #[export]
    sprites: Option<Gd<AnimatedSprite2D>>,
    #[export(range = (0.0,100.0, 1.0))]
    #[var(get, set = set_width_radius)]
    #[init(default = 9.0)]
    width_radius: f32,
    #[export(range = (0.0,100.0, 1.0))]
    #[var(get, set = set_height_radius)]
    #[init(default = 19.0)]
    height_radius: f32,
    #[export]
    #[init(default = 20.0)]
    push_radius: f32,
    #[export]
    #[init(default = 6.5)]
    jump_force: f32,
    #[export]
    sensor_shape: Option<Gd<CollisionShape2D>>,
    #[export]
    hitbox_shape: Option<Gd<CollisionShape2D>>,
    #[export]
    sensor_floor_left: Option<Gd<Sensor>>,
    #[export]
    sensor_floor_right: Option<Gd<Sensor>>,
    #[export]
    sensor_ceiling_left: Option<Gd<Sensor>>,
    #[export]
    sensor_ceiling_right: Option<Gd<Sensor>>,
    #[export]
    is_grounded: bool,
    #[export]
    last_ground_angle: f32,
    #[export]
    #[var(get,set= set_mode)]
    mode: Mode,
    #[export]
    enable_in_editor: bool,
    base: Base<CharacterBody2D>,
}

#[godot_api]
impl ICharacterBody2D for Character {
    fn physics_process(&mut self, _delta: f64) {
        if Engine::singleton().is_editor_hint() && !self.enable_in_editor {
            return;
        }
        if let Some(result) = self.ground_check() {
            if self.collides_with_floor(result) {
                self.last_ground_angle = result.normal.plane_angle();
                if self.is_grounded {
                    self.snap_to_floor(result.distance);
                    let ground_angle = result.normal.plane_angle();
                    self.base_mut().set_rotation(ground_angle);
                    self.set_mode(Mode::from_ground_angle(ground_angle))
                }
                self.is_grounded = true;
            } else {
                self.is_grounded = false;
            }
        } else {
            self.is_grounded = false;
        }
        self.ceiling_check();
    }
}

#[godot_api]
impl Character {
    #[func]
    fn set_mode(&mut self, value: Mode) {
        self.mode = value;
        self.update_sensors();
        self.update_shapes();
    }
    #[func]
    fn set_width_radius(&mut self, value: f32) {
        self.width_radius = value;
        self.update_sensors();
        self.update_shapes();
    }

    #[func]
    fn set_height_radius(&mut self, value: f32) {
        let delta = value - self.height_radius;
        self.height_radius = value;
        self.update_sensors();
        self.update_shapes();
        self.adjust_y_position(delta);
    }

    #[func]
    fn set_character(&mut self, value: Kind) {
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
    fn set_state(&mut self, value: State) {
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
                State::AirBall | State::RollingBall => {
                    sprites.play_ex().name(c"rolling".into()).done()
                }
            }
        }
    }
    #[func]
    pub fn update_sensors(&mut self) {
        let half_width = self.width_radius;
        let half_height = self.height_radius;
        let mask = self.base().get_collision_layer();

        let down_direction = self.mode.down_direction();
        let up_direction = self.mode.up_direction();

        let angle = self.mode.angle();
        let bottom_left = Vector2::new(-half_width, half_height).rotated(angle);
        let bottom_right = Vector2::new(half_width, half_height).rotated(angle);
        let top_left = Vector2::new(-half_width, -half_height).rotated(angle);
        let top_right = Vector2::new(half_width, -half_height).rotated(angle);

        if let Some(sensor_floor_left) = &mut self.sensor_floor_left {
            sensor_floor_left.set_position(bottom_left);
            sensor_floor_left.bind_mut().set_direction(down_direction);
            sensor_floor_left.set_collision_mask(mask);
        };
        if let Some(sensor_floor_right) = &mut self.sensor_floor_right {
            sensor_floor_right.set_position(bottom_right);
            sensor_floor_right.bind_mut().set_direction(down_direction);
            sensor_floor_right.set_collision_mask(mask);
        };
        if let Some(sensor_ceiling_left) = &mut self.sensor_ceiling_left {
            sensor_ceiling_left.set_position(top_left);
            sensor_ceiling_left.bind_mut().set_direction(up_direction);
            sensor_ceiling_left.set_collision_mask(mask);
        };
        if let Some(sensor_ceiling_right) = &mut self.sensor_ceiling_right {
            sensor_ceiling_right.set_position(top_right);
            sensor_ceiling_right.bind_mut().set_direction(up_direction);
            sensor_ceiling_right.set_collision_mask(mask);
        };
    }
}

impl Character {
    pub fn adjust_y_position(&mut self, delta: f32) {
        let mut position = self.base().get_position();
        position.y -= delta;
        self.base_mut().set_position(position);
    }
    pub fn update_shapes(&mut self) {
        let width = self.width_radius * 2.0 + 1.0;
        let height = self.height_radius * 2.0 + 1.0;
        if let Some(mut shape) = self
            .sensor_shape
            .as_deref_mut()
            .and_then(|cs| cs.get_shape())
            .and_then(|shape| shape.try_cast::<RectangleShape2D>().ok())
        {
            let mut size = Vector2::new(width, height);
            if self.mode.is_sideways() {
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
            if self.mode.is_sideways() {
                size = Vector2::new(size.y, size.x);
            }
            hitbox.set_size(size);
        }
    }
    fn snap_to_floor(&mut self, distance: f32) {
        let mode = Mode::from_ground_angle(self.last_ground_angle);
        let mut position = self.base().get_global_position();
        if mode.is_sideways() {
            position.x += distance
        } else {
            position.y += distance;
        }
        self.base_mut().set_global_position(position);
    }
    fn collides_with_floor(&self, result: DetectionResult) -> bool {
        // Sonic 1
        result.distance > -14.0 && result.distance < 14.0
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
    fn ground_check(&mut self) -> Option<DetectionResult> {
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
        };
        results
            .into_iter()
            .min_by(|a, b| a.distance.total_cmp(&b.distance))
    }
    fn ceiling_check(&mut self) -> Option<DetectionResult> {
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
            .max_by(|a, b| a.distance.total_cmp(&b.distance))
    }
}
