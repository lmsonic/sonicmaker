use crate::character::utils::MotionDirection;

use super::{godot_api::State, Character, DropDashState, MidAirAction};
use godot::prelude::*;

impl Character {
    pub(super) fn airborne(&mut self, delta: f32) {
        // Airborne
        let input = Input::singleton();

        godot_print!("Airborne");

        self.handle_mid_air_action(&input);

        // Disable input when hurt
        let can_input = !self.state.is_hurt();
        if can_input {
            self.handle_variable_jump(&input);

            self.air_accelerate(&input, delta);

            self.air_drag(delta);
        }

        self.tick_spring_bounce_animation();
        self.update_animation();

        self.update_position(delta);

        self.apply_gravity(delta);

        self.rotate_to_zero();

        // Air collision checks
        self.check_walls_air();
        self.check_ceiling_air();
        self.check_floor_air();
    }

    fn handle_mid_air_action(&mut self, input: &Gd<Input>) {
        match self.mid_air_action {
            MidAirAction::DropDash => {
                if self.state == State::JumpBall {
                    let is_jump_pressed = input.is_action_pressed(c"jump".into());
                    if !is_jump_pressed {
                        self.has_released_jump = true;
                    }
                    match self.drop_dash_state {
                        DropDashState::NotCharged => {
                            if self.has_released_jump && is_jump_pressed {
                                self.drop_dash_state = DropDashState::Charging { timer: 20 };
                            }
                        }
                        DropDashState::Charging { ref mut timer } => {
                            if is_jump_pressed {
                                *timer -= 1;
                                if *timer <= 0 {
                                    self.drop_dash_state = DropDashState::Charged;
                                }
                            } else {
                                self.drop_dash_state = DropDashState::NotCharged;
                            }
                        }
                        DropDashState::Charged => {}
                    }
                }
            }
            MidAirAction::InstaShield => {
                if self.insta_shield_timer > 0 {
                    self.insta_shield_timer -= 1;
                    if self.insta_shield_timer <= 0 {
                        // Reset hitbox
                        self.update_shapes();
                    }
                }
                if self.state == State::JumpBall {
                    let is_jump_pressed = input.is_action_just_pressed(c"jump".into());
                    if is_jump_pressed {
                        self.insta_shield_timer = 14;
                        // Set bigger attacking hitbox
                        self.set_hitbox_size(Vector2::new(49.0, 49.0));
                    }
                }
            }
            MidAirAction::Flying => {}
            MidAirAction::Gliding => {}
            MidAirAction::None => {}
        }
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
        if self.has_jumped && !input.is_action_pressed(c"jump".into()) && self.velocity.y < -4.0 {
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
                        self.has_jumped = false;
                        self.land_on_floor();

                        self.land();
                    }
                }
            }
            MotionDirection::Up => {}
        }
    }

    pub(super) fn land(&mut self) {
        match self.state {
            State::JumpBall | State::SpringBounce | State::Hurt => {
                self.set_state(State::Idle);
                self.update_animation();
            }
            _ => {}
        }

        if self.drop_dash_state == DropDashState::Charged {
            self.drop_dash();
        }
        self.drop_dash_state = DropDashState::NotCharged;
    }

    pub(super) fn drop_dash(&mut self) {
        let facing_left = self.get_flip_h();

        let is_moving_forwards = facing_left && self.velocity.x < 0.0
            || (!facing_left && self.velocity.x > 0.0)
            || self.velocity.x == 0.0;
        let direction = if facing_left { -1.0 } else { 1.0 };

        if is_moving_forwards {
            self.ground_speed = self.ground_speed / 4.0 + self.drop_dash_speed * direction;
        } else if self.ground_angle == 0.0 {
            self.ground_speed = self.drop_dash_speed * direction;
        } else {
            self.ground_speed = self.ground_speed / 2.0 + self.drop_dash_speed * direction;
        }
        self.ground_speed = self
            .ground_speed
            .clamp(-self.drop_dash_max_speed, self.drop_dash_max_speed);

        self.set_state(State::RollingBall);
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
        if self.state == State::RollingBall || self.state == State::JumpBall {
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

    fn air_accelerate(&mut self, input: &Gd<Input>, delta: f32) {
        if input.is_action_pressed(c"left".into()) {
            godot_print!("Accelerate left");
            self.velocity.x -= self.air_acceleration * delta;
            self.set_flip_h(true);
            self.velocity.x = self.velocity.x.max(-self.top_speed);
        }
        if input.is_action_pressed(c"right".into()) {
            godot_print!("Accelerate right");
            self.velocity.x += self.air_acceleration * delta;
            self.set_flip_h(false);
            self.velocity.x = self.velocity.x.min(self.top_speed);
        }
    }

    fn air_drag(&mut self, delta: f32) {
        if self.velocity.y < 0.0 && self.velocity.y > -4.0 {
            godot_print!("Apply drag");
            self.velocity.x -= (self.velocity.x.div_euclid(0.125)) / 256.0 * delta;
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
}
