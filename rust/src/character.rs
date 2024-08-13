mod collision;
mod godot_api;
mod lifecycle;
mod utils;

use godot::engine::{AnimatedSprite2D, Area2D, CollisionShape2D};
use godot::prelude::*;
use godot_api::{Kind, SolidObjectKind, State};

use crate::sensor::Sensor;
#[allow(clippy::struct_excessive_bools)]
#[derive(GodotClass)]
#[class(init, base=Node2D)]
pub struct Character {
    #[export]
    #[var(get, set = set_character)]
    character: Kind,
    #[export]
    #[var(get, set = set_state)]
    state: State,
    #[export]
    sprites: Option<Gd<AnimatedSprite2D>>,
    #[export(range = (0.0,100.0, 1.0))]
    #[var(get, set = set_width_radius)]
    #[init(default = 9.0)]
    width_radius: f32,
    #[export(range = (0.0,100.0, 1.0))]
    #[var(get, set = set_height_radius)]
    #[init(default = 19.0)]
    height_radius: f32,
    #[export(range = (0.0,100.0, 1.0))]
    #[var(get, set = set_push_radius)]
    #[init(default = 10.0)]
    push_radius: f32,

    #[init(default = 6.5)]
    jump_force: f32,
    #[init(default = 0.09375)]
    air_acceleration: f32,
    #[init(default = 0.046875)]
    acceleration: f32,
    #[init(default = 0.5)]
    deceleration: f32,
    #[init(default = 0.046875)]
    friction: f32,
    #[init(default = 6.0)]
    top_speed: f32,
    #[init(default = 0.21875)]
    gravity: f32,
    #[init(default = 0.125)]
    slope_factor_normal: f32,
    #[init(default = 0.078125)]
    slope_factor_rollup: f32,
    #[init(default = 0.3125)]
    slope_factor_rolldown: f32,
    #[init(default = 0.0234375)]
    roll_friction: f32,
    #[init(default = 0.125)]
    roll_deceleration: f32,
    #[init(default = 16.0)]
    roll_top_speed: f32,
    #[export]
    ground_speed: f32,

    #[export]
    sensor_shape: Option<Gd<CollisionShape2D>>,
    #[export]
    hitbox_area: Option<Gd<Area2D>>,
    #[export]
    hitbox_shape: Option<Gd<CollisionShape2D>>,
    #[export]
    sensor_floor_left: Option<Gd<Sensor>>,
    #[export]
    sensor_floor_right: Option<Gd<Sensor>>,
    #[export]
    sensor_ceiling_left: Option<Gd<Sensor>>,
    #[export]
    sensor_ceiling_right: Option<Gd<Sensor>>,
    #[export]
    sensor_push_left: Option<Gd<Sensor>>,
    #[export]
    sensor_push_right: Option<Gd<Sensor>>,
    #[export]
    #[var(get,set= set_grounded)]
    is_grounded: bool,
    #[export(range = (0.0, 360.0, 0.001, radians_as_degrees))]
    #[var(get,set= set_ground_angle)]
    ground_angle: f32,
    control_lock_timer: i32,
    #[export]
    debug_draw: bool,
    #[export(flags_2d_physics)]
    #[var(get, set = set_collision_layer)]
    collision_layer: u32,
    #[export]
    #[var(set, get)]
    pub velocity: Vector2,
    #[export]
    #[var(set, get)]
    rings: i32,
    #[var(set, get)]
    attacking: bool,

    #[init(default = 2.0)]
    hurt_x_force: f32,
    #[init(default = -4.0)]
    hurt_y_force: f32,
    #[init(default = 0.1875)]
    hurt_gravity: f32,

    #[export]
    #[init(default = true)]
    fix_delta: bool,

    solid_object_to_stand_on: Option<SolidObjectKind>,
    base: Base<Node2D>,
}
