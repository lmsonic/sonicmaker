mod airborne;
mod collision;
pub mod godot_api;
mod grounded;
mod lifecycle;
mod utils;

use godot::classes::{AnimatedSprite2D, CollisionShape2D};
use godot::prelude::*;
use godot_api::{SolidObjectKind, State};

#[derive(GodotConvert, Var, Export, Default, Debug, PartialEq, Eq, Clone, Copy)]
#[godot(via = GString)]
pub enum SpindashStyle {
    #[default]
    None,
    Genesis,
    CD,
}

#[derive(GodotConvert, Var, Export, Default, Debug, PartialEq, Eq, Clone, Copy)]
#[godot(via = GString)]
enum MidAirAction {
    #[default]
    None,
    DropDash,
    InstaShield,
    Flying,
    Gliding,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
enum DropDashState {
    #[default]
    NotCharged,
    Charging {
        timer: i32,
    },
    Charged,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
enum SuperPeeloutState {
    #[default]
    NotCharged,
    Charging {
        timer: i32,
    },
    Charged,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
enum SpindashCDState {
    #[default]
    NotCharged,
    Charging {
        timer: i32,
    },
    Charged,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
enum SpindashGenesisState {
    #[default]
    NotCharged,
    Charging {
        charge: f32,
    },
}

use crate::sensor::Sensor;
/// Player class, the code is from all over <https://info.sonicretro.org/Sonic_Physics_Guide>
/// but I will point to specifics when needed
#[allow(clippy::struct_excessive_bools)]
#[derive(GodotClass)]
#[class(init, base=Node2D)]
pub struct Character {
    /// Character state, used both for logic and animation
    #[export]
    #[var(get, set = set_state)]
    pub(crate) state: State,
    #[export]
    sprites: Option<Gd<AnimatedSprite2D>>,
    #[export(range = (0.0,100.0, 1.0))]
    #[var(get, set = set_width_radius)]
    #[init(val = 9.0)]
    width_radius: f32,
    #[export(range = (0.0,100.0, 1.0))]
    #[var(get, set = set_height_radius)]
    #[init(val = 19.0)]
    height_radius: f32,
    #[export(range = (0.0,100.0, 1.0))]
    #[var(get, set = set_push_radius)]
    #[init(val = 10.0)]
    push_radius: f32,
    #[init(val = 6.5)]
    jump_force: f32,
    #[init(val = 0.09375)]
    air_acceleration: f32,
    #[init(val = 0.046875)]
    acceleration: f32,
    #[init(val = 0.5)]
    deceleration: f32,
    #[init(val = 0.046875)]
    friction: f32,
    /// Top speed on the air and grounded (except when rolling)
    #[init(val = 6.0)]
    top_speed: f32,
    #[init(val = 0.21875)]
    gravity: f32,
    /// Slope multiplier when not rolling
    #[init(val = 0.125)]
    slope_factor_normal: f32,
    /// Slope multiplier rolling up
    #[init(val = 0.078125)]
    slope_factor_rollup: f32,
    /// Slope multiplier rolling down
    #[init(val = 0.3125)]
    slope_factor_rolldown: f32,
    #[init(val = 0.0234375)]
    roll_friction: f32,
    #[init(val = 0.125)]
    roll_deceleration: f32,
    /// Top speed when rolling
    #[init(val = 16.0)]
    roll_top_speed: f32,
    /// Main speed variable, used for maintaining momentum on different slopes and from/to the air
    #[var(set, get)]
    ground_speed: f32,
    /// Debug sensor shape made from the 6 sensors
    #[export]
    sensor_shape: Option<Gd<CollisionShape2D>>,
    /// Hitbox debug shape
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
    /// Used for spawning rings when getting hurt
    #[export]
    scattered_ring_scene: Option<Gd<PackedScene>>,

    #[export]
    #[var(get,set= set_grounded)]
    is_grounded: bool,
    #[export(range = (0.0, 360.0, 0.001, radians_as_degrees))]
    #[var(get,set= set_ground_angle)]
    ground_angle: f32,
    /// Used to stop accepting input for a time
    #[var(set, get)]
    control_lock_timer: i32,
    /// Set to true to display the sensors and ground angle
    #[export]
    debug_draw: bool,
    #[export(flags_2d_physics)]
    #[var(get, set = set_collision_layer)]
    collision_layer: u32,
    /// Spindash mode, either Genesis(Sonic 2 and 3&K) or Sonic CD
    #[export]
    spindash_style: SpindashStyle,
    #[export]
    spindash_dust: Option<Gd<AnimatedSprite2D>>,
    spindash_cd_state: SpindashCDState,
    spindash_genesis_state: SpindashGenesisState,
    /// Set to true to make the spindash boost dependent on how much you charge it
    #[export]
    variable_cd_spindash: bool,

    /// Set to true to give the player the Super Peel Out
    #[export]
    has_super_peel_out: bool,
    super_peel_out_state: SuperPeeloutState,
    /// Set to true to make the super peelout boost dependent on how much you charge it
    #[export]
    variable_super_peelout: bool,
    /// Set the mid air action, either DropDash(Mania), InstaShield(3&K), Flying(Tails) or Gliding(Knuckles)
    #[export]
    mid_air_action: MidAirAction,
    #[init(val = 8.0)]
    drop_dash_speed: f32,
    #[init(val = 12.0)]
    drop_dash_max_speed: f32,

    drop_dash_state: DropDashState,
    insta_shield_timer: i32,

    #[var(set, get)]
    pub velocity: Vector2,

    #[var(set = set_rings, get)]
    rings: i32,
    #[var(set, get)]
    has_jumped: bool,
    has_released_jump: bool,
    #[var(get)]
    attacking: bool,
    invulnerability_timer: i32,
    regather_rings_timer: i32,
    #[var(set, get)]
    spring_bounce_timer: i32,

    #[init(val = 2.0)]
    hurt_x_force: f32,
    #[init(val = -4.0)]
    hurt_y_force: f32,
    #[init(val = 0.1875)]
    hurt_gravity: f32,

    /// Set to true to make the delta used for the player fixed to 60 FPS
    #[export]
    #[init(val = true)]
    fix_delta: bool,

    solid_object_to_stand_on: Option<SolidObjectKind>,
    base: Base<Node2D>,
}
