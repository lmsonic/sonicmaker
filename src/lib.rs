mod character;
mod layer_switcher;
mod sensor;
pub mod vec3_ext;

use godot::prelude::*;

struct SonicMaker;

#[gdextension]
unsafe impl ExtensionLibrary for SonicMaker {}
