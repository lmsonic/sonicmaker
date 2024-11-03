/// Most of the code in this project is based on <https://info.sonicretro.org/Sonic_Physics_Guide>
mod character;

pub mod layer_switcher;
mod level_maker;
pub mod sensor;
mod solid_object;

mod solid_path_2d;
mod tool;
mod vec3_ext;

use godot::prelude::*;

struct SonicMaker;

#[gdextension]
unsafe impl ExtensionLibrary for SonicMaker {}
