use std::f32::consts::{FRAC_PI_2, PI};

use godot::{
    classes::{Button, IButton, InputEvent, InputEventMouseButton, Texture2D},
    global::MouseButton,
    prelude::*,
};
#[derive(GodotConvert, Var, Export, Default, Debug, PartialEq, Eq, Clone, Copy)]
#[godot(via = i32)]
pub enum Direction {
    #[default]
    Up = 0,
    Right = 1,
    Down = 2,
    Left = 3,
}
#[derive(GodotClass)]
#[class(init, base=Button)]
pub struct Tool {
    #[export]
    pub cursor_placeholder: Option<Gd<Texture2D>>,
    #[export]
    pub only_just_pressed: bool,
    #[export]
    pub can_rotate: bool,
    #[var(set, get)]
    pub tool_direction: Direction,
    #[export]
    pub tile_size: Vector2,
    base: Base<Button>,
}
#[godot_api]
impl IButton for Tool {
    fn ready(&mut self) {
        self.setup();
    }
    fn input(&mut self, event: Gd<InputEvent>) {
        if !self.can_rotate {
            return;
        }
        if let Ok(mouse_button) = event.try_cast::<InputEventMouseButton>() {
            if mouse_button.is_pressed() {
                if mouse_button.get_button_index() == MouseButton::WHEEL_UP {
                    self.tool_direction = match self.tool_direction {
                        Direction::Up => Direction::Right,
                        Direction::Right => Direction::Down,
                        Direction::Down => Direction::Left,
                        Direction::Left => Direction::Up,
                    }
                } else if mouse_button.get_button_index() == MouseButton::WHEEL_DOWN {
                    self.tool_direction = match self.tool_direction {
                        Direction::Up => Direction::Left,
                        Direction::Right => Direction::Up,
                        Direction::Down => Direction::Right,
                        Direction::Left => Direction::Down,
                    }
                }
            }
        }
    }
}

#[godot_api]
impl Tool {
    #[func]
    pub(crate) fn tool_rotation(&self) -> f32 {
        match self.tool_direction {
            Direction::Up => 0.0,
            Direction::Right => FRAC_PI_2,
            Direction::Down => PI,
            Direction::Left => -FRAC_PI_2,
        }
    }
    #[func]
    fn on_pressed(&mut self) {
        let base = self.base().clone();
        self.base_mut()
            .emit_signal("tool_selected", &[base.to_variant()]);
    }

    pub fn on_tool_used(&mut self) {
        self.base_mut().emit_signal("tool_used", &[]);
    }
    pub fn on_tool_rotated(&mut self) {
        self.base_mut().emit_signal("tool_rotated", &[]);
    }
    #[func]
    fn setup(&mut self) {
        let base = self.base().clone();
        self.base_mut()
            .connect(c"pressed", &base.callable(c"on_pressed"));
    }
    #[signal]
    fn tool_rotated();
    #[signal]
    fn tool_selected(tool: Gd<Tool>);
    #[signal]
    fn tool_used();
}
