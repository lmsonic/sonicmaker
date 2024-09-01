use godot::{
    engine::{Control, InputEvent, Sprite2D},
    prelude::*,
};

use crate::tool::Tool;

#[derive(GodotClass)]
#[class(init, base=Node2D)]
pub struct LevelMaker {
    #[export]
    toolbar: Option<Gd<Control>>,
    #[export]
    cursor: Option<Gd<Sprite2D>>,
    #[export]
    selected_tool: Option<Gd<Tool>>,
    base: Base<Node2D>,
}
#[godot_api]
impl INode2D for LevelMaker {
    fn ready(&mut self) {
        let base = self.base_mut().clone();
        if let Some(toolbar) = &mut self.toolbar {
            for mut tool in toolbar.get_children().iter_shared() {
                tool.connect("tool_selected".into(), base.callable(c"on_tool_selected"));
            }
        }
    }
    fn process(&mut self, _delta: f64) {
        let mouse_position = self.base().get_global_mouse_position();
        if let Some(cursor) = &mut self.cursor {
            cursor.set_global_position(mouse_position);
        }
    }
    fn unhandled_input(&mut self, event: Gd<InputEvent>) {
        if Input::singleton().is_action_just_pressed(c"click".into()) {
            if let Some(tool) = &mut self.selected_tool {
                tool.bind_mut().on_tool_used();
            }
        }
    }
}
#[godot_api]
impl LevelMaker {
    #[func]
    fn on_tool_selected(&mut self, tool: Gd<Tool>) {
        if let Some(cursor) = &mut self.cursor {
            if let Some(cursor_placeholder) = &tool.bind().cursor_placeholder {
                cursor.set_texture(cursor_placeholder.clone());
            }
        }
        self.selected_tool = Some(tool);
    }
}
