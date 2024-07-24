use std::f32::consts::PI;

use godot::{
    engine::{Engine, IRayCast2D, RayCast2D, ThemeDb},
    prelude::*,
};
use real_consts::{FRAC_2_PI, FRAC_PI_2};

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
    fn is_horizontal(&self) -> bool {
        *self == Direction::Left || *self == Direction::Right
    }
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
#[class(tool,init, base=RayCast2D)]
pub struct Sensor {
    #[export]
    #[var(get, set = set_direction)]
    direction: Direction,
    #[export]
    update_in_editor: bool,
    #[export]
    show_debug_label: bool,
    last_result: Option<DetectionResult>,
    base: Base<RayCast2D>,
}

#[derive(Debug, Clone, Copy)]
pub struct DetectionResult {
    pub distance: f32,
    pub normal: Vector2,
}

impl GodotConvert for DetectionResult {
    type Via = Dictionary;
}
impl ToGodot for DetectionResult {
    fn to_godot(&self) -> Self::Via {
        dict! {"distance":self.distance,"normal":self.normal}
    }
}
impl FromGodot for DetectionResult {
    fn try_from_godot(dict: Self::Via) -> Result<Self, ConvertError> {
        let distance = dict
            .get("distance")
            .ok_or(ConvertError::default())?
            .try_to()?;
        let normal = dict
            .get("normal")
            .ok_or(ConvertError::default())?
            .try_to()?;
        Ok(Self { distance, normal })
    }
}

impl DetectionResult {
    fn new(distance: f32, normal: Vector2) -> Self {
        Self { distance, normal }
    }
}
#[godot_api]
impl IRayCast2D for Sensor {
    fn draw(&mut self) {
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
            Some(result) => Variant::from(result),
            None => Variant::nil(),
        }
    }
}

impl Sensor {
    fn update_debug_label(&mut self) {
        let text: GString = match self.last_result {
            Some(result) => {
                let angle = result.normal.plane_angle();
                format!("{:.0} \n{:.0}°", result.distance, angle.to_degrees()).into()
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
            if detection.distance < 1.0 {
                // Regression, hit a solid wall
                let snapped_position = self.base().get_position();
                let tile_above_position = snapped_position - self.direction.get_target_direction();
                self.base_mut().set_position(tile_above_position);
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
        DetectionResult::new(distance, normal)
    }

    fn get_distance(&self, position: Vector2, collision_point: Vector2) -> f32 {
        if self.direction.is_horizontal() {
            collision_point.x - position.x
        } else {
            collision_point.y - position.y
        }
    }

    fn snap_position(&mut self) {
        let mut position = self.base().get_global_position();
        if self.direction.is_horizontal() {
            position.x = position.x - position.x % TILE_SIZE;
        } else {
            position.y = position.y - position.y % TILE_SIZE;
        };

        self.base_mut().set_global_position(position);
    }
    fn reset_target_position(&mut self) {
        let target_direction = self.direction.get_target_direction();
        self.base_mut().set_target_position(target_direction);
    }
}
