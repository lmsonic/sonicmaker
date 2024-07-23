use std::f32::consts::{FRAC_PI_2, PI};

use godot::builtin::Vector2;

pub trait Vector2Ext {
    fn angle_0_360(&self) -> f32;
}
impl Vector2Ext for Vector2 {
    fn angle_0_360(&self) -> f32 {
        let mut angle = self.angle() + FRAC_PI_2;
        if angle < 0.0 {
            angle += PI * 2.0;
        }
        angle
    }
}
