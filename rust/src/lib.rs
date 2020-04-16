use gdnative::*;

mod gameworld;

fn init(handle: init::InitHandle) {
    handle.add_class::<gameworld::GameWorld>();
}

godot_gdnative_init!();
godot_nativescript_init!(init);
godot_gdnative_terminate!();
