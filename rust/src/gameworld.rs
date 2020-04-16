use std::sync::Mutex;
use gdnative::{
    godot_error, godot_wrap_method, godot_wrap_method_inner, godot_wrap_method_parameter_count,
    methods, NativeClass, InputEvent, Node2D, Vector2
};
use legion::prelude::*;
use lazy_static::lazy_static;


// -----------------------------------------------------------------------------
//     - World -
// -----------------------------------------------------------------------------
lazy_static! {
    static ref WORLD: Mutex<World> = Mutex::new(Universe::new().create_world());
}

pub fn with_world<F>(mut f: F)
where
    F: FnMut(&mut World),
{
    let _ = WORLD.try_lock().map(|mut world| f(&mut world));
}


// -----------------------------------------------------------------------------
//     - Resources -
// -----------------------------------------------------------------------------
pub struct Delta(pub f32);

// -----------------------------------------------------------------------------
//     - Schedules -
// -----------------------------------------------------------------------------
struct Process {
    resources: Resources,
    schedule: Schedule,
}

impl Process {
    fn new() -> Self {
        let mut resources = Resources::default();
        resources.insert(Delta(0.));

        let schedule = Schedule::builder()
            //.add_system(not_threadsafe())
            .add_thread_local(move_node())
            .build();

        Self {
            resources,
            schedule,
        }
    }

    fn execute(&mut self, delta: f64) {
        self.resources
            .get_mut::<Delta>()
            .map(|mut d| d.0 = delta as f32);

        with_world(|mut world| {
            self.schedule.execute(&mut world, &mut self.resources);
        })
    }
}


// -----------------------------------------------------------------------------
//     - Godot node -
//     The world node
// -----------------------------------------------------------------------------
#[derive(NativeClass)]
#[inherit(Node2D)]
pub struct GameWorld {
    process: Process
}

#[methods]
impl GameWorld {
    pub fn _init(_owner: Node2D) -> Self {
        Self {
            process: Process::new(),
        }
    }

    #[export]
    pub fn _ready(&self, owner: Node2D) {
        unsafe {
            let node = owner
                .get_node("TheNode".into())
                .and_then(|node| node.cast::<Node2D>())
                .unwrap();
            
            with_world(|world| {
                world.insert(
                    (), // No tags
                    vec![(NodeComponent(node), )]
                );
            });
        }
    }

    #[export]
    pub fn _unhandled_input(&self, owner: Node2D, event: InputEvent) {
    }

    #[export]
    pub fn _process(&mut self, owner: Node2D, delta: f64) {
        self.process.execute(delta);
    }

    #[export]
    pub fn _physics_process(&self, owner: Node2D, delta: f64) {
    }
}


pub struct NodeComponent(Node2D);

unsafe impl Send for NodeComponent {}
unsafe impl Sync for NodeComponent {}

fn move_node() -> Box<dyn Runnable> {
    SystemBuilder::new("move nodes")
        .read_resource::<Delta>()
        .with_query(<Write<NodeComponent>>::query())
        .build_thread_local(|cmd, world, delta, query| {
            for mut node in query.iter_mut(world) {
                unsafe {
                    let speed = 80.;
                    let vel = Vector2::new(1.0, 0.0) * speed * delta.0;
                    node.0.global_translate(vel);
                }
            }
        })
}
