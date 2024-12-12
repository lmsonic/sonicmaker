use godot::{
    classes::{CollisionPolygon2D, IPath2D, Path2D},
    prelude::*,
};

#[derive(GodotClass)]
#[class(tool,init, base=Path2D)]
pub struct SolidPath2D {
    #[export]
    shape: Option<Gd<CollisionPolygon2D>>,

    old_points: PackedVector2Array,
    base: Base<Path2D>,
}

#[godot_api]
impl IPath2D for SolidPath2D {
    fn physics_process(&mut self, _delta: f64) {
        if let Some(curve) = self.base().get_curve() {
            if let Some(shape) = &mut self.shape {
                let points = curve.get_baked_points();
                if points != self.old_points {
                    self.old_points = points.clone();
                    shape.set_polygon(&points);
                }
            };
        };
    }
}
