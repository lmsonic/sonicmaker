use godot::{engine::ThemeDb, prelude::*};
use real_consts::PI;

use crate::character::{
    godot_api::State,
    utils::{Mode, MotionDirection},
    Character,
};

use super::godot_api::SolidObjectKind;

// Genesis runs at 60 fps
const FPS: f32 = 60.0;
#[godot_api]
impl INode2D for Character {
    fn draw(&mut self) {
        if self.debug_draw {
            let velocity = self.velocity;
            let rotation = self.base().get_rotation();
            self.base_mut()
                .draw_set_transform_ex(Vector2::ZERO)
                .rotation(-rotation)
                .done();
            self.base_mut()
                .draw_line_ex(Vector2::ZERO, velocity * 20.0, Color::RED)
                .width(2.0)
                .done();

            let angle = self.ground_angle.to_degrees();
            if let Some(font) = ThemeDb::singleton()
                .get_project_theme()
                .and_then(|theme| theme.get_default_font())
            {
                self.base_mut().draw_string(
                    font,
                    Vector2::new(10.0, -30.0),
                    format!("{angle:.0}°").into_godot(),
                );
            }
        }
    }
    fn physics_process(&mut self, delta: f64) {
        self.base_mut().queue_redraw();

        let delta = if self.fix_delta {
            1.0
        } else {
            delta as f32 * FPS
        };

        self.handle_invulnerability();
        self.stand_on_solid_object();
        if self.is_grounded {
            self.grounded(delta);
        } else {
            self.airborne(delta);
        }
    }
}
impl Character {
    fn handle_invulnerability(&mut self) {
        if self.regather_rings_timer > 0 {
            self.regather_rings_timer -= 1;
        }
        if self.invulnerability_timer > 0 {
            self.invulnerability_timer -= 1;
            if self.invulnerability_timer % 4 == 0 {
                if let Some(sprite) = &mut self.sprites {
                    if sprite.is_visible() {
                        sprite.hide();
                    } else {
                        sprite.show();
                    }
                }
            }
        }
    }
    fn stand_on_solid_object(&mut self) {
        let Some(solid_object) = &self.solid_object_to_stand_on else {
            return;
        };
        let mut position = self.global_position();

        let (object_position, obj_width_radius, object_top_position, velocity) = match solid_object
        {
            SolidObjectKind::Simple(object) => {
                let velocity = object.bind().get_velocity();
                let object_position = object.bind().collision_shape_global_position() + velocity;
                let obj_width_radius = object.bind().get_width_radius();
                let obj_height_radius = object.bind().get_height_radius();
                let object_top_position =
                    object_position.y - obj_height_radius - self.height_radius - 1.0;
                (
                    object_position,
                    obj_width_radius,
                    object_top_position,
                    velocity,
                )
            }
            SolidObjectKind::Sloped(object) => {
                let velocity = object.bind().get_velocity();

                let object_position = object.bind().global_center() + velocity;
                let obj_width_radius = object.bind().width_radius();

                let (top, _) = object.bind().current_top_bottom(position);
                let object_top_position = top - self.height_radius - 1.0;
                (
                    object_position,
                    obj_width_radius,
                    object_top_position,
                    velocity,
                )
            }
        };

        position.x += velocity.x;
        position.y = object_top_position;
        self.base_mut().set_global_position(position);
        self.set_grounded(true);
        godot_print!("Stand on solid object at y={object_top_position}");

        // Check if you walked off the edge
        let combined_x_radius = obj_width_radius + self.push_radius + 1.0;
        let x_left_distance = (position.x - object_position.x) + combined_x_radius;
        if x_left_distance <= 0.0 || x_left_distance >= combined_x_radius * 2.0 {
            self.clear_standing_objects();

            self.set_grounded(false);
            godot_print!("walk off solid object");
        }
    }
    fn airborne(&mut self, delta: f32) {
        // Airborne
        let input = Input::singleton();

        godot_print!("Airborne");

        if !self.state.is_hurt() {
            // Disable input when hurt
            self.handle_variable_jump(&input);

            self.air_accelerate(&input, delta);

            self.air_drag(delta);
        }

        self.tick_spring_bounce_animation();
        self.update_animation_air();

        self.update_position(delta);

        self.apply_gravity(delta);

        self.rotate_to_zero();

        // Air collision checks
        self.check_walls_air();
        self.check_ceiling_air();
        self.check_floor_air();
    }

    fn tick_spring_bounce_animation(&mut self) {
        if self.spring_bounce_timer > 0 {
            self.spring_bounce_timer -= 1;
            if self.spring_bounce_timer <= 0 {
                self.set_state(State::Idle);
            }
        }
    }
    fn handle_variable_jump(&mut self, input: &Gd<Input>) {
        if self.state.is_jumping()
            && !input.is_action_pressed(c"jump".into())
            && self.velocity.y < -4.0
        {
            self.velocity.y = -4.0;
        }
    }

    fn check_floor_air(&mut self) {
        match self.current_motion_direction() {
            MotionDirection::Right | MotionDirection::Left | MotionDirection::Down => {
                if let Some(result) = self.ground_check(true) {
                    if self.is_landed(result) {
                        // Floor collision
                        let mut position = self.global_position();
                        position.y += result.distance;
                        self.set_global_position(position);

                        godot_print!("floor collision dy:{}", result.distance);
                        self.set_ground_angle_from_result(result);
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
                if let Some(result) = self.ceiling_check(true) {
                    if result.distance < 0.0 {
                        // Ceiling collision
                        let mut position = self.global_position();
                        position.y -= result.distance;
                        self.set_global_position(position);
                        godot_print!("{}", position);
                        godot_print!("ceiling collision dy:{}", -result.distance);

                        if self.should_land_on_ceiling() {
                            self.set_ground_angle_from_result(result);
                            self.set_grounded(true);
                            self.land_on_ceiling();
                            godot_print!("land on ceiling");
                        } else {
                            self.velocity.y = 0.0;
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
                if let Some(result) = self.wall_right_sensor_check(true) {
                    if result.distance < 0.0 {
                        self.airborne_right_wall_collision(result.distance);
                    }
                }
                if let Some(result) = self.wall_left_sensor_check(true) {
                    if result.distance < 0.0 {
                        self.airborne_left_wall_collision(result.distance);
                    }
                }
            }
            MotionDirection::Right => {
                if let Some(result) = self.wall_right_sensor_check(true) {
                    if result.distance < 0.0 {
                        self.airborne_right_wall_collision(result.distance);
                    }
                }
            }
            MotionDirection::Left => {
                if let Some(result) = self.wall_left_sensor_check(true) {
                    if result.distance < 0.0 {
                        self.airborne_left_wall_collision(result.distance);
                    }
                }
            }
        }
    }

    fn rotate_to_zero(&mut self) {
        // Rotate ground angle to 0
        if self.state.is_ball() {
            self.base_mut().set_rotation(0.0);
        } else {
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
        }
    }

    fn apply_gravity(&mut self, delta: f32) {
        godot_print!("Apply gravity");
        if self.state.is_hurt() {
            self.velocity.y += self.hurt_gravity * delta;
        } else {
            self.velocity.y += self.gravity * delta;
        }
        // Top y speed
        self.velocity.y = self.velocity.y.min(16.0);
    }

    fn update_position(&mut self, delta: f32) {
        godot_print!("Update position");
        let mut position = self.global_position();
        position += self.velocity * delta;
        self.set_global_position(position);
    }

    fn air_accelerate(&mut self, input: &Gd<Input>, delta: f32) {
        if input.is_action_pressed(c"left".into()) {
            godot_print!("Accelerate left");
            self.velocity.x -= self.air_acceleration * delta;
            self.set_flip_h(true);
        }
        if input.is_action_pressed(c"right".into()) {
            godot_print!("Accelerate right");
            self.velocity.x += self.air_acceleration * delta;
            self.set_flip_h(false);
        }
        self.velocity.x = self.velocity.x.clamp(-self.top_speed, self.top_speed);
    }

    fn update_animation_air(&mut self) {
        if !(self.state.is_ball() || self.state.is_hurt() || self.state.is_spring_bouncing()) {
            if self.velocity.x.abs() >= self.top_speed {
                self.set_state(State::FullMotion);
            } else if self.velocity.x.abs() > 0.0 {
                self.set_state(State::StartMotion);
            } else {
                self.set_state(State::Idle);
            };
        }
    }
    fn grounded(&mut self, delta: f32) {
        self.check_unrolling();

        // Grounded
        let input = Input::singleton();

        godot_print!("Grounded");

        self.apply_slope_factor(delta);

        if self.handle_jump(&input) {
            self.update_position(delta);
            return;
        }

        self.ground_accelerate(&input, delta);

        self.apply_friction(&input, delta);

        self.check_walls();

        self.update_animation();

        if self.solid_object_to_stand_on.is_none() {
            self.check_floor();
        }

        self.check_rolling(&input);

        self.update_velocity();

        self.update_position(delta);

        self.handle_slipping();
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
        if let Some(result) = self.ground_check(false) {
            if self.should_snap_to_floor(result) {
                self.snap_to_floor(result.distance);
                self.set_ground_angle_from_result(result);
            } else {
                godot_print!("Detach from floor: Shouldn't snap");
                self.set_grounded(false);
            }
        } else {
            godot_print!("Detach from floor: No ground detected");
            self.set_grounded(false);
        }
    }

    fn update_velocity(&mut self) {
        // Adjust velocity based on slope
        godot_print!("Update velocity based on slope");

        let x = self.ground_speed * self.ground_angle.cos();
        let y = -self.ground_speed * self.ground_angle.sin();
        self.velocity = Vector2::new(x, y);
    }

    fn check_walls(&mut self) {
        // Wall checking

        if self.should_activate_wall_sensors() {
            if self.ground_speed > 0.0 {
                if let Some(result) = self.wall_right_sensor_check(false) {
                    if result.distance < 0.0 {
                        self.grounded_right_wall_collision(result.distance);
                    }
                }
            } else if self.ground_speed < 0.0 {
                if let Some(result) = self.wall_left_sensor_check(false) {
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
            let (sin, cos) = self.ground_angle.sin_cos();
            self.velocity.x -= self.jump_force * sin;
            self.velocity.y -= self.jump_force * cos;
            godot_print!("Jump {}", self.velocity);

            self.set_grounded(false);
            self.set_state(State::JumpBall);
            self.clear_standing_objects();

            return true;
        }
        false
    }

    fn apply_friction(&mut self, input: &Gd<Input>, delta: f32) {
        // Optional fix: use friction always when control lock is active

        // Friction
        let horizontal_input_pressed =
            input.is_action_pressed(c"left".into()) || input.is_action_pressed(c"right".into());
        if self.state.is_rolling() || !horizontal_input_pressed {
            godot_print!("Apply friction");

            self.ground_speed -= self.ground_speed.abs().min(self.current_friction() * delta)
                * self.ground_speed.signum();
        }
    }

    fn ground_accelerate(&mut self, input: &Gd<Input>, delta: f32) {
        if self.control_lock_timer <= 0 {
            let is_rolling = self.state.is_rolling();
            // Ground Acceleration
            let horizontal_input = i32::from(input.is_action_pressed(c"right".into()))
                - i32::from(input.is_action_pressed(c"left".into()));
            if horizontal_input < 0 {
                if self.ground_speed > 0.0 {
                    // Turn around
                    godot_print!("Turn around");
                    self.ground_speed -= self.current_deceleration() * delta;
                    if self.state.is_pushing() {
                        self.set_state(State::Idle);
                    }
                    if self.ground_speed > 4.0 {
                        self.set_state(State::Skidding);
                    }
                    let roll_turn_threshold = (self.roll_deceleration + self.roll_friction) * delta;
                    if self.ground_speed <= 0.0
                        || is_rolling && self.ground_speed.abs() < roll_turn_threshold
                    {
                        self.ground_speed = -0.5;
                    }
                } else if self.ground_speed > -self.top_speed && !is_rolling {
                    godot_print!("Accelerate left");
                    self.ground_speed -= self.acceleration * delta;
                    self.ground_speed = self.ground_speed.max(-self.top_speed);
                }

                self.set_flip_h(true);
            } else if horizontal_input > 0 {
                if self.ground_speed < 0.0 {
                    // Turn around
                    godot_print!("Turn around");
                    self.ground_speed += self.current_deceleration() * delta;
                    if self.state.is_pushing() {
                        self.set_state(State::Idle);
                    }
                    if self.ground_speed < -4.0 {
                        self.set_state(State::Skidding);
                    }
                    let roll_turn_threshold = (self.roll_deceleration + self.roll_friction) * delta;
                    if self.ground_speed >= 0.0
                        || is_rolling && self.ground_speed.abs() < roll_turn_threshold
                    {
                        self.ground_speed = 0.5;
                    }
                } else if self.ground_speed < self.top_speed && !is_rolling {
                    godot_print!("Accelerate right");
                    self.ground_speed += self.acceleration * delta;
                    self.ground_speed = self.ground_speed.min(self.top_speed);
                }

                self.set_flip_h(false);
            }
            // Cap roll velocity
        }
        if self.state.is_rolling() {
            // Optional fix : use ground speed instead
            self.velocity.x = self
                .velocity
                .x
                .clamp(-self.roll_top_speed, self.roll_top_speed);
        }
    }

    fn apply_slope_factor(&mut self, delta: f32) {
        const STEEP_ANGLE: f32 = 0.05078125;
        // Slow down uphill and speeding up downhill
        if self.current_mode() != Mode::Ceiling {
            let slope_factor = self.current_slope_factor() * self.ground_angle.sin();
            // Forces moving when walking on steep slopes
            let is_moving = self.ground_speed.abs() > 0.0;
            let is_rolling = self.state.is_rolling();
            let is_on_steep = slope_factor >= STEEP_ANGLE;
            if is_moving || is_rolling || is_on_steep {
                godot_print!("Applying slope factor {slope_factor}");
                self.ground_speed -= slope_factor * delta;
            }
        }
    }

    fn update_animation(&mut self) {
        if self.state.is_pushing() {
            let input = Input::singleton();
            let horizontal_input = i32::from(input.is_action_pressed(c"right".into()))
                - i32::from(input.is_action_pressed(c"left".into()));
            if horizontal_input == 0
                || horizontal_input > 0 && self.facing_left()
                || horizontal_input < 0 && !self.facing_left()
            {
                self.set_state(State::Idle);
            }
        }

        if !(self.state.is_rolling() || self.state.is_skidding() || self.state.is_pushing()) {
            if self.ground_speed.abs() >= self.top_speed {
                self.set_state(State::FullMotion);
            } else if self.ground_speed.abs() > 0.1 {
                self.set_state(State::StartMotion);
            } else {
                self.set_state(State::Idle);
            }
        }
    }

    fn land_on_ceiling(&mut self) {
        self.ground_speed = self.velocity.y * -self.ground_angle.sin().signum();
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
        if self.state.is_hurt() {
            self.ground_speed = 0.0;
            self.set_velocity(Vector2::ZERO);
            return;
        }

        let floor_kind = FloorKind::from_floor_angle(self.ground_angle);
        let motion_direction = MotionDirection::from_velocity(self.velocity);

        self.ground_speed = match floor_kind {
            FloorKind::Flat => self.velocity.x,
            FloorKind::Slope => {
                if motion_direction.is_horizontal() {
                    self.velocity.x
                } else {
                    self.velocity.y * 0.5 * -self.ground_angle.sin().signum()
                }
            }
            FloorKind::Steep => {
                if motion_direction.is_horizontal() {
                    self.velocity.x
                } else {
                    self.velocity.y * -self.ground_angle.sin().signum()
                }
            }
        };
        self.set_velocity(Vector2::ZERO);
    }
    fn air_drag(&mut self, delta: f32) {
        if self.velocity.y < 0.0 && self.velocity.y > -4.0 {
            godot_print!("Apply drag");
            self.velocity.x -= (self.velocity.x / 0.125) / 256.0 * delta;
        }
    }
}
