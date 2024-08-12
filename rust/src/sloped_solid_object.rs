use core::f32;

use godot::{
    engine::{Area2D, CollisionPolygon2D, IArea2D},
    obj::WithBaseField,
    prelude::*,
};

use crate::{character::Character, sensor::TILE_SIZE};
#[derive(GodotClass)]
#[class(init, base=Area2D)]
pub struct SlopedSolidObject {
    #[export]
    top_solid_only: bool,
    #[export]
    collision_polygon: Option<Gd<CollisionPolygon2D>>,
    #[var]
    velocity: Vector2,
    position_last_frame: Vector2,
    base: Base<Area2D>,
}

#[derive(Debug, Clone)]
pub struct RaycastResult {
    position: Vector2,
    normal: Vector2,
    collider: Gd<Object>,
    rid: Rid,
    shape: i32,
}
impl GodotConvert for RaycastResult {
    type Via = Dictionary;
}

impl FromGodot for RaycastResult {
    fn try_from_godot(dict: Self::Via) -> Result<Self, ConvertError> {
        let position = dict
            .get("position")
            .ok_or(ConvertError::default())?
            .try_to()?;
        let normal = dict
            .get("normal")
            .ok_or(ConvertError::default())?
            .try_to()?;
        let collider = dict
            .get("collider")
            .ok_or(ConvertError::default())?
            .try_to()?;
        let rid = dict.get("rid").ok_or(ConvertError::default())?.try_to()?;
        let shape = dict.get("shape").ok_or(ConvertError::default())?.try_to()?;

        Ok(Self {
            position,
            normal,
            collider,
            rid,
            shape,
        })
    }
}
#[godot_api]
impl IArea2D for SlopedSolidObject {
    fn physics_process(&mut self, _delta: f64) {
        if let Some(player) = self
            .base()
            .get_tree()
            .and_then(|mut tree| tree.get_first_node_in_group(c"player".into()))
            .and_then(|player| player.try_cast::<Character>().ok())
        {
            self.sloped_solid_object_collision(player);
        }
        self.base_mut().queue_redraw();
        let position = self.base().get_global_position();
        self.velocity = position - self.position_last_frame;
        self.position_last_frame = position;
    }
    fn draw(&mut self) {
        let center = self.global_center();
        let height_radius = self.height_radius();
        let width_radius = self.width_radius();
        let global_position = self.base().get_global_position();
        self.base_mut()
            .draw_circle(center - global_position, 2.0, Color::PURPLE);
        self.base_mut().draw_line(
            center + Vector2::new(0.0, height_radius) - global_position,
            center - Vector2::new(0.0, height_radius) - global_position,
            Color::BLUE,
        );
        self.base_mut().draw_line(
            center + Vector2::new(width_radius, 0.0) - global_position,
            center - Vector2::new(width_radius, 0.0) - global_position,
            Color::BLUE,
        );
        if let Some(player) = self
            .base()
            .get_tree()
            .and_then(|mut tree| tree.get_first_node_in_group(c"player".into()))
            .and_then(|player| player.try_cast::<Character>().ok())
        {
            let player_position = player.get_global_position();
            let position = self.global_center();
            let (top, bottom) = self.current_top_bottom(player_position);
            let current_height = if player_position.y > position.y {
                bottom
            } else {
                top
            };
            self.base_mut().draw_line(
                player_position - global_position,
                Vector2::new(player_position.x, current_height) - global_position,
                Color::RED,
            );
        }
    }
}

impl SlopedSolidObject {
    pub(super) fn sloped_solid_object_collision(&mut self, player: Gd<Character>) {
        let player_position = player.get_global_position();
        let (top, bottom) = self.current_top_bottom(player_position);

        let mut position = self.global_center();
        position.y = (bottom + top) * 0.5;
        let radius = Vector2::new(self.width_radius(), (bottom - top) * 0.5);

        if self.top_solid_only {
            self.solid_object_collision_top_solid(player, position, radius)
        } else {
            self.solid_object_collision(player, position, radius)
        }
    }

    pub fn global_center(&self) -> Vector2 {
        if let Some(collision_polygon) = &self.collision_polygon {
            return collision_polygon.get_global_position() + self.polygon_center();
        }
        self.base().get_global_position() + self.polygon_center()
    }

    pub fn current_top_bottom(&self, player_position: Vector2) -> (f32, f32) {
        // Is player on the top or bottom
        let Some(collision_polygon) = &self.collision_polygon else {
            let position = self.base().get_global_position();
            return (position.y, position.y);
        };
        let position = collision_polygon.get_global_position();
        let global_center = self.global_center();
        let width_radius = self.width_radius();
        let polygon = collision_polygon.get_polygon().to_vec();
        let x = player_position.x;

        // Handle edges
        if x < global_center.x - width_radius {
            // Calculate leftmost vertex
            if let Some((i, v)) = polygon
                .iter()
                .enumerate()
                .map(|(i, v)| (i, *v + position))
                .min_by(|(_, a), (_, b)| a.x.total_cmp(&b.x))
            {
                let prev = {
                    if i == 0 {
                        polygon[polygon.len() - 1]
                    } else {
                        polygon[(i - 1) % polygon.len()]
                    }
                } + position;
                let next = polygon[(i + 1) % polygon.len()] + position;
                let closest = if (prev.x - v.x).abs() < (next.x - v.x).abs() {
                    prev
                } else {
                    next
                };
                if (closest.x - v.x).abs() < 0.1 {
                    let bottom = v.y.max(closest.y);
                    let top = v.y.min(closest.y);
                    return (top, bottom);
                } else {
                    return (v.y, v.y);
                }
            } else {
                return (position.y, position.y);
            }
        }
        if x > global_center.x + width_radius {
            // Calculate rightmost vertex
            if let Some((i, v)) = polygon
                .iter()
                .enumerate()
                .map(|(i, v)| (i, *v + position))
                .max_by(|(_, a), (_, b)| a.x.total_cmp(&b.x))
            {
                let prev = {
                    if i == 0 {
                        polygon[polygon.len() - 1]
                    } else {
                        polygon[(i - 1) % polygon.len()]
                    }
                } + position;
                let next = polygon[(i + 1) % polygon.len()] + position;
                let closest = if (prev.x - v.x).abs() < (next.x - v.x).abs() {
                    prev
                } else {
                    next
                };
                if (closest.x - v.x).abs() < 0.1 {
                    let bottom = v.y.max(closest.y);
                    let top = v.y.min(closest.y);
                    return (top, bottom);
                } else {
                    return (v.y, v.y);
                }
            } else {
                return (position.y, position.y);
            }
        }

        // Calculate the lowest y on an edge containing player position x
        // Calculate the highest y on an edge containing player position x
        let mut min = f32::MAX;
        let mut max = f32::MIN;
        for i in 0..polygon.len() {
            let next_index = (i + 1) % polygon.len();
            // Edge positions in global space
            let mut point = polygon[i] + position;
            let mut next_point = polygon[next_index] + position;
            if next_point.x < point.x {
                std::mem::swap(&mut point, &mut next_point);
            }
            if point.x <= x && x <= next_point.x {
                // Edge contains the player x position
                // Avoid dividing by zero when line is vertical
                if (next_point.x - point.x) > 0.1 {
                    // Calculate line equation
                    let m = (next_point.y - point.y) / (next_point.x - point.x);
                    let c = next_point.y - m * next_point.x;
                    // Calculate height at x;
                    let y = m * x + c;
                    min = min.min(y);
                    max = max.max(y);
                } else {
                    min = min.min(point.y).min(next_point.y);
                    max = max.max(point.y).max(next_point.y);
                }
            }
        }
        godot_print!("top{min} bottom{max}");
        (min, max)
    }

    fn polygon_center(&self) -> Vector2 {
        let (min_x, max_x) = self.min_max_x();
        let (min_y, max_y) = self.min_max_y();
        Vector2::new((min_x + max_x) * 0.5, (min_y + max_y) * 0.5)
    }
    fn min_max_y(&self) -> (f32, f32) {
        if let Some(collision_polygon) = &self.collision_polygon {
            let polygon = collision_polygon.get_polygon().to_vec();
            let min = polygon
                .iter()
                .map(|a| a.y)
                .min_by(|a, b| a.total_cmp(b))
                .unwrap_or_default();

            let max = polygon
                .iter()
                .map(|a| a.y)
                .max_by(|a, b| a.total_cmp(b))
                .unwrap_or_default();

            return (min, max);
        }
        (0.0, 0.0)
    }

    fn min_max_x(&self) -> (f32, f32) {
        if let Some(collision_polygon) = &self.collision_polygon {
            let polygon = collision_polygon.get_polygon().to_vec();
            let min = polygon
                .iter()
                .map(|a| a.x)
                .min_by(|a, b| a.total_cmp(b))
                .unwrap_or_default();

            let max = polygon
                .iter()
                .map(|a| a.x)
                .max_by(|a, b| a.total_cmp(b))
                .unwrap_or_default();

            return (min, max);
        }
        (0.0, 0.0)
    }

    pub fn width_radius(&self) -> f32 {
        let (min, max) = self.min_max_x();
        (max - min) * 0.5
    }
    pub fn height_radius(&self) -> f32 {
        let (min, max) = self.min_max_y();
        (max - min) * 0.5
    }

    pub(super) fn solid_object_collision_top_solid(
        &mut self,
        mut player: Gd<Character>,
        position: Vector2,
        radius: Vector2,
    ) {
        let velocity = player.bind().get_velocity();
        if self.top_solid_only && velocity.y < 0.0 {
            return;
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
            return;
        }

        let object_surface_y = position.y - radius.y;
        let player_bottom_y = player_position.y + player_height_radius + 4.0;
        if object_surface_y > player_bottom_y {
            // Platform is too low
            return;
        }
        let y_distance = object_surface_y - player_bottom_y;
        if !(-16.0..0.0).contains(&y_distance) {
            // Platform is too low
            return;
        }
        player_position.y += y_distance + 3.0;
        player.set_global_position(player_position);

        player.bind_mut().set_grounded(false);
        player.bind_mut().set_ground_angle(0.0);
        player.bind_mut().set_ground_speed(velocity.x);
        player
            .bind_mut()
            .set_stand_on_sloped_object(self.base().clone().cast::<SlopedSolidObject>());
        godot_print!(
            "upwards land on top solid collision dy : {}",
            -y_distance - 1.0
        );
    }

    pub(super) fn solid_object_collision(
        &mut self,
        mut player: Gd<Character>,
        position: Vector2,
        radius: Vector2,
    ) {
        let mut velocity = player.bind().get_velocity();
        if self.top_solid_only && velocity.y < 0.0 {
            return;
        }
        // Check overlap
        let combined_x_radius = radius.x + player.bind().get_push_radius() + 1.0;
        let combined_y_radius = radius.y + player.bind().get_height_radius();

        let mut player_position = player.get_global_position();

        let combined_x_diameter = combined_x_radius * 2.0;
        let left_difference = (player_position.x - position.x) + combined_x_radius;

        // the Player is too far to the left to be touching
        // the Player is too far to the right to be touching
        if left_difference < 0.0 || left_difference > combined_x_diameter {
            return;
        }

        let top_difference = (player_position.y - position.y) + 4.0 + combined_y_radius;
        let combined_y_diameter = combined_y_radius * 2.0;

        // the Player is too far above to be touching
        // the Player is too far down to be touching

        if top_difference < 0.0 || top_difference > combined_y_diameter {
            return;
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
        let is_grounded = player.bind().get_is_grounded();
        // Is the distance closer on horizontal side or vertical side
        if x_distance.abs() > y_distance.abs() || y_distance.abs() <= 4.0 {
            // Collide vertically
            if y_distance < 0.0 {
                // Downwards collision
                if velocity.y >= 0.0 && is_grounded {
                    // Die from getting crushed
                    player.bind_mut().die();
                } else if velocity.y < 0.0 && y_distance < 0.0 {
                    player_position.y -= y_distance;
                    player.set_global_position(player_position);
                    velocity.y = 0.0;
                    player.bind_mut().set_velocity(velocity);
                    godot_print!("downwards solid collision dy : {}", -y_distance);
                }
            } else if y_distance < TILE_SIZE {
                // Land on object
                let y_distance = y_distance - 4.0;
                // Distance to object right edge
                let x_comparison = position.x - player_position.x + radius.x;
                // if the Player is too far to the right
                // if the Player is too far to the left
                if x_comparison < 0.0 || x_comparison >= combined_x_diameter {
                    return;
                }
                // Going up and not landing
                if velocity.y < 0.0 {
                    return;
                }

                player_position.y -= y_distance;
                player_position.y -= 1.0;
                player.set_global_position(player_position);
                player.bind_mut().set_grounded(false);
                // TODO: what if i wanted to calculate the angle?
                player.bind_mut().set_ground_angle(0.0);
                player.bind_mut().set_ground_speed(velocity.x);
                player
                    .bind_mut()
                    .set_stand_on_sloped_object(self.base().clone().cast::<SlopedSolidObject>());
                godot_print!("upwards land on solid collision dy : {}", -y_distance - 1.0);
            }
        } else {
            // Collide horizontally
            if x_distance == 0.0 {
                // Do not reset speeds
            } else if (x_distance > 0.0 && velocity.x > 0.0)
                || (x_distance < 0.0 && velocity.x < 0.0)
            {
                // Reset speeds only when moving left if on right side or
                //when moving right if on left side

                player.bind_mut().set_ground_speed(0.0);
                velocity.x = 0.0;
                player.bind_mut().set_velocity(velocity);
            }
            godot_print!("horizontal solid collision dx : {}", -x_distance);

            player_position.x -= x_distance;
            player.set_global_position(player_position);
        }
    }
}
