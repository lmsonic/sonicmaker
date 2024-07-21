use godot::{
    engine::{CollisionShape2D, RectangleShape2D, Shape2D},
    prelude::*,
};
#[derive(GodotClass)]
#[class(tool,init, base=CharacterBody2D)]
pub struct Character {
    #[export(range = (0.0,100.0, 1.0))]
    #[var(get, set = set_width_radius)]
    #[init(default = 19.0)]
    width_radius: f32,
    #[export(range = (0.0,100.0, 1.0))]
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
    collision_shape: Option<Gd<CollisionShape2D>>,
}

#[godot_api]
impl Character {
    #[func]
    fn set_width_radius(&mut self, value: f32) {
        self.width_radius = value;
        let Some(mut rectangle) = self.get_rectangle() else {
            return;
        };

        let y = rectangle.get_size().y;
        rectangle.set_size(Vector2::new(self.width_radius, y));
    }

    #[func]
    fn set_height_radius(&mut self, value: f32) {
        self.height_radius = value;
        let Some(mut rectangle) = self.get_rectangle() else {
            return;
        };

        let x = rectangle.get_size().x;
        rectangle.set_size(Vector2::new(x, self.height_radius));
    }

    fn get_rectangle(&self) -> Option<Gd<RectangleShape2D>> {
        self.collision_shape
            .as_deref()?
            .get_shape()?
            .try_cast()
            .ok()
    }
}
