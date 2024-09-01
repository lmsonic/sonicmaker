use godot::{
    engine::{CollisionShape2D, ILine2D, Line2D, PhysicsBody2D, RectangleShape2D},
    prelude::*,
};
#[derive(GodotClass)]
#[class(init, base=Line2D)]
pub struct SolidLine2D {
    #[export]
    body: Option<Gd<PhysicsBody2D>>,
    base: Base<Line2D>,
}
#[godot_api]
impl ILine2D for SolidLine2D {
    fn ready(&mut self) {
        let count = self.base().get_point_count() as usize;
        if count < 2 {
            return;
        }
        let width = self.base().get_width();

        let points = self.base().get_points();
        let Some(body) = &mut self.body else {
            return;
        };
        for i in 0..count - 1 {
            let mut shape = CollisionShape2D::new_alloc();
            let mut rect = RectangleShape2D::new_gd();
            let p1 = points[i];
            let p2 = points[i + 1];
            shape.set_position((p1 + p2) / 2.0);
            shape.set_rotation(p1.direction_to(p2).angle());
            let length = p1.distance_to(p2);
            rect.set_size(Vector2::new(length + 2.0, width));
            shape.set_shape(rect.upcast());
            body.add_child(shape.upcast());
        }
    }
}
