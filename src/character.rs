mod collision_checking;

use collision_checking::Mode;
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
    #[init(default = 10.0)]
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
    sensor_push_left: Option<Gd<Sensor>>,
    #[export]
    sensor_push_right: Option<Gd<Sensor>>,
    #[export]
    #[var(get,set= set_grounded)]
    pub(crate) is_grounded: bool,
    #[export(range = (0.0, 360.0, 0.001, radians_as_degrees))]
    #[var(get,set= set_ground_angle)]
    pub(crate) last_ground_angle: f32,

    ground_speed: f32,

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
        let input = Input::singleton();
        if self.is_grounded {
            // Grounded
            // Jump Check
            if input.is_action_just_pressed(c"jump".into()) && self.can_jump() {
                let mut velocity = self.velocity();
                velocity += Vector2::UP * self.jump_force;
                self.set_velocity(velocity);
            }

            // Wall checking
            if self.should_activate_wall_sensors() {
                if self.ground_speed > 0.0 {
                    if let Some(result) = self.right_sensor_check() {
                        if result.distance.abs() <= 0.0 {
                            self.ground_speed = 0.0;
                            let mut velocity = self.velocity();
                            let right = self.current_mode().right();
                            velocity += right * result.distance;
                            self.set_velocity(velocity);
                        }
                    }
                } else if let Some(result) = self.left_sensor_check() {
                    if result.distance.abs() <= 0.0 {
                        self.ground_speed = 0.0;
                        let mut velocity = self.velocity();
                        let left = self.current_mode().left();
                        velocity += left;
                        self.set_velocity(velocity);
                    }
                }
            }
            // Floor checking
            if let Some(result) = self.ground_check() {
                if self.should_snap_to_floor(result) {
                    self.snap_to_floor(result.distance);
                    let ground_angle = result.normal.plane_angle();
                    self.set_ground_angle(ground_angle);
                } else {
                    self.set_grounded(false);
                }
            } else {
                self.set_grounded(false);
            }
        } else {
            // Airborne

            // Floor check
            if let Some(result) = self.ground_check() {
                if self.is_landed(result) {
                    self.snap_to_floor(result.distance);
                    let ground_angle = result.normal.plane_angle();
                    self.set_ground_angle(ground_angle);
                    self.set_grounded(true);
                }
            }

            // Ceiling check
            self.ceiling_check();
        }
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
        let mask = self.base().get_collision_layer();
        {
            let half_width = self.width_radius;
            let half_height = self.height_radius;
            let mode = self.current_mode();

            // Floor and ceiling sensors
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
        }
        {
            // Push Sensors
            let half_width = self.push_radius;
            let mode = Mode::from_wall_angle(self.last_ground_angle);
            let half_height = if mode == Mode::Floor { 8.0 } else { 0.0 };
            let mut right_direction = mode.right_direction();
            let mut left_direction = mode.left_direction();
            let mut angle = mode.angle();
            if mode.is_sideways() {
                // I have no idea, Glam and Godot have different ideas of rotation?
                angle = -angle;
                std::mem::swap(&mut left_direction, &mut right_direction);
            }
            let center_left = Vector2::new(-half_width, half_height).rotated(angle);
            let center_right = Vector2::new(half_width, half_height).rotated(angle);
            if let Some(sensor_push_left) = &mut self.sensor_push_left {
                sensor_push_left.set_position(center_left);
                sensor_push_left.bind_mut().set_direction(left_direction);
                sensor_push_left.set_collision_mask(mask);
            };
            if let Some(sensor_push_right) = &mut self.sensor_push_right {
                sensor_push_right.set_position(center_right);
                sensor_push_right.bind_mut().set_direction(right_direction);
                sensor_push_right.set_collision_mask(mask);
            };
        }
        self.update_shapes();
    }
}
impl Character {
    fn update_y_position(&mut self, delta: f32) {
        let mut position = self.base().get_global_position();
        let down = self.current_mode().down();
        position += down * delta;
        self.base_mut().set_global_position(position)
    }
    fn velocity(&self) -> Vector2 {
        self.base().get_velocity()
    }
    fn set_velocity(&mut self, value: Vector2) {
        self.base_mut().set_velocity(value)
    }
    fn position(&self) -> Vector2 {
        self.base().get_position()
    }
    fn set_position(&mut self, value: Vector2) {
        self.base_mut().set_position(value)
    }
    fn global_position(&self) -> Vector2 {
        self.base().get_global_position()
    }
    fn set_global_position(&mut self, value: Vector2) {
        self.base_mut().set_global_position(value)
    }
}
