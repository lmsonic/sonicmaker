use std::f32::consts::FRAC_PI_2;

use godot::{
    classes::{Engine, PhysicsRayQueryParameters2D, ThemeDb, TileData, TileMap},
    prelude::*,
};

use crate::vec3_ext::Vector2Ext;

#[derive(GodotConvert, Var, Export, Default, Debug, PartialEq, Eq, Clone, Copy)]
#[godot(via = GString)]
pub enum Direction {
    Up,
    #[default]
    Down,
    Left,
    Right,
}
pub const TILE_SIZE: f32 = 16.0;

impl Direction {
    fn target_direction(self) -> Vector2 {
        match self {
            Self::Left => Vector2::LEFT * TILE_SIZE,
            Self::Up => Vector2::UP * TILE_SIZE,
            Self::Right => Vector2::RIGHT * TILE_SIZE,
            Self::Down => Vector2::DOWN * TILE_SIZE,
        }
    }
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
            .ok_or_else(ConvertError::default)?
            .try_to()?;
        let normal = dict
            .get("normal")
            .ok_or_else(ConvertError::default)?
            .try_to()?;
        let collider = dict
            .get("collider")
            .ok_or_else(ConvertError::default)?
            .try_to()?;
        let rid = dict
            .get("rid")
            .ok_or_else(ConvertError::default)?
            .try_to()?;
        let shape = dict
            .get("shape")
            .ok_or_else(ConvertError::default)?
            .try_to()?;

        Ok(Self {
            position,
            normal,
            collider,
            rid,
            shape,
        })
    }
}

#[derive(GodotClass, Debug)]
#[class(tool,init, base=Node2D)]
pub struct Sensor {
    /// Direction where the sensor points
    #[export]
    #[var(get, set = set_direction)]
    direction: Direction,
    /// Set to true if you want the sensor collision updates to happen in the editor
    #[export]
    update_in_editor: bool,
    /// Set to true if you want the sensor shape to be shown
    #[export]
    display_debug_shape: bool,
    /// Set to true if you want the sensor data (angle, distance, and if it collides) shown
    #[export]
    display_debug_label: bool,
    last_collision_point: Option<Vector2>,
    last_result: Option<DetectionResult>,
    /// What physics layers sensor should collide with
    #[export(flags_2d_physics)]
    #[var(get, set)]
    #[init(default = 1)]
    collision_mask: u32,
    /// Sensor debug shape color
    #[export]
    #[init(default = Color::from_rgba(0.0, 0.6, 0.7, 0.42))]
    debug_color: Color,
    base: Base<Node2D>,
}

#[derive(GodotConvert, Var, Export, Default, Debug, PartialEq, Eq, Clone, Copy)]
#[godot(via = i32)]
pub enum Solidity {
    #[default]
    Fully,
    Top,
}

#[derive(Debug, Clone, Copy)]
pub struct DetectionResult {
    pub distance: f32,
    pub angle: f32,
    pub solidity: Solidity,
    pub snap: bool,
}

impl GodotConvert for DetectionResult {
    type Via = Dictionary;
}
impl ToGodot for DetectionResult {
    type ToVia<'v> = Dictionary
    where
        Self: 'v;

    fn to_godot(&self) -> Self::Via {
        dict! {"distance":self.distance,"angle":self.angle,"solidity":self.solidity,"snap":self.snap}
    }
}
impl FromGodot for DetectionResult {
    fn try_from_godot(dict: Self::Via) -> Result<Self, ConvertError> {
        let distance = dict
            .get("distance")
            .ok_or_else(ConvertError::default)?
            .try_to()?;
        let angle = dict
            .get("angle")
            .ok_or_else(ConvertError::default)?
            .try_to()?;
        let solidity = dict
            .get("solidity")
            .ok_or_else(ConvertError::default)?
            .try_to()?;
        let snap = dict
            .get("snap")
            .ok_or_else(ConvertError::default)?
            .try_to()?;
        Ok(Self {
            distance,
            angle,
            solidity,
            snap,
        })
    }
}

impl DetectionResult {
    const fn new(distance: f32, angle: f32, solidity: Solidity, snap: bool) -> Self {
        Self {
            distance,
            angle,
            solidity,
            snap,
        }
    }
}
#[godot_api]
impl INode2D for Sensor {
    fn physics_process(&mut self, _delta: f64) {
        if Engine::singleton().is_editor_hint() && self.update_in_editor {
            self.sense();
        }
        if self.display_debug_shape {
            self.base_mut().queue_redraw();
        }
    }
    fn draw(&mut self) {
        if !self.display_debug_shape {
            return;
        }
        self.draw_ray();
        self.last_result = None;
        self.last_collision_point = None;
    }
}

#[godot_api]
impl Sensor {
    #[func]
    pub fn set_direction(&mut self, value: Direction) {
        self.direction = value;
    }

    /// Converts the sensed DetectionResult to a GodotDictionary for access from GDScript
    #[func]
    pub fn sense_godot(&mut self) -> Variant {
        self.sense()
            .map_or_else(Variant::nil, |result| result.to_variant())
    }
}

fn is_polygon_full(array: &PackedVector2Array) -> bool {
    let full_polygon = PackedVector2Array::from(&[
        Vector2::new(-8.0, -8.0),
        Vector2::new(8.0, -8.0),
        Vector2::new(8.0, 8.0),
        Vector2::new(-8.0, 8.0),
    ]);
    *array == full_polygon
}

impl Sensor {
    fn global_position(&self) -> Vector2 {
        self.base().get_global_position()
    }
    /// Shows debug shape and debug data if activated
    fn draw_ray(&mut self) {
        let debug_color = self.debug_color;
        if let Some(point) = self.last_collision_point {
            let point = point - self.global_position();
            self.base_mut()
                .draw_line_ex(Vector2::ZERO, point, debug_color)
                .width(1.0)
                .done();

            self.base_mut().draw_circle(point, 2.0, Color::RED);
        }
        if let Some(result) = self.last_result {
            if !self.display_debug_label {
                return;
            }
            let angle = if result.snap {
                (result.angle / FRAC_PI_2).round() * FRAC_PI_2
            } else {
                result.angle
            };

            let text = format!(
                "{:.0}px \n{:.0}° {}",
                result.distance,
                angle.to_degrees(),
                if result.snap { "snap" } else { "" }
            );

            if let Some(font) = ThemeDb::singleton()
                .get_project_theme()
                .and_then(|theme| theme.get_default_font())
            {
                self.base_mut().draw_string(&font, Vector2::ZERO, &text);
            }
        } else {
            let target_direction = self.direction.target_direction();

            self.base_mut()
                .draw_line_ex(Vector2::ZERO, target_direction, debug_color)
                .width(1.0)
                .done();
        }
    }
    /// From: <https://info.sonicretro.org/SPG:Solid_Tiles#Sensor_Regression_.26_Extension>
    pub fn sense(&mut self) -> Option<DetectionResult> {
        let target_direction = self.direction.target_direction();
        let snapped_position = self.snapped_position();
        let mut result;
        if let Some(r) = self.raycast(snapped_position, snapped_position + target_direction) {
            let distance = self.get_distance(r.position);
            result = Some(self.get_detection(&r));
            if distance <= 0.0 {
                // Regression
                let tile_above_position = snapped_position - target_direction;
                if let Some(r) =
                    self.raycast(tile_above_position, tile_above_position + target_direction)
                {
                    let distance = self.get_distance(r.position);

                    if distance < TILE_SIZE {
                        result = Some(self.get_detection(&r));
                    }
                }
            }
        } else {
            // Extension
            let tile_below_position = snapped_position + target_direction;

            result = self
                .raycast(tile_below_position, tile_below_position + target_direction)
                .map(|r| self.get_detection(&r));
        };
        self.last_result = result;
        result
    }

    /// Boilerplate function to cast a raycast
    fn raycast(&self, from: Vector2, to: Vector2) -> Option<RaycastResult> {
        let mut space_state = self.base().get_world_2d()?.get_direct_space_state()?;
        let mask = self.collision_mask;

        let mut query = PhysicsRayQueryParameters2D::create_ex(from, to)
            .collision_mask(mask)
            .done()?;
        query.set_collide_with_areas(false);
        query.set_hit_from_inside(true);
        let result = space_state.intersect_ray(&query);
        if result.is_empty() {
            None
        } else {
            RaycastResult::try_from_godot(result).ok()
        }
    }

    /// Uses the `RaycastResult` information to produce a `DetectionResult`
    fn get_detection(&mut self, result: &RaycastResult) -> DetectionResult {
        let collision_point = result.position;
        self.last_collision_point = Some(collision_point);
        let distance = self.get_distance(collision_point);
        let normal = result.normal;
        let (solidity, snapped) = if let Some((layer, tile_data)) = get_collided_tile_data(result) {
            let polygon_full = if tile_data.get_collision_polygons_count(layer) > 0 {
                let collision_data = tile_data.get_collision_polygon_points(layer, 0);
                is_polygon_full(&collision_data)
            } else {
                false
            };
            // Checking for flagged tiles: https://info.sonicretro.org/SPG:Solid_Tiles#Flagged_Tiles
            let snapped = polygon_full || tile_data.get_custom_data("snap").booleanize();
            let solidity = if tile_data.get_collision_polygons_count(layer) > 0
                && tile_data.is_collision_polygon_one_way(layer, 0)
            {
                Solidity::Top
            } else {
                Solidity::Fully
            };
            (solidity, snapped)
        } else {
            (Solidity::Fully, false)
        };
        let angle = normal.plane_angle();

        DetectionResult::new(
            distance,
            angle,
            solidity,
            normal == Vector2::ZERO || snapped,
        )
    }
    /// Absolute distance from current global position to the collision point
    fn get_distance(&self, collision_point: Vector2) -> f32 {
        let position = self.global_position();
        match self.direction {
            Direction::Up => position.y - collision_point.y,
            Direction::Down => collision_point.y - position.y,
            Direction::Left => position.x - collision_point.x,
            Direction::Right => collision_point.x - position.x,
        }
    }

    /// Returns the current global position snapped to `TILE_SIZE`
    fn snapped_position(&self) -> Vector2 {
        let mut position = self.global_position();
        match self.direction {
            Direction::Up => position.y = (position.y / TILE_SIZE).ceil() * TILE_SIZE,
            Direction::Down => position.y = (position.y / TILE_SIZE).floor() * TILE_SIZE,
            Direction::Left => position.x = (position.x / TILE_SIZE).ceil() * TILE_SIZE,
            Direction::Right => position.x = (position.x / TILE_SIZE).floor() * TILE_SIZE,
        }

        position
    }
}

/// If the `RaycastResult` has collided with a physics body attached to a `TileMap` , it will return the `TileData` for it
fn get_collided_tile_data(raycast_result: &RaycastResult) -> Option<(i32, Gd<TileData>)> {
    let collider_rid = raycast_result.rid;
    let mut tilemap = raycast_result.collider.clone().try_cast::<TileMap>().ok()?;
    let map_coords = tilemap.get_coords_for_body_rid(collider_rid);
    let layer = tilemap.get_layer_for_body_rid(collider_rid);
    let tile_data = tilemap.get_cell_tile_data(layer, map_coords)?;
    Some((layer, tile_data))
}
