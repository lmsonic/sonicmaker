use godot::{
    engine::{CollisionPolygon2D, Curve2D, IPath2D, Path2D},
    prelude::*,
};

#[derive(GodotClass)]
#[class(tool,init, base=Path2D)]
pub struct SolidPath2D {
    #[export]
    shape: Option<Gd<CollisionPolygon2D>>,
    #[export]
    #[var(set=set_width,get)]
    #[init(default = 1.0)]
    width: f32,
    base: Base<Path2D>,
}

#[godot_api]
impl IPath2D for SolidPath2D {
    fn enter_tree(&mut self) {
        if let Some(mut curve) = self.base().get_curve() {
            if !curve.is_connected(c"changed".into(), self.base().callable(c"curve_changed")) {
                curve.connect(c"changed".into(), self.base().callable(c"curve_changed"));
            }
        }
    }
}
#[godot_api]
impl SolidPath2D {
    #[func]
    fn set_width(&mut self, value: f32) {
        self.width = value;
        self.curve_changed();
    }

    #[func]
    fn curve_changed(&self) {
        let Some(curve) = self.base().get_curve() else {
            return;
        };
        let Some(shape) = &mut self.shape.clone() else {
            return;
        };
        let points = curve.get_baked_points();
        let len = points.len();
        let mut segments = points.clone();

        for i in (0..len).rev() {
            let normal = if i == 0 {
                Vector2::DOWN
            } else {
                let p1 = points[i - 1];
                let p2 = points[i];
                let dx = p2.x - p1.x;
                let dy = p2.y - p1.y;
                Vector2::new(-dy, dx).normalized()
            };
            let point = points[i] + normal * self.width;
            segments.push(point);
        }
        shape.set_polygon(segments);
    }
}
