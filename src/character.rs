mod collision_checking;

use godot::engine::{
    AnimatedSprite2D, CharacterBody2D, CollisionShape2D, Engine, ICharacterBody2D,
};
use godot::prelude::*;

use crate::sensor::Sensor;
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
    #[var(get,set= set_grounded)]
    pub(crate) is_grounded: bool,
    #[export(range = (0.0, 360.0, 0.001, radians_as_degrees))]
    #[var(get,set= set_ground_angle)]
    pub(crate) last_ground_angle: f32,

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
            if self.is_grounded {
                // Grounded
                if self.should_snap_to_floor(result) {
                    self.snap_to_floor(result.distance);
                    let ground_angle = result.normal.plane_angle();
                    self.set_ground_angle(ground_angle);
                } else {
                    self.set_grounded(false);
                }
            } else {
                // Airborne
                if self.is_landed(result) {
                    self.snap_to_floor(result.distance);
                    let ground_angle = result.normal.plane_angle();
                    self.set_ground_angle(ground_angle);
                    self.set_grounded(true);
                }
            }
        }
        self.ceiling_check();
    }
}

#[godot_api]
impl Character {
    #[func]
    fn set_grounded(&mut self, value: bool) {
        self.is_grounded = value;
        self.update_sensors();
    }
    #[func]
    fn set_ground_angle(&mut self, value: f32) {
        self.last_ground_angle = value;
        self.base_mut().set_rotation(value);
        self.update_sensors();
    }
    #[func]
    fn set_width_radius(&mut self, value: f32) {
        self.width_radius = value;
        self.update_sensors();
    }

    #[func]
    fn set_height_radius(&mut self, value: f32) {
        self.height_radius = value;
        self.update_sensors();
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
        let mode = self.current_mode();

        let mut down_direction = mode.down_direction();
        let mut up_direction = mode.up_direction();
        let mut angle = mode.angle();
        if mode.is_sideways() {
            // I have no idea, Glam and Godot have different ideas of rotation?
            angle = -angle;
            std::mem::swap(&mut down_direction, &mut up_direction);
        }
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
        self.update_shapes();
    }
}
