mod character;

mod layer_switcher;
mod level_maker;
mod sensor;
mod solid_object;

mod solid_path_2d;
mod tool;
mod vec3_ext;

use godot::prelude::*;

struct SonicMaker;

#[gdextension]
unsafe impl ExtensionLibrary for SonicMaker {}
