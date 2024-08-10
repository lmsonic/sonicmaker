mod character;

mod layer_switcher;
mod sensor;
mod sloped_solid_object;
mod solid_object;
mod vec3_ext;

use godot::prelude::*;

struct SonicMaker;

#[gdextension]
unsafe impl ExtensionLibrary for SonicMaker {}
