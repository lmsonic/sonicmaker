use std::f32::consts::FRAC_PI_2;

use godot::{
    engine::{Button, IButton, InputEvent, InputEventMouseButton, Texture2D},
    global::MouseButton,
    prelude::*,
};

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
    pub tool_rotation: f32,
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
                    self.tool_rotation += FRAC_PI_2;
                } else if mouse_button.get_button_index() == MouseButton::WHEEL_DOWN {
                    self.tool_rotation -= FRAC_PI_2;
                }
            }
        }
    }
}

#[godot_api]
impl Tool {
    #[func]
    fn on_pressed(&mut self) {
        let base = self.base().clone();
        self.base_mut()
            .emit_signal("tool_selected".into(), &[base.to_variant()]);
    }

    pub fn on_tool_used(&mut self) {
        self.base_mut().emit_signal("tool_used".into(), &[]);
    }
    pub fn on_tool_rotated(&mut self) {
        self.base_mut().emit_signal("tool_rotated".into(), &[]);
    }
    #[func]
    fn setup(&mut self) {
        let base = self.base().clone();
        self.base_mut()
            .connect(c"pressed".into(), base.callable(c"on_pressed"));
    }
    #[signal]
    fn tool_rotated();
    #[signal]
    fn tool_selected(tool: Gd<Tool>);
    #[signal]
    fn tool_used();
}
