use godot::{
    engine::{CollisionShape2D, RectangleShape2D},
    prelude::*,
};
#[derive(GodotClass)]
#[class(tool,init, base=CharacterBody2D)]
pub struct Character {
    #[export]
    #[var(get, set = set_width_radius)]
    #[init(default = 19.0)]
    width_radius: f32,
    #[export]
    #[var(get, set = set_height_radius)]
    #[init(default = 39.0)]
    height_radius: f32,
    #[export]
    #[init(default = 20.0)]
    push_radius: f32,
    #[export]
    #[init(default = 6.5)]
    jump_force: f32,
    #[export]
    shape: Option<Gd<CollisionShape2D>>,
}

#[godot_api]
impl Character {
    #[func]
    fn set_width_radius(&mut self, value: f32) {
        self.width_radius = value;
        if let Some(shape) = self.shape.as_deref().and_then(|shape| shape.get_shape()) {
            if let Ok(mut rectangle) = shape.try_cast::<RectangleShape2D>() {
                let y = rectangle.get_size().y;
                rectangle.set_size(Vector2::new(self.width_radius, y));
            }
        }
    }

    #[func]
    fn set_height_radius(&mut self, value: f32) {
        self.height_radius = value;
        if let Some(shape) = self.shape.as_deref().and_then(|shape| shape.get_shape()) {
            if let Ok(mut rectangle) = shape.try_cast::<RectangleShape2D>() {
                let x = rectangle.get_size().x;
                rectangle.set_size(Vector2::new(x, self.height_radius));
            }
        }
    }
}
