use std::f32::consts::FRAC_PI_2;

use godot::{
    engine::{
        CollisionObject2D, CollisionShape2D, Engine, PhysicsRayQueryParameters2D, ThemeDb,
        TileData, TileMap,
    },
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
const TILE_SIZE: f32 = 16.0;

impl Direction {
    fn target_direction(&self) -> Vector2 {
        match *self {
            Direction::Left => Vector2::LEFT * TILE_SIZE,
            Direction::Up => Vector2::UP * TILE_SIZE,
            Direction::Right => Vector2::RIGHT * TILE_SIZE,
            Direction::Down => Vector2::DOWN * TILE_SIZE,
        }
    }
    fn target_direction_normalized(&self) -> Vector2 {
        match *self {
            Direction::Left => Vector2::LEFT,
            Direction::Up => Vector2::UP,
            Direction::Right => Vector2::RIGHT,
            Direction::Down => Vector2::DOWN,
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
#[derive(GodotClass)]
#[class(tool,init, base=Node2D)]
pub struct Sensor {
    #[export]
    #[var(get, set = set_direction)]
    direction: Direction,
    #[export]
    update_in_editor: bool,
    #[export]
    display_debug_label: bool,

    last_result: Option<DetectionResult>,
    #[export(flags_2d_physics)]
    #[var(get, set)]
    collision_mask: u32,
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
    SidesAndBottom,
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
    fn to_godot(&self) -> Self::Via {
        dict! {"distance":self.distance,"angle":self.angle,"solidity":self.solidity,"snap":self.snap}
    }
}
impl FromGodot for DetectionResult {
    fn try_from_godot(dict: Self::Via) -> Result<Self, ConvertError> {
        let distance = dict
            .get("distance")
            .ok_or(ConvertError::default())?
            .try_to()?;
        let angle = dict.get("angle").ok_or(ConvertError::default())?.try_to()?;
        let solidity = dict
            .get("solidity")
            .ok_or(ConvertError::default())?
            .try_to()?;
        let snap = dict.get("snap").ok_or(ConvertError::default())?.try_to()?;
        Ok(Self {
            distance,
            angle,
            solidity,
            snap,
        })
    }
}

impl DetectionResult {
    fn new(distance: f32, angle: f32, solidity: Solidity, snap: bool) -> Self {
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
            self._sense();
        }
        self.base_mut().queue_redraw();
    }
    fn draw(&mut self) {
        self.draw_ray();
    }
}

#[godot_api]
impl Sensor {
    #[func]
    pub fn set_direction(&mut self, value: Direction) {
        self.direction = value;
    }

    #[func]
    pub fn sense(&mut self) -> Variant {
        match self._sense() {
            Some(result) => result.into_godot().to_variant(),
            None => Variant::nil(),
        }
    }
}

fn is_polygon_full(array: PackedVector2Array) -> bool {
    let full_polygon = PackedVector2Array::from(&[
        Vector2::new(-8.0, -8.0),
        Vector2::new(8.0, -8.0),
        Vector2::new(8.0, 8.0),
        Vector2::new(-8.0, 8.0),
    ]);
    array == full_polygon
}

impl Sensor {
    fn global_position(&self) -> Vector2 {
        self.base().get_global_position()
    }

    fn draw_ray(&mut self) {
        let debug_color = self.debug_color;
        if let Some(result) = self.last_result {
            let collision_point = self.direction.target_direction_normalized() * result.distance;
            self.base_mut()
                .draw_line_ex(Vector2::ZERO, collision_point, debug_color)
                .width(1.0)
                .done();

            self.base_mut()
                .draw_circle(collision_point, 2.0, Color::RED);

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
                self.base_mut()
                    .draw_string(font, collision_point, text.into_godot());
            }
        } else {
            let target_direction = self.direction.target_direction();

            self.base_mut()
                .draw_line_ex(Vector2::ZERO, target_direction, debug_color)
                .width(1.0)
                .done();
        }
    }

    fn _sense(&mut self) -> Option<DetectionResult> {
        let position = self.global_position();
        let target_direction = self.direction.target_direction();
        let snapped_position = self.snapped_position();
        let mut result;
        if let Some(r) = self.raycast(snapped_position, snapped_position + target_direction) {
            let distance = self.get_distance(position, r.position);
            result = Some(self.get_detection(&r));
            if distance < 0.0 {
                let tile_above_position = snapped_position - target_direction;
                if let Some(r) =
                    self.raycast(tile_above_position, tile_above_position + target_direction)
                {
                    let distance = self.get_distance(position, r.position);

                    if distance < TILE_SIZE {
                        result = Some(self.get_detection(&r));
                    }
                }
            }
        } else {
            let double_target_direction = 2.0 * target_direction;
            result = self
                .raycast(position, position + double_target_direction)
                .map(|r| self.get_detection(&r));
        };
        self.last_result = result;
        result
    }

    fn raycast(&self, from: Vector2, to: Vector2) -> Option<RaycastResult> {
        let mut space_state = self.base().get_world_2d()?.get_direct_space_state()?;
        let mask = self.collision_mask;

        let mut query = PhysicsRayQueryParameters2D::create_ex(from, to)
            .collision_mask(mask)
            .done()?;
        query.set_collide_with_areas(false);
        query.set_hit_from_inside(true);
        let result = space_state.intersect_ray(query);
        if !result.is_empty() {
            RaycastResult::try_from_godot(result).ok()
        } else {
            None
        }
    }

    fn update_debug(&mut self) {
        let Some(result) = self.last_result else {
            return;
        };
        godot_print!("DRAW");
        let collision_point = self.direction.target_direction_normalized() * result.distance;
        self.base_mut()
            .draw_circle(collision_point, 50.0, Color::RED);

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
            self.base_mut()
                .draw_string(font, collision_point, text.into_godot());
        }
    }
    fn get_detection(&self, result: &RaycastResult) -> DetectionResult {
        let collision_point = result.position;
        let position = self.global_position();
        let distance = self.get_distance(position, collision_point);
        let normal = result.normal;
        let snapped = if let Some((layer, tile_data)) = self.get_collided_tile_data(result) {
            let polygon_full = if tile_data.get_collision_polygons_count(layer) > 0 {
                let collision_data = tile_data.get_collision_polygon_points(layer, 0);
                is_polygon_full(collision_data)
            } else {
                false
            };
            polygon_full || tile_data.get_custom_data("snap".into_godot()).booleanize()
        } else {
            false
        };
        let angle = normal.plane_angle();
        let solidity = if let Some(shape) = self.get_collider_shape(result) {
            if shape.is_one_way_collision_enabled() {
                Solidity::Top
            } else {
                Solidity::Fully
            }
        } else {
            Solidity::Fully
        };
        DetectionResult::new(
            distance,
            angle,
            solidity,
            normal == Vector2::ZERO || snapped,
        )
    }

    fn get_collider_shape(&self, result: &RaycastResult) -> Option<Gd<CollisionShape2D>> {
        let target = result
            .collider
            .clone()
            .try_cast::<CollisionObject2D>()
            .ok()?;
        let shape_id = result.shape;
        let owner_id = target.shape_find_owner(shape_id);
        target
            .shape_owner_get_owner(owner_id)?
            .try_cast::<CollisionShape2D>()
            .ok()
    }
    fn get_collided_tile_data(
        &self,
        raycast_result: &RaycastResult,
    ) -> Option<(i32, Gd<TileData>)> {
        let collider_rid = raycast_result.rid;
        let mut tilemap = raycast_result.collider.clone().try_cast::<TileMap>().ok()?;
        let map_coords = tilemap.get_coords_for_body_rid(collider_rid);
        let layer = tilemap.get_layer_for_body_rid(collider_rid);
        let tile_data = tilemap.get_cell_tile_data(layer, map_coords)?;
        Some((layer, tile_data))
    }

    fn get_distance(&self, position: Vector2, collision_point: Vector2) -> f32 {
        match self.direction {
            Direction::Up => position.y - collision_point.y,
            Direction::Down => collision_point.y - position.y,
            Direction::Left => position.x - collision_point.x,
            Direction::Right => collision_point.x - position.x,
        }
    }

    fn snapped_position(&self) -> Vector2 {
        let mut position = self.global_position();
        match self.direction {
            Direction::Up => position.y += TILE_SIZE - (position.y % TILE_SIZE),
            Direction::Down => position.y -= position.y % TILE_SIZE,
            Direction::Left => position.x += TILE_SIZE - (position.x % TILE_SIZE),
            Direction::Right => position.x -= position.x % TILE_SIZE,
        }

        position
    }
}
