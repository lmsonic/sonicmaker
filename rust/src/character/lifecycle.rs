use godot::{
    engine::{ICharacterBody2D, ThemeDb},
    prelude::*,
};
use real_consts::PI;

use crate::character::{
    setters::State,
    utils::{Mode, MotionDirection},
    Character,
};
#[godot_api]
impl ICharacterBody2D for Character {
    fn draw(&mut self) {
        if self.debug_draw {
            let velocity = self.velocity();
            let rotation = self.base().get_rotation();
            self.base_mut()
                .draw_set_transform_ex(Vector2::ZERO)
                .rotation(-rotation)
                .done();
            self.base_mut()
                .draw_line_ex(Vector2::ZERO, velocity * 20.0, Color::RED)
                .width(5.0)
                .done();

            let text = velocity.angle().to_degrees().to_string().into_godot();
            let angle = self.ground_angle.to_degrees().to_string().into_godot();
            if let Some(font) = ThemeDb::singleton()
                .get_project_theme()
                .and_then(|theme| theme.get_default_font())
            {
                self.base_mut()
                    .draw_string(font, Vector2::new(10.0, -30.0), angle);
            }
        }
    }
    fn physics_process(&mut self, _delta: f64) {
        self.base_mut().queue_redraw();
        if self.is_grounded {
            self.grounded()
        } else {
            self.airborne()
        }
    }
}
impl Character {
    fn airborne(&mut self) {
        // Airborne
        let input = Input::singleton();

        godot_print!("Airborne");

        let mut velocity = self.velocity();

        self.handle_variable_jump(&input, &mut velocity);

        self.air_accelerate(&input, &mut velocity);

        self.air_drag(&mut velocity);

        self.update_animation_air(velocity);

        self.update_position(velocity);

        self.apply_gravity(velocity);

        self.rotate_to_zero();

        // Air collision checks
        self.check_walls_air();
        self.check_ceiling_air();
        self.check_floor_air();
    }
    fn handle_variable_jump(&self, input: &Gd<Input>, velocity: &mut Vector2) {
        if self.state.is_jumping() && !input.is_action_pressed(c"jump".into()) && velocity.y < -4.0
        {
            velocity.y = -4.0;
        }
    }

    fn check_floor_air(&mut self) {
        match self.current_motion_direction() {
            MotionDirection::Right | MotionDirection::Left | MotionDirection::Down => {
                if let Some(result) = self.ground_check() {
                    if self.is_landed(result) {
                        // Floor collision
                        let mut position = self.global_position();
                        position.y += result.distance;
                        self.set_global_position(position);

                        godot_print!("floor collision dy:{}", result.distance);
                        self.set_ground_angle(result);
                        self.set_grounded(true);
                        if !self.state.is_rolling() {
                            self.update_animation();
                        }

                        self.land_on_floor();
                    }
                }
            }
            MotionDirection::Up => {}
        }
    }

    fn check_ceiling_air(&mut self) {
        match self.current_motion_direction() {
            MotionDirection::Right | MotionDirection::Left | MotionDirection::Up => {
                if let Some(result) = self.ceiling_check() {
                    if result.distance < -0.1 {
                        // Ceiling collision
                        let mut position = self.global_position();
                        position.y -= result.distance;
                        self.set_global_position(position);
                        godot_print!("ceiling collision dy:{}", -result.distance);

                        if self.should_land_on_ceiling() {
                            self.set_ground_angle(result);
                            self.set_grounded(true);
                            self.land_on_ceiling();
                            godot_print!("land on ceiling");
                        } else {
                            let velocity = self.velocity();
                            self.set_velocity(Vector2::new(velocity.x, 0.0));
                            godot_print!("bump on ceiling");
                        }
                    }
                }
            }
            MotionDirection::Down => {}
        }
    }

    fn check_walls_air(&mut self) {
        match self.current_motion_direction() {
            MotionDirection::Up | MotionDirection::Down => {
                if let Some(result) = self.airborne_wall_right_sensor_check() {
                    if result.distance <= 0.0 {
                        self.airborne_right_wall_collision(result.distance);
                    }
                }
                if let Some(result) = self.airborne_wall_left_sensor_check() {
                    if result.distance <= 0.0 {
                        self.airborne_left_wall_collision(result.distance);
                    }
                }
            }
            MotionDirection::Right => {
                if let Some(result) = self.airborne_wall_right_sensor_check() {
                    if result.distance <= 0.0 {
                        self.airborne_right_wall_collision(result.distance);
                    }
                }
            }
            MotionDirection::Left => {
                if let Some(result) = self.airborne_wall_left_sensor_check() {
                    if result.distance <= 0.0 {
                        self.airborne_left_wall_collision(result.distance);
                    }
                }
            }
        }
    }

    fn rotate_to_zero(&mut self) {
        // Rotate ground angle to 0
        if !self.state.is_ball() {
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
        } else {
            self.base_mut().set_rotation(0.0);
        }
    }

    fn apply_gravity(&mut self, mut velocity: Vector2) {
        godot_print!("Apply gravity");
        velocity.y += self.gravity;
        // Top y speed
        velocity.y = velocity.y.min(16.0);
        self.set_velocity(velocity);
    }

    fn update_position(&mut self, velocity: Vector2) {
        godot_print!("Update position");
        let mut position = self.global_position();
        position += velocity;
        self.set_global_position(position);
    }

    fn air_accelerate(&mut self, input: &Gd<Input>, velocity: &mut Vector2) {
        if input.is_action_pressed(c"left".into()) {
            godot_print!("Accelerate left");
            velocity.x -= self.air_acceleration;
            self.set_flip_h(true);
        }
        if input.is_action_pressed(c"right".into()) {
            godot_print!("Accelerate right");
            velocity.x += self.air_acceleration;
            self.set_flip_h(false);
        }
        velocity.x = velocity.x.clamp(-self.top_speed, self.top_speed);
    }

    fn update_animation_air(&mut self, velocity: Vector2) {
        if !self.state.is_ball() {
            if velocity.x.abs() >= self.top_speed {
                self.set_state(State::FullMotion)
            } else if velocity.x.abs() > 0.0 {
                self.set_state(State::StartMotion)
            } else {
                self.set_state(State::Idle)
            };
        }
    }
    fn grounded(&mut self) {
        self.check_unrolling();

        self.update_animation();
        // Grounded
        let input = Input::singleton();

        godot_print!("Grounded");

        self.apply_slope_factor();

        if self.handle_jump(&input) {
            return;
        }

        self.ground_accelerate(&input);

        self.apply_friction(&input);

        self.check_walls();

        self.check_rolling(&input);

        let velocity = self.update_velocity();

        self.update_position(velocity);

        self.check_floor();

        // self.handle_slipping();
    }
    fn check_rolling(&mut self, input: &Gd<Input>) {
        if !self.state.is_rolling() && input.is_action_pressed(c"roll".into()) && self.can_roll() {
            godot_print!("Rolling");
            self.set_state(State::RollingBall);
        }
    }
    fn check_unrolling(&mut self) {
        if self.state.is_rolling() && self.ground_speed.abs() < 0.5 {
            godot_print!("Unrolling");
            self.set_state(State::Idle);
        }
    }

    fn handle_slipping(&mut self) {
        if self.control_lock_timer <= 0 {
            // Slipping check
            if self.ground_speed.abs() < 2.5 && self.is_slipping() {
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
    }

    fn check_floor(&mut self) {
        // Floor checking
        if let Some(result) = self.ground_check() {
            if self.should_snap_to_floor(result) {
                self.snap_to_floor(result.distance);
                self.set_ground_angle(result)
            } else {
                godot_print!("Detach from floor: Shouldn't snap");
                self.set_grounded(false);
            }
        } else {
            godot_print!("Detach from floor: No ground detected");
            self.set_grounded(false);
        }
    }

    fn update_velocity(&mut self) -> Vector2 {
        // Adjust velocity based on slope
        godot_print!("Update velocity based on slope");

        let x = self.ground_speed * self.ground_angle.cos();
        let y = -self.ground_speed * self.ground_angle.sin();
        let velocity = Vector2::new(x, y);
        self.set_velocity(velocity);

        velocity
    }

    fn check_walls(&mut self) {
        // Wall checking

        if self.should_activate_wall_sensors() {
            if self.ground_speed > 0.0 {
                if let Some(result) = self.wall_right_sensor_check() {
                    if result.distance < 0.0 {
                        self.grounded_right_wall_collision(result.distance);
                    }
                }
            } else if self.ground_speed < 0.0 {
                if let Some(result) = self.wall_left_sensor_check() {
                    if result.distance < 0.0 {
                        self.grounded_left_wall_collision(result.distance);
                    }
                }
            }
        }
    }

    fn handle_jump(&mut self, input: &Gd<Input>) -> bool {
        // Jump Check
        if input.is_action_just_pressed(c"jump".into()) && self.can_jump() {
            godot_print!("Jump");
            let mut velocity = self.velocity();
            let (sin, cos) = self.ground_angle.sin_cos();
            velocity.x -= self.jump_force * sin;
            velocity.y -= self.jump_force * cos;
            godot_print!("{velocity}");

            self.set_grounded(false);
            self.set_state(State::JumpBall);
            self.set_velocity(velocity);
            self.update_position(velocity);

            return true;
        }
        false
    }

    fn apply_friction(&mut self, input: &Gd<Input>) {
        // Optional fix: use friction always when control lock is active

        // Friction
        let horizontal_input_pressed =
            input.is_action_pressed(c"left".into()) || input.is_action_pressed(c"right".into());
        if self.state.is_rolling() || !horizontal_input_pressed {
            godot_print!("Apply friction");

            self.ground_speed -=
                self.ground_speed.abs().min(self.current_friction()) * self.ground_speed.signum();
        }
    }

    fn ground_accelerate(&mut self, input: &Gd<Input>) {
        if self.control_lock_timer <= 0 {
            let is_rolling = self.state.is_rolling();
            // Ground Acceleration
            if input.is_action_pressed(c"left".into()) {
                if self.ground_speed > 0.0 {
                    // Turn around
                    godot_print!("Turn around");
                    self.ground_speed -= self.current_deceleration();
                    if self.ground_speed <= 0.0 || is_rolling && self.ground_speed.abs() < 0.1484375
                    {
                        self.ground_speed = -0.5;
                    }
                } else if self.ground_speed > -self.top_speed && !is_rolling {
                    godot_print!("Accelerate left");
                    self.ground_speed -= self.acceleration;
                    self.ground_speed = self.ground_speed.max(-self.top_speed);
                }

                self.set_flip_h(true);
            }
            if input.is_action_pressed(c"right".into()) {
                if self.ground_speed < 0.0 {
                    // Turn around
                    godot_print!("Turn around");
                    self.ground_speed += self.current_deceleration();
                    if self.ground_speed >= 0.0 || is_rolling && self.ground_speed.abs() < 0.1484375
                    {
                        self.ground_speed = 0.5;
                    }
                } else if self.ground_speed < self.top_speed && !is_rolling {
                    godot_print!("Accelerate right");
                    self.ground_speed += self.acceleration;
                    self.ground_speed = self.ground_speed.min(self.top_speed);
                }

                self.set_flip_h(false);
            }
            // Cap roll velocity
        }
        if self.state.is_rolling() {
            // Optional fix : use ground speed instead
            let mut velocity = self.velocity();
            velocity.x = velocity.x.clamp(-self.roll_top_speed, self.roll_top_speed);
            self.set_velocity(velocity);
        }
    }

    fn apply_slope_factor(&mut self) {
        // Slow down uphill and speeding up downhill
        if self.current_mode() != Mode::Ceiling {
            let slope_factor = self.current_slope_factor() * self.ground_angle.sin();
            // Forces moving when walking on steep slopes
            let is_moving = self.ground_speed.abs() > 0.0;
            let is_rolling = self.state.is_rolling();
            let is_on_steep = slope_factor >= 0.05078125;
            if is_moving || is_rolling || is_on_steep {
                godot_print!("Applying slope factor {slope_factor}");
                self.ground_speed -= slope_factor;
            }
        }
    }

    fn update_animation(&mut self) {
        if !self.state.is_rolling() {
            if self.ground_speed.abs() >= self.top_speed {
                self.set_state(State::FullMotion);
            } else if self.ground_speed.abs() > 0.0 {
                self.set_state(State::StartMotion);
            } else {
                self.set_state(State::Idle);
            }
        }
    }

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
        };
        self.set_velocity(Vector2::ZERO);
    }
    fn air_drag(&self, velocity: &mut Vector2) {
        if velocity.y < 0.0 && velocity.y > -4.0 {
            godot_print!("Apply drag");
            velocity.x -= (velocity.x / 0.125) / 256.0;
        }
    }
}
