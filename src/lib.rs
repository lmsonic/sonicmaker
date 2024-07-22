mod character;
mod sensor;

use godot::prelude::*;

struct SonicMaker;

#[gdextension]
unsafe impl ExtensionLibrary for SonicMaker {}
