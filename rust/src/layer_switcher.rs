use godot::{
    classes::{CollisionShape2D, Engine, SegmentShape2D, ThemeDb},
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

/// From <https://info.sonicretro.org/SPG:Solid_Terrain#Layers>
/// It switches either collision layer, Z-index, or both when going from one side to the other
#[derive(GodotClass)]
#[class(tool,init, base=Node2D)]
struct LayerSwitcher {
    /// Size of the layer switcher, it will not collide outside of it
    #[export(range = (0.0, 100.0,1.0,or_greater))]
    #[var(get,set = set_length)]
    #[init(val = 50.0)]
    length: f32,

    /// Direction of the switcher, either horizontal or vertical
    #[export]
    #[var(get,set = set_direction)]
    direction: Direction,
    /// Collision shape only for debug purposes, we don't use Godot collision detection
    #[export]
    collision_shape: Option<Gd<CollisionShape2D>>,
    base: Base<Node2D>,
    /// Set to true for making it switch layers only when the player is grounded (for example, at the top of a loop)
    #[export]
    grounded_only: bool,
    /// Switcher functionality, either changes physics layer, z-index or both
    #[export]
    change_type: SwitcherTypeChange,

    /// Negative: Left or Down
    /// Physics layer on the negative side
    #[export(flags_3d_physics)]
    negative_side_layer: u32,
    /// Negative: Left or Down
    /// Z-index on the negative side
    #[export]
    negative_side_z_index: i32,
    /// Positive: Right or Up
    /// Physics layer on the positive side
    #[export(flags_3d_physics)]
    positive_side_layer: u32,
    /// Positive: Right or Up
    /// Z-index on the positive side
    #[export]
    positive_side_z_index: i32,
    #[export]
    current_side_of_player: bool,
    /// Set to true to change layers for the player even when moving it in the editor
    #[export]
    enable_in_editor: bool,
}
#[godot_api]
impl INode2D for LayerSwitcher {
    fn physics_process(&mut self, _delta: f64) {
        if Engine::singleton().is_editor_hint() && !self.enable_in_editor {
            return;
        }
        // Collision check
        if let Some(mut player) = self.get_player() {
            let is_player_on_positive_side = self.is_player_on_positive_side(&player);
            let is_player_grounded = player.bind().get_is_grounded();
            if self.check_player_entered(&player)
                && self.current_side_of_player != is_player_on_positive_side
                && (self.grounded_only == is_player_grounded || !self.grounded_only)
            {
                self.switch(&mut player, is_player_on_positive_side);
            }
            self.current_side_of_player = is_player_on_positive_side;
        };
    }
    fn draw(&mut self) {
        if !Engine::singleton().is_editor_hint() {
            return;
        }
        if let Some(font) = ThemeDb::singleton()
            .get_project_theme()
            .and_then(|theme| theme.get_default_font())
        {
            match self.direction {
                Direction::Vertical => {
                    self.base_mut()
                        .draw_string(&font, Vector2::new(-10.0, 0.0), "A");
                    self.base_mut()
                        .draw_string(&font, Vector2::new(5.0, 0.0), "B");
                }
                Direction::Horizontal => {
                    self.base_mut()
                        .draw_string(&font, Vector2::new(0.0, 15.0), "A");
                    self.base_mut()
                        .draw_string(&font, Vector2::new(0.0, -5.0), "B");
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
    /// Boilerplate to fetch the player from node groups
    fn get_player(&self) -> Option<Gd<Character>> {
        self.base()
            .get_tree()?
            .get_first_node_in_group("player")?
            .try_cast::<Character>()
            .ok()
    }
    /// Returns true if the player has crossed the layer switcher
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
    /// Returns true if the player is on the positive side (Up,Right) and
    /// false if the player is on the negative side (Down,Left)
    fn is_player_on_positive_side(&self, player: &Gd<Character>) -> bool {
        let player_position = player.get_global_position();
        let position = self.base().get_global_position();
        match self.direction {
            // Negative y is up
            Direction::Horizontal => player_position.y <= position.y,
            Direction::Vertical => player_position.x >= position.x,
        }
    }
    /// Changes physics layer and/or z-index for the player
    fn switch(&self, player: &mut Gd<Character>, current_player_side: bool) {
        let layer = if current_player_side {
            self.positive_side_layer
        } else {
            self.negative_side_layer
        };
        let z_index = if current_player_side {
            self.positive_side_z_index
        } else {
            self.negative_side_z_index
        };
        match self.change_type {
            SwitcherTypeChange::PhysicsLayer => player.bind_mut().set_collision_layer(layer),
            SwitcherTypeChange::ZIndex => player.set_z_index(z_index),
            SwitcherTypeChange::Both => {
                player.bind_mut().set_collision_layer(layer);
                player.set_z_index(z_index);
            }
        }
        player.bind_mut().update_sensors();
    }
    /// Updates debug collision shape
    fn update_segment(&self, mut segment: Gd<SegmentShape2D>) {
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
