use godot::{
    engine::{Area2D, CollisionShape2D, IArea2D, RectangleShape2D, SegmentShape2D, ThemeDb},
    prelude::*,
};

use crate::character::Character;

#[derive(GodotConvert, Var, Export, Default, Debug, PartialEq, Eq, Clone, Copy)]
#[godot(via = GString)]
enum Direction {
    Horizontal,
    #[default]
    Vertical,
}
#[derive(GodotConvert, Var, Export, Default, Debug, PartialEq, Eq, Clone, Copy)]
#[godot(via = GString)]
enum SwitcherTypeChange {
    #[default]
    PhysicsLayer,
    ZIndex,
    Both,
}
#[derive(GodotClass)]
#[class(tool,init, base=Area2D)]
struct LayerSwitcher {
    #[export(range = (0.0, 100.0,1.0,or_greater))]
    #[var(get,set = set_length)]
    #[init(default = 50.0)]
    length: f32,

    #[export]
    #[var(get,set = set_direction)]
    direction: Direction,
    #[export]
    collision_shape: Option<Gd<CollisionShape2D>>,
    base: Base<Area2D>,
    #[export]
    grounded_only: bool,
    #[export]
    change_type: SwitcherTypeChange,

    // Negative: Left or Down
    // Positive: Right or Up
    #[export(flags_3d_physics)]
    negative_side_layer: u32,
    #[export]
    negative_side_z_index: u32,
    #[export(flags_3d_physics)]
    positive_side_layer: u32,
    #[export]
    positive_side_z_index: u32,
    #[export]
    current_side_of_player: bool,
}
#[godot_api]
impl IArea2D for LayerSwitcher {
    fn physics_process(&mut self, delta: f64) {
        if let Some(player) = self.get_player() {
            let is_player_on_positive_side = self.is_player_on_positive_side(&player);
            if self.check_player_entered(&player)
                && self.current_side_of_player != is_player_on_positive_side
            {
                self.switch(player);
            }
            self.current_side_of_player = is_player_on_positive_side;
        };
    }
    fn draw(&mut self) {
        if let Some(font) = ThemeDb::singleton()
            .get_project_theme()
            .and_then(|theme| theme.get_default_font())
        {
            match self.direction {
                Direction::Vertical => {
                    self.base_mut()
                        .draw_string(font.clone(), Vector2::new(-10.0, 0.0), "A".into());
                    self.base_mut()
                        .draw_string(font.clone(), Vector2::new(5.0, 0.0), "B".into());
                }
                Direction::Horizontal => {
                    self.base_mut()
                        .draw_string(font.clone(), Vector2::new(0.0, 15.0), "A".into());
                    self.base_mut()
                        .draw_string(font, Vector2::new(0.0, -5.0), "B".into());
                }
            }
        }
    }
}

#[godot_api]
impl LayerSwitcher {
    #[func]
    fn set_length(&mut self, value: f32) {
        self.length = value;
        if let Some(rectangle) = self.get_segment_mut() {
            self.update_segment(rectangle);
        }
    }

    #[func]
    fn set_direction(&mut self, value: Direction) {
        self.direction = value;
        if let Some(rectangle) = self.get_segment_mut() {
            self.update_segment(rectangle);
        }
        self.base_mut().queue_redraw();
    }
}

impl LayerSwitcher {
    fn get_player(&self) -> Option<Gd<Character>> {
        let player_group = StringName::from(c"player");
        self.base()
            .get_tree()?
            .get_first_node_in_group(player_group)?
            .try_cast::<Character>()
            .ok()
    }
    fn check_player_entered(&self, player: &Gd<Character>) -> bool {
        let position = self.base().get_global_position();
        let player_position = player.get_global_position();
        let check_range = match self.direction {
            Direction::Horizontal => (position.x - self.length)..=(position.x + self.length),
            Direction::Vertical => (position.y - self.length)..=(position.y + self.length),
        };
        check_range.contains(match self.direction {
            Direction::Horizontal => &player_position.x,
            Direction::Vertical => &player_position.y,
        })
    }
    fn is_player_on_positive_side(&self, player: &Gd<Character>) -> bool {
        let player_position = player.get_global_position();
        let position = self.base().get_global_position();
        match self.direction {
            // Negative y is up
            Direction::Horizontal => player_position.y <= position.y,
            Direction::Vertical => player_position.x >= position.x,
        }
    }
    fn switch(&mut self, player: Gd<Character>) {
        godot_print!("switching");
    }
    fn update_segment(&mut self, mut segment: Gd<SegmentShape2D>) {
        match self.direction {
            Direction::Vertical => {
                segment.set_a(Vector2::new(0.0, -self.length));
                segment.set_b(Vector2::new(0.0, self.length));
            }
            Direction::Horizontal => {
                segment.set_a(Vector2::new(-self.length, 0.0));
                segment.set_b(Vector2::new(self.length, 0.0));
            }
        }
    }
    fn get_segment_mut(&mut self) -> Option<Gd<SegmentShape2D>> {
        self.collision_shape
            .as_deref_mut()?
            .get_shape()?
            .try_cast::<SegmentShape2D>()
            .ok()
    }
}
