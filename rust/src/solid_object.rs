pub(crate) mod sloped_solid_object;

use godot::{
    engine::{Area2D, CollisionShape2D, IArea2D, RectangleShape2D},
    obj::WithBaseField,
    prelude::*,
};

use crate::{
    character::Character,
    sensor::{Sensor, TILE_SIZE},
};
#[derive(GodotClass)]
#[class(init, base=Area2D)]
pub struct SolidObject {
    #[export(range = (0.0,100.0, 1.0))]
    #[var(get, set = set_width_radius)]
    #[init(default = 8.0)]
    width_radius: f32,
    #[export(range = (0.0,100.0, 1.0))]
    #[var(get, set = set_height_radius)]
    #[init(default = 8.0)]
    height_radius: f32,
    #[export]
    top_solid_only: bool,
    #[export]
    collision_shape: Option<Gd<CollisionShape2D>>,
    #[export]
    is_pushable: bool,
    #[export]
    #[init(default = 2.5)]
    pushable_block_gravity: f32,
    #[export]
    floor_sensor: Option<Gd<Sensor>>,
    #[export]
    #[var]
    velocity: Vector2,

    position_last_frame: Vector2,
    #[export]
    state: State,
    pixels_moved: f32,
    base: Base<Area2D>,
}
#[derive(GodotConvert, Var, Export, Default, Debug, PartialEq, Eq, Clone, Copy)]
#[godot(via = GString)]
enum State {
    #[default]
    Normal,
    Falling,
}

impl State {
    /// Returns `true` if the state is [`Falling`].
    ///
    /// [`Falling`]: State::Falling
    #[must_use]
    fn is_falling(&self) -> bool {
        matches!(self, Self::Falling)
    }
}

#[godot_api]
impl IArea2D for SolidObject {
    fn physics_process(&mut self, _delta: f64) {
        if let Some(mut player) = self
            .base()
            .get_tree()
            .and_then(|mut tree| tree.get_first_node_in_group(c"player".into()))
            .and_then(|player| player.try_cast::<Character>().ok())
        {
            let position = self.collision_shape_global_position();
            let radius = Vector2::new(self.width_radius, self.height_radius);
            if self.top_solid_only {
                if solid_object_collision_top_solid(&mut player, position, radius) {
                    player
                        .bind_mut()
                        .set_stand_on_object(self.base().clone().cast::<Self>());
                }
            } else if let Some(collision_direction) =
                solid_object_collision(&mut player, position, radius)
            {
                if collision_direction == CollisionDirection::Up {
                    player
                        .bind_mut()
                        .set_stand_on_object(self.base().clone().cast::<Self>())
                }
                if self.is_pushable {
                    let mut position = self.base().get_global_position();
                    let mut player_position = player.get_global_position();
                    match collision_direction {
                        CollisionDirection::Left => {
                            player_position.x += 0.5;
                            player.bind_mut().velocity.x = 0.0;
                            player.bind_mut().set_ground_speed(0.25);
                            player.set_global_position(player_position);

                            position.x += 0.5;
                            self.base_mut().set_global_position(position);
                            if let Some(floor_sensor) = &mut self.floor_sensor {
                                if let Some(result) = floor_sensor.bind_mut().sense() {
                                    if result.distance > 0.0 {
                                        self.pixels_moved = 4.0;
                                        position.x += 4.0;

                                        self.state = State::Falling;
                                    }
                                } else {
                                    self.pixels_moved = 4.0;
                                    position.x += 4.0;
                                    self.state = State::Falling;
                                }
                            }
                        }
                        CollisionDirection::Right => {
                            player_position.x -= 0.5;
                            player.bind_mut().velocity.x = 0.0;
                            player.bind_mut().set_ground_speed(-0.25);
                            player.set_global_position(player_position);

                            position.x -= 0.5;
                            self.base_mut().set_global_position(position);
                            if let Some(floor_sensor) = &mut self.floor_sensor {
                                if let Some(result) = floor_sensor.bind_mut().sense() {
                                    if result.distance > 0.0 {
                                        position.x -= 4.0;
                                        self.pixels_moved = -4.0;
                                        self.state = State::Falling;
                                    }
                                } else {
                                    position.x -= 4.0;
                                    self.pixels_moved = -4.0;
                                }
                            }
                        }
                        _ => {}
                    }
                    self.base_mut().set_global_position(position);
                }
            }
        }
        let mut position = self.base().get_global_position();

        if let Some(floor_sensor) = &mut self.floor_sensor {
            if let Some(result) = floor_sensor.bind_mut().sense() {
                if result.distance > 0.0 {
                    self.state = State::Falling;
                } else {
                    if self.state.is_falling() {
                        position.y -= result.distance;
                    }
                    self.state = State::Normal;
                }
            } else {
                self.state = State::Falling;
            }
        }

        if self.state.is_falling() {
            if self.pixels_moved.abs() > 16.0 || self.pixels_moved == 0.0 {
                // Start to actually fall
                self.pixels_moved = 0.0;
                position.y += self.pushable_block_gravity;
                self.base_mut().set_global_position(position);
                if let Some(floor_sensor) = &mut self.floor_sensor {
                    if let Some(result) = floor_sensor.bind_mut().sense() {
                        if result.distance <= 0.0 {
                            self.state = State::Normal;
                            position.y -= result.distance + self.pushable_block_gravity;
                        }
                    }
                }
                self.base_mut().set_global_position(position);
            } else {
                // Move to the push direction at speed = 4 until you moved 16 px
                let moving_right = self.pixels_moved.signum() > 0.0;
                self.pixels_moved += if moving_right { 4.0 } else { -4.0 };
                position.x += if moving_right { 4.0 } else { -4.0 };
                self.base_mut().set_global_position(position);
            }
        }
        let position = self.base().get_global_position();
        self.velocity = position - self.position_last_frame;
        self.position_last_frame = position;
    }
}

#[godot_api]
impl SolidObject {
    #[func]
    fn set_width_radius(&mut self, value: f32) {
        self.width_radius = value;
        self.update_shape();
    }
    #[func]
    fn set_height_radius(&mut self, value: f32) {
        self.height_radius = value;
        self.update_shape();
    }
}

impl SolidObject {
    pub fn collision_shape_global_position(&self) -> Vector2 {
        if let Some(collision_shape) = &self.collision_shape {
            return collision_shape.get_global_position();
        }
        self.base().get_global_position()
    }
    fn update_shape(&mut self) {
        if let Some(mut rect) = self
            .collision_shape
            .as_deref()
            .and_then(|cs| cs.get_shape())
            .and_then(|shape| shape.try_cast::<RectangleShape2D>().ok())
        {
            rect.set_size(Vector2::new(
                self.width_radius * 2.0,
                self.height_radius * 2.0,
            ))
        }
    }
}
/// Returns true when player needs to snap on top
fn solid_object_collision_top_solid(
    player: &mut Gd<Character>,
    position: Vector2,
    radius: Vector2,
) -> bool {
    let velocity = player.bind().get_velocity();
    if velocity.y < 0.0 {
        return false;
    }
    // Check overlap
    let combined_x_radius = radius.x + player.bind().get_push_radius() + 1.0;
    let player_height_radius = player.bind().get_height_radius();

    let mut player_position = player.get_global_position();

    let combined_x_diameter = combined_x_radius * 2.0;
    let left_difference = (player_position.x - position.x) + combined_x_radius;
    // the Player is too far to the left to be touching
    // the Player is too far to the right to be touching
    if left_difference < 0.0 || left_difference > combined_x_diameter {
        return false;
    }

    let object_surface_y = position.y - radius.y;
    let player_bottom_y = player_position.y + player_height_radius + 4.0;
    if object_surface_y > player_bottom_y {
        // Platform is too low
        return false;
    }
    let y_distance = object_surface_y - player_bottom_y;
    if !(-16.0..0.0).contains(&y_distance) {
        // Platform is too low
        return false;
    }
    player_position.y += y_distance + 3.0;
    player.set_global_position(player_position);

    player.bind_mut().set_grounded(false);
    player.bind_mut().set_ground_angle(0.0);
    player.bind_mut().set_ground_speed(velocity.x);
    godot_print!(
        "upwards land on top solid collision dy : {}",
        -y_distance - 1.0
    );
    true
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CollisionDirection {
    Left,
    Right,
    Up,
    Down,
}

/// Returns true when player needs to snap on top
fn solid_object_collision(
    player: &mut Gd<Character>,
    position: Vector2,
    radius: Vector2,
) -> Option<CollisionDirection> {
    // Check overlap
    let combined_x_radius = radius.x + player.bind().get_push_radius() + 1.0;
    let player_height_radius = player.bind().get_height_radius();
    let combined_y_radius = radius.y + player_height_radius;

    let mut player_position = player.get_global_position();

    let combined_x_diameter = combined_x_radius * 2.0;
    let left_difference = (player_position.x - position.x) + combined_x_radius;
    // the Player is too far to the left to be touching
    // the Player is too far to the right to be touching
    if left_difference < 0.0 || left_difference > combined_x_diameter {
        return None;
    }

    let top_difference = (player_position.y - position.y) + 4.0 + combined_y_radius;
    let combined_y_diameter = combined_y_radius * 2.0;

    // the Player is too far above to be touching
    // the Player is too far down to be touching
    if top_difference < 0.0 || top_difference > combined_y_diameter {
        return None;
    }

    // Find which side on the object you are nearest and calculate overlap distance
    let x_distance = if player_position.x > position.x {
        // Right side: x distance is < 0.0
        left_difference - combined_x_diameter
    } else {
        // Left side: x distance is > 0.0
        left_difference
    };

    let y_distance = if player_position.y > position.y {
        // Bottom side:  y distance is < 0.0
        top_difference - combined_y_diameter - 4.0
    } else {
        // Top side: y distance is > 0.0
        top_difference
    };
    let mut velocity = player.bind().get_velocity();
    let is_grounded = player.bind().get_is_grounded();
    // Is the distance closer on horizontal side or vertical side
    if x_distance.abs() > y_distance.abs() || y_distance.abs() <= 4.0 {
        // Collide vertically
        if y_distance < 0.0 {
            // Downwards collision
            if velocity.y.is_zero_approx() && is_grounded {
                // Die from getting crushed
                player.bind_mut().die();
                Some(CollisionDirection::Down)
            } else if velocity.y < 0.0 && y_distance < 0.0 {
                player_position.y -= y_distance;
                player.set_global_position(player_position);
                velocity.y = 0.0;
                player.bind_mut().set_velocity(velocity);
                godot_print!("downwards solid collision dy : {}", -y_distance);
                Some(CollisionDirection::Down)
            } else {
                None
            }
        } else if y_distance < TILE_SIZE {
            // Land on object
            let y_distance = y_distance - 4.0;
            // Distance to object right edge
            let x_comparison = position.x - player_position.x + radius.x;
            // if the Player is too far to the right
            // if the Player is too far to the left
            if x_comparison < 0.0 || x_comparison >= combined_x_diameter {
                return None;
            }
            // Going up and not landing
            if velocity.y < 0.0 {
                return None;
            }

            player_position.y -= y_distance;
            player_position.y -= 1.0;
            player.set_global_position(player_position);
            player.bind_mut().set_grounded(false);
            player.bind_mut().set_ground_angle(0.0);
            player.bind_mut().set_ground_speed(velocity.x);

            godot_print!("upwards land on solid collision dy : {}", -y_distance - 1.0);
            Some(CollisionDirection::Up)
        } else {
            None
        }
    } else {
        // Collide horizontally
        if x_distance == 0.0 {
            // Do not reset speeds
        } else if (x_distance > 0.0 && velocity.x > 0.0) || (x_distance < 0.0 && velocity.x < 0.0) {
            // Reset speeds only when moving left if on right side or
            //when moving right if on left side

            player.bind_mut().set_ground_speed(0.0);
            velocity.x = 0.0;
            player.bind_mut().set_velocity(velocity);
        }
        godot_print!("horizontal solid collision dx : {}", -x_distance);

        player_position.x -= x_distance;
        player.set_global_position(player_position);

        if x_distance < 0.0 {
            Some(CollisionDirection::Right)
        } else {
            Some(CollisionDirection::Left)
        }
    }
}
