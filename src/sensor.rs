use godot::{
    engine::{Engine, Font, IRayCast2D, Label, RayCast2D, Theme, ThemeDb},
    prelude::*,
};

#[derive(GodotConvert, Var, Export, Default, Debug, PartialEq, Eq, Clone, Copy)]
#[godot(via = GString)]
enum Direction {
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
struct Sensor {
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
struct DetectionResult {
    distance: f32,
    normal: Vector2,
}

impl DetectionResult {
    fn new(distance: f32, normal: Vector2) -> Self {
        Self { distance, normal }
    }
}
#[godot_api]
impl IRayCast2D for Sensor {
    fn physics_process(&mut self, delta: f64) {
        if self.update_in_editor && Engine::singleton().is_editor_hint() {
            self.last_result = self.detect_solid();
            self.base_mut().queue_redraw();
        }
    }
    fn draw(&mut self) {
        if self.show_debug_label {
            self.update_debug_label();
        }
    }
}

#[godot_api]
impl Sensor {
    fn update_debug_label(&mut self) {
        let text: GString = match self.detect_solid() {
            Some(result) => format!(
                "{:.0} \n{:.0}°",
                result.distance,
                result.normal.angle().to_degrees()
            )
            .into(),

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
    #[func]
    fn set_direction(&mut self, value: Direction) {
        self.direction = value;
        self.reset_target_position();
    }

    fn detect_solid(&mut self) -> Option<DetectionResult> {
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
