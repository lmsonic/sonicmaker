mod collision_checking;

use collision_checking::{Mode, MotionDirection};
use godot::engine::{
    AnimatedSprite2D, CharacterBody2D, CollisionShape2D, Engine, ICharacterBody2D,
};
use godot::prelude::*;
use real_consts::{PI, TAU};

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

    #[must_use]
    fn is_rolling(&self) -> bool {
        matches!(self, Self::RollingBall)
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
    #[init(default = 0.09375)]
    air_acceleration: f32,
    #[init(default = 0.046875)]
    acceleration: f32,
    #[init(default = 0.5)]
    deceleration: f32,
    #[init(default = 0.046875)]
    friction: f32,
    #[init(default = 6.0)]
    top_speed: f32,
    #[init(default = 0.21875)]
    gravity: f32,
    #[init(default = 0.125)]
    slope_factor_normal: f32,
    #[init(default = 0.078125)]
    slope_factor_rollup: f32,
    #[init(default = 0.3125)]
    slope_factor_rolldown: f32,
    #[export]
    ground_speed: f32,
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
    pub(crate) ground_angle: f32,
    control_lock_timer: i32,
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
            godot_print!("Grounded");
            // Slow down uphill and speeding up downhill
            if self.current_mode() != Mode::Ceiling {
                let slope_factor = self.current_slope_factor() * self.ground_angle.sin();
                // Forces moving when walking on steep slopes
                let is_stopped = self.ground_speed != 0.0;
                let is_rolling = self.state.is_rolling();
                let is_on_steep = slope_factor >= 0.05078125;
                if !is_stopped || is_rolling || is_on_steep {
                    godot_print!("Applying slope factor {slope_factor}");
                    self.ground_speed -= slope_factor * self.ground_angle.sin();
                }
            }
            // Input
            let mut velocity = self.velocity();

            if self.control_lock_timer <= 0 {
                // Ground Acceleration
                if input.is_action_pressed(c"left".into()) {
                    if self.ground_speed > 0.0 {
                        // Turn around
                        godot_print!("Turn around");
                        self.ground_speed -= self.deceleration;
                        if self.ground_speed <= 0.0 {
                            self.ground_speed = -0.5;
                        }
                    } else if self.ground_speed > -self.top_speed {
                        godot_print!("Accelerate left");
                        self.ground_speed -= self.acceleration;
                        self.ground_speed = self.ground_speed.max(-self.top_speed);
                    }
                    self.set_state(if self.ground_speed <= -self.top_speed {
                        State::FullMotion
                    } else {
                        State::StartMotion
                    });
                    self.set_flip_h(true);
                }
                if input.is_action_pressed(c"right".into()) {
                    if self.ground_speed < 0.0 {
                        // Turn around
                        godot_print!("Turn around");
                        self.ground_speed += self.deceleration;
                        if self.ground_speed >= 0.0 {
                            self.ground_speed = 0.5;
                        }
                    } else if self.ground_speed < self.top_speed {
                        godot_print!("Accelerate right");
                        self.ground_speed += self.acceleration;
                        self.ground_speed = self.ground_speed.min(self.top_speed);
                    }

                    self.set_state(if self.ground_speed >= self.top_speed {
                        State::FullMotion
                    } else {
                        State::StartMotion
                    });
                    self.set_flip_h(false);
                }
            }

            // Optional fix: use friction always when control lock is active
            // Friction
            if !input.is_action_pressed(c"left".into()) && !input.is_action_pressed(c"right".into())
            {
                godot_print!("Apply friction");

                self.ground_speed -=
                    self.ground_speed.abs().min(self.friction) * self.ground_speed.signum();
            }
            if self.ground_speed == 0.0 {
                self.set_state(State::Idle);
            }
            // Jump Check
            if input.is_action_just_pressed(c"jump".into()) && self.can_jump() {
                godot_print!("Jump");
                let (sin, cos) = self.ground_angle.sin_cos();
                velocity.x -= self.jump_force * sin;
                velocity.y -= self.jump_force * cos;
                godot_print!("{velocity}");
                self.set_grounded(false);
                self.set_state(State::AirBall);
                self.set_velocity(velocity);
                godot_print!("Update position");
                let mut position = self.global_position();
                position += velocity;
                self.set_global_position(position);
                return;
            }

            // Wall checking
            if self.should_activate_wall_sensors() {
                if self.ground_speed > 0.0 {
                    if let Some(result) = self.wall_right_sensor_check() {
                        if result.distance <= 0.0 {
                            self.grounded_right_wall_collision(result.distance);
                        }
                    }
                } else if let Some(result) = self.wall_left_sensor_check() {
                    if result.distance <= 0.0 {
                        self.grounded_left_wall_collision(result.distance);
                    }
                }
            }
            // Adjust velocity based on slope
            godot_print!("Slope velocity adjustment");
            let (sin, cos) = self.ground_angle.sin_cos();
            velocity.x = self.ground_speed * cos;
            velocity.y = -self.ground_speed * sin;
            self.set_velocity(velocity);

            // Update position
            godot_print!("Update position");
            let mut position = self.global_position();
            position += velocity;
            self.set_global_position(position);

            // Floor checking
            if let Some(result) = self.ground_check() {
                if self.should_snap_to_floor(result) {
                    godot_print!("Snap to floor");
                    self.snap_to_floor(result.distance);
                    self.set_ground_angle(result.normal.plane_angle())
                } else {
                    godot_print!("Detach from floor");
                    self.set_grounded(false);
                }
            } else {
                godot_print!("Detach from floor");
                self.set_grounded(false);
            }
            if self.control_lock_timer <= 0 {
                // Slipping check
                if self.velocity().x.abs() < 2.5 && self.is_slipping() {
                    self.control_lock_timer = 30;
                    // Fall check
                    if self.is_falling() {
                        // Detach
                        godot_print!("Fall");
                        self.set_grounded(false);
                        self.ground_speed = 0.0;
                    } else {
                        godot_print!("Slip");
                        // Slipe / slide down
                        self.ground_speed += if self.ground_angle < PI { -0.5 } else { 0.5 }
                    }
                }
            } else {
                self.control_lock_timer -= 1;
            }
        } else {
            // Airborne
            godot_print!("Airborne");
            let mut velocity = self.velocity();
            if velocity.x.abs() > 0.0 {
                self.set_flip_h(velocity.x < 0.0)
            }
            if !self.state.is_ball() {
                self.set_state(if self.ground_speed.abs() >= self.top_speed {
                    State::FullMotion
                } else if self.ground_speed.abs() >= 0.0 {
                    State::StartMotion
                } else {
                    State::Idle
                });
            }
            // Air Acceleration
            if input.is_action_pressed(c"left".into()) {
                godot_print!("Accelerate left");
                velocity.x -= self.air_acceleration;
            }
            if input.is_action_pressed(c"right".into()) {
                godot_print!("Accelerate right");
                velocity.x += self.air_acceleration;
            }
            velocity.x = velocity.x.clamp(-self.top_speed, self.top_speed);

            // Air Drag
            if velocity.y < 0.0 && velocity.y > -4.0 {
                godot_print!("Apply drag");
                velocity.x -= (velocity.x / 0.125) / 256.0;
            }

            // Move player
            godot_print!("Update position");
            let mut position = self.global_position();
            position += velocity;
            self.set_global_position(position);

            // Gravity
            godot_print!("Apply gravity");
            velocity.y += self.gravity;
            // Top y speed
            velocity.y = velocity.y.min(16.0);

            // Rotate ground angle to 0
            let mut rotation = self.base().get_rotation();
            let delta = f32::to_radians(2.8125);
            if rotation > 0.0 {
                rotation -= delta;
                rotation = rotation.max(0.0);
            } else if rotation < 0.0 {
                rotation += delta;
                rotation = rotation.min(0.0);
            }
            self.base_mut().set_rotation(rotation);

            // Air collision checks
            let motion_direction = MotionDirection::from_velocity(velocity);

            // Wall check
            match motion_direction {
                MotionDirection::Up | MotionDirection::Down => {
                    if let Some(result) = self.wall_right_sensor_check() {
                        if result.distance < 0.0 {
                            self.airborne_right_wall_collision(result.distance);
                        }
                    }
                    if let Some(result) = self.wall_left_sensor_check() {
                        if result.distance < 0.0 {
                            self.airborne_left_wall_collision(result.distance);
                        }
                    }
                }
                MotionDirection::Right => {
                    if let Some(result) = self.wall_right_sensor_check() {
                        if result.distance < 0.0 {
                            self.airborne_right_wall_collision(result.distance);
                        }
                    }
                }
                MotionDirection::Left => {
                    if let Some(result) = self.wall_left_sensor_check() {
                        if result.distance < 0.0 {
                            self.airborne_left_wall_collision(result.distance);
                        }
                    }
                }
            }

            // Ceiling check
            match motion_direction {
                MotionDirection::Right | MotionDirection::Left | MotionDirection::Up => {
                    if let Some(result) = self.ceiling_check() {
                        if result.distance < 0.0 {
                            // Ceiling collision
                            godot_print!("ceiling collision");
                            let mut position = self.global_position();
                            position.y -= result.distance;
                            self.set_global_position(position);

                            if self.should_land_on_ceiling() {
                                self.set_ground_angle(result.normal.plane_angle());
                                self.set_grounded(true);
                                self.land_on_ceiling();
                                godot_print!("land on ceiling");
                            } else {
                                velocity.y = 0.0;

                                godot_print!("bump on ceiling");
                            }
                        }
                    }
                }
                MotionDirection::Down => {}
            }
            // Floor check
            match motion_direction {
                MotionDirection::Right | MotionDirection::Left | MotionDirection::Down => {
                    if let Some(result) = self.ground_check() {
                        if self.is_landed(result) {
                            godot_print!("floor collision");
                            // Floor collision
                            let mut position = self.global_position();
                            position.y += result.distance;
                            self.set_global_position(position);

                            self.set_ground_angle(result.normal.plane_angle());
                            self.set_grounded(true);

                            self.land_on_floor();
                        }
                    }
                }
                MotionDirection::Up => {}
            }
            self.set_velocity(velocity);
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
    fn set_ground_angle(&mut self, mut value: f32) {
        self.ground_angle = value;
        if value < PI {
            value += TAU;
        }
        self.base_mut().set_rotation(TAU - value);
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
    fn set_flip_h(&mut self, value: bool) {
        if let Some(sprites) = &mut self.sprites {
            sprites.set_flip_h(value);
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
    fn land_on_ceiling(&mut self) {
        let velocity = self.velocity();
        self.ground_speed = velocity.y * -self.ground_angle.sin().signum();
    }
    fn land_on_floor(&mut self) {
        #[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
        enum FloorKind {
            #[default]
            Flat,
            Slope,
            Steep,
        }
        impl FloorKind {
            #[allow(clippy::just_underscores_and_digits)]
            fn from_floor_angle(angle: f32) -> Self {
                let _23 = f32::to_radians(23.0);
                let _45 = f32::to_radians(45.0);
                let _316 = f32::to_radians(316.0);
                let _339 = f32::to_radians(339.0);
                let _360 = f32::to_radians(360.0);
                if (0.0..=_23).contains(&angle) || (_339..=_360).contains(&angle) {
                    Self::Flat
                } else if (_23..=_45).contains(&angle) || (_316..=_339).contains(&angle) {
                    Self::Slope
                } else {
                    Self::Steep
                }
            }
        }
        let floor_kind = FloorKind::from_floor_angle(self.ground_angle);
        let velocity = self.velocity();
        let motion_direction = MotionDirection::from_velocity(velocity);
        self.ground_speed = match floor_kind {
            FloorKind::Flat => velocity.x,
            FloorKind::Slope => {
                if motion_direction.is_horizontal() {
                    velocity.x
                } else {
                    velocity.y * 0.5 * -self.ground_angle.sin().signum()
                }
            }
            FloorKind::Steep => {
                if motion_direction.is_horizontal() {
                    velocity.x
                } else {
                    velocity.y * -self.ground_angle.sin().signum()
                }
            }
        }
    }

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
    fn is_uphill(&self) -> bool {
        self.ground_speed.signum() == self.ground_angle.sin().signum()
    }

    #[allow(clippy::just_underscores_and_digits)]
    fn is_slipping(&self) -> bool {
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
    fn is_falling(&self) -> bool {
        let _69 = f32::to_radians(69.0);
        let _293 = f32::to_radians(293.0);
        // Sonic 3
        (_69..=_293).contains(&self.ground_angle)
    }

    fn current_slope_factor(&self) -> f32 {
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
}
