use gdnative::prelude::*;

mod gameworld;

fn init(handle: InitHandle) {
    handle.add_class::<gameworld::GameWorld>();
}

godot_init!(init);
