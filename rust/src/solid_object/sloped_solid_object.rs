use core::f32;

use godot::{
    engine::{Area2D, CollisionPolygon2D, IArea2D},
    obj::WithBaseField,
    prelude::*,
};

use crate::{character::Character, sensor::TILE_SIZE};

use super::{solid_object_collision, solid_object_collision_top_solid, Collision};
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
    pub(super) fn sloped_solid_object_collision(&mut self, mut player: Gd<Character>) {
        let player_position = player.get_global_position();
        let (top, bottom) = self.current_top_bottom(player_position);

        let mut position = self.global_center();
        position.y = (bottom + top) * 0.5;
        let radius = Vector2::new(self.width_radius(), (bottom - top) * 0.5);

        if self.top_solid_only {
            if solid_object_collision_top_solid(&mut player, position, radius) {
                player
                    .bind_mut()
                    .set_stand_on_sloped_object(self.base().clone().cast::<Self>());
            }
        } else if let Some(collision_direction) =
            solid_object_collision(&mut player, position, radius)
        {
            if collision_direction == Collision::Up {
                player
                    .bind_mut()
                    .set_stand_on_sloped_object(self.base().clone().cast::<Self>())
            }
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
}
