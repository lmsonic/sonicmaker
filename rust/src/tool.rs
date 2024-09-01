use godot::{
    engine::{Button, IButton, Texture2D},
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
    pub tile_size: Vector2,
    base: Base<Button>,
}
#[godot_api]
impl IButton for Tool {
    fn ready(&mut self) {
        self.setup();
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
    #[func]
    fn setup(&mut self) {
        let base = self.base().clone();
        self.base_mut()
            .connect(c"pressed".into(), base.callable(c"on_pressed"));
    }

    #[signal]
    fn tool_selected(tool: Gd<Tool>);
    #[signal]
    fn tool_used();
}
