use godot::{
    engine::{
        CollisionObject2D, CollisionShape2D, Engine, IRayCast2D, RayCast2D, ThemeDb, TileData,
        TileMap,
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
    fn get_target_direction(&self) -> Vector2 {
        match *self {
            Direction::Left => Vector2::LEFT * TILE_SIZE,
            Direction::Up => Vector2::UP * TILE_SIZE,
            Direction::Right => Vector2::RIGHT * TILE_SIZE,
            Direction::Down => Vector2::DOWN * TILE_SIZE,
        }
    }
}
#[derive(GodotClass)]
#[class(init, base=RayCast2D)]
pub struct Sensor {
    #[export]
    #[var(get, set = set_direction)]
    direction: Direction,
    #[export]
    enable_in_editor: bool,
    #[export]
    show_debug_label: bool,
    last_result: Option<DetectionResult>,
    base: Base<RayCast2D>,
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
impl IRayCast2D for Sensor {
    fn physics_process(&mut self, _delta: f64) {
        if Engine::singleton().is_editor_hint() && !self.enable_in_editor {
            return;
        }
        self._detect_solid();
    }
    fn draw(&mut self) {
        if Engine::singleton().is_editor_hint() && !self.enable_in_editor {
            return;
        }
        if self.show_debug_label {
            self.update_debug_label();
        }
    }
}

#[godot_api]
impl Sensor {
    #[func]
    pub fn set_direction(&mut self, value: Direction) {
        self.direction = value;
        self.reset_target_position();
    }

    #[func]
    pub fn detect_solid(&mut self) -> Variant {
        match self._detect_solid() {
            Some(result) => result.into_godot().to_variant(),
            None => Variant::nil(),
        }
    }
}

impl Sensor {
    fn update_debug_label(&mut self) {
        if self.last_result.is_some() {
            let collision_point = self.base().get_collision_point();
            let position = self.base().get_global_position();
            self.base_mut()
                .draw_circle(collision_point - position, 2.0, Color::RED);
        }

        let text: GString = match self.last_result {
            Some(result) => {
                let angle = result.angle;
                format!("{:.0} \n{:.0}°", result.distance, angle.to_degrees(),).into()
            }

            None => "".into(),
        };
        if let Some(font) = ThemeDb::singleton()
            .get_project_theme()
            .and_then(|theme| theme.get_default_font())
        {
            self.base_mut()
                .draw_string(font, Vector2::new(0.0, 0.0), text);
        }
    }
    fn _detect_solid(&mut self) -> Option<DetectionResult> {
        // Reset positions
        let original_position = self.base().get_global_position();
        let original_target = self.base().get_target_position();
        self.snap_position();

        self.base_mut().force_raycast_update();

        let result = if self.base().is_colliding() {
            let mut detection = self.get_detection(original_position);
            if detection.distance <= 0.0 {
                // Regression, hit a solid wall

                let snapped_position = self.base().get_global_position();

                let tile_above_position = snapped_position - self.direction.get_target_direction();
                self.base_mut().set_global_position(tile_above_position);
                self.base_mut().force_raycast_update();

                detection = self.get_detection(original_position);
            }
            Some(detection)
        } else {
            // Extension
            // Checking extending to tile below

            let new_position = original_target * 2.0;
            self.base_mut().set_target_position(new_position);
            self.base_mut().force_raycast_update();
            if self.base().is_colliding() {
                Some(self.get_detection(original_position))
            } else {
                None
            }
        };
        self.base_mut().set_target_position(original_target);
        self.base_mut().set_global_position(original_position);
        self.last_result = result;

        result
    }
    fn get_detection(&self, original_position: Vector2) -> DetectionResult {
        let collision_point = self.base().get_collision_point();
        let distance = self.get_distance(original_position, collision_point);
        let normal = self.base().get_collision_normal();
        let snapped = if let Some(tile_data) = self.get_collided_tile_data() {
            tile_data.get_custom_data("snap".into_godot()).booleanize()
        } else {
            false
        };
        let angle = normal.plane_angle();
        let solidity = if let Some(shape) = self.get_collider_shape() {
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
    fn get_collider_shape(&self) -> Option<Gd<CollisionShape2D>> {
        let target = self
            .base()
            .get_collider()?
            .try_cast::<CollisionObject2D>()
            .ok()?;
        let shape_id = self.base().get_collider_shape();
        let owner_id = target.shape_find_owner(shape_id);
        target
            .shape_owner_get_owner(owner_id)?
            .try_cast::<CollisionShape2D>()
            .ok()
    }
    fn get_collided_tile_data(&self) -> Option<Gd<TileData>> {
        let world_coords = self.base().get_collision_point();
        let tilemap = self.base().get_collider()?.try_cast::<TileMap>().ok()?;
        let local_coords = tilemap.to_local(world_coords);
        let map_coords = tilemap.local_to_map(local_coords);
        tilemap.get_cell_tile_data(0, map_coords)
    }

    fn get_distance(&self, position: Vector2, collision_point: Vector2) -> f32 {
        match self.direction {
            Direction::Up => position.y - collision_point.y,
            Direction::Down => collision_point.y - position.y,
            Direction::Left => position.x - collision_point.x,
            Direction::Right => collision_point.x - position.x,
        }
    }

    fn snap_position(&mut self) {
        let mut position = self.base().get_global_position();
        match self.direction {
            Direction::Up => position.y += TILE_SIZE - (position.y % TILE_SIZE),
            Direction::Down => position.y -= position.y % TILE_SIZE,
            Direction::Left => position.x += TILE_SIZE - (position.x % TILE_SIZE),
            Direction::Right => position.x -= position.x % TILE_SIZE,
        }

        self.base_mut().set_global_position(position);
    }
    fn reset_target_position(&mut self) {
        let target_direction = self.direction.get_target_direction();
        self.base_mut().set_target_position(target_direction);
    }
}
