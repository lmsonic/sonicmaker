use real_consts::PI;

use crate::character::{godot_api::State, utils::Mode};

use super::Character;
use godot::prelude::*;

impl Character {
    pub(super) fn grounded(&mut self, delta: f32) {
        self.check_unrolling();

        // Grounded
        let input = Input::singleton();

        godot_print!("Grounded");

        self.apply_slope_factor(delta);

        self.handle_crouch(&input);

        if self.handle_jump(&input) {
            self.update_position(delta);
            return;
        }
        if !self.state.is_crouching() {
            self.ground_accelerate(&input, delta);
        }

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

    fn handle_crouch(&mut self, input: &Gd<Input>) {
        if input.is_action_pressed(c"roll".into()) && self.ground_speed.abs() <= 1.0 {
            self.ground_speed = 0.0;
            self.set_state(State::Crouch);
        } else if self.state.is_crouching() && !input.is_action_pressed(c"roll".into()) {
            self.set_state(State::Idle);
        }
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
            self.has_jumped = true;
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
        let top_speed = if self.state.is_rolling() {
            self.roll_top_speed
        } else {
            self.top_speed
        };
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
                } else if self.ground_speed > -self.top_speed {
                    godot_print!("Accelerate left");
                    self.ground_speed -= self.acceleration * delta;
                    // Cap velocity
                    self.ground_speed = self.ground_speed.max(-top_speed);
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
                } else if self.ground_speed < top_speed {
                    godot_print!("Accelerate right");
                    self.ground_speed += self.acceleration * delta;
                    self.ground_speed = self.ground_speed.min(top_speed);
                }

                self.set_flip_h(false);
            }
            // Cap roll velocity
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
}
