use godot::{classes::ThemeDb, prelude::*};

use crate::character::{
    godot_api::{SolidObjectKind, State},
    Character,
};

use super::SuperPeeloutState;

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
                    &font,
                    Vector2::new(10.0, -30.0),
                    &format!("{angle:.0}°"),
                );
            }
        }
    }
    fn physics_process(&mut self, delta: f64) {
        if self.debug_draw {
            self.base_mut().queue_redraw();
        }

        // changes delta to assume 60 FPS  if fix_delta is true
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
    /// From: <https://info.sonicretro.org/SPG:Solid_Objects#Standing_On_Solid_Objects>
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
    /// From: <https://info.sonicretro.org/SPG:Ring_Loss#Invulnerability>
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

    pub(super) fn update_position(&mut self, delta: f32) {
        godot_print!("Update position");
        let mut position = self.global_position();
        position += self.velocity * delta;
        self.set_global_position(position);
    }

    /// From <https://info.sonicretro.org/SPG:Animations#Animation_Rules>
    pub(super) fn update_animation(&mut self) {
        match self.state {
            State::Idle | State::StartMotion | State::FullMotion => {
                let speed = if self.is_grounded {
                    self.ground_speed.abs()
                } else {
                    self.velocity.x.abs()
                };
                if speed >= self.top_speed {
                    self.set_state(State::FullMotion);
                } else if speed > 0.1 {
                    self.set_state(State::StartMotion);
                } else {
                    self.set_state(State::Idle);
                }
            }
            State::RollingBall => {
                if self.ground_speed.abs() > 6.0 {
                    self.play_animation("rolling_fast");
                } else {
                    self.play_animation("rolling");
                }
            }
            State::Pushing => {
                let input = Input::singleton();
                let horizontal_input = i32::from(input.is_action_pressed("right"))
                    - i32::from(input.is_action_pressed("left"));
                if horizontal_input == 0
                    || horizontal_input > 0 && self.facing_left()
                    || horizontal_input < 0 && !self.facing_left()
                {
                    self.set_state(State::Idle);
                }
            }
            State::SuperPeelOut => {
                if self.super_peel_out_state == SuperPeeloutState::NotCharged
                    && self.ground_speed.abs() <= 6.0
                {
                    self.set_state(State::Idle);
                    self.update_animation();
                }
            }
            _ => {}
        }
    }
}
