use flecs_ecs::prelude::*;
use std::{any::{Any, TypeId}, collections::HashMap, sync::{mpsc::Sender, Arc, RwLock}};
// todo: build basic systems here

#[derive(Component)]
struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Component)]
struct Velocity {
    pub dx: f32,
    pub dy: f32,
}

#[derive(Component)]
struct Sprite;


pub struct GameSystems {
    world: World,
    services: ServiceManager,
    // todo add IOC/services
}

impl GameSystems {
    pub fn new() -> Self {
        let world = World::new();

        // init ioc/services
        let services = ServiceManager::new();

        // create systems
        world
            .system_named::<(&mut Position, &Velocity)>("move")
            .each(|(p, v)| {
                p.x += v.dx;
                p.y += v.dy;
                println!("{{ {}, {} }}", p.x, p.y);
            });

        // register default entity
        world
            .entity()
            .set(Position { x: 10.0, y: 20.0 })
            .set(Velocity { dx: 0.0, dy: 2.0 });

        // set tick rate
        // to do make configurable
        world.set_target_fps(20.0);

        Self { world, services }
    }

    pub fn with_render(&self, tx: Sender<(f32, f32)>){
        self.world
            .system_named::<(&Position, &Sprite)>("render")
            .each(move |(p, _)| {
                let _ = tx.send((p.x, p.y));
            });
    }

    pub fn world(&self) -> &World {
        &self.world
    }
}


// services
pub trait Service: Any + Send + Sync {
    fn init() -> Self where Self: Sized;
    fn post_init(&mut self);
}

struct ServiceManager {
    services:  RwLock<HashMap<TypeId, Arc<RwLock<Box<dyn Service>>>>>
}

impl ServiceManager {
    pub fn new() -> Self {
        Self { services: RwLock::new(HashMap::new()) }
    }

    pub fn init_service<T: Service>(&mut self) {
        let s: T = T::init();
        let ds: Arc<RwLock<Box<dyn Service>>> = Arc::new( RwLock::new( Box::new(s) ) );

        self.services
            .write()
            .expect("service manager poisoned")
            .insert(TypeId::of::<T>(), ds);
    }

    pub fn post_init_service<T: Service>(&mut self) {
        self.get_service::<T>()
            .map(|service| {
                let mut binding = service
                    .write()
                    .expect("service poisoned");

                binding.post_init();
            });
    }

    pub fn get_service<T>(&mut self) -> Option<Arc<RwLock<T>>> where T: Service + Send + Sync {
        let binding = self.services
            .write()
            .expect("service manager poisoned");

        binding
            .get(&TypeId::of::<T>())
            .and_then(|service| {
                let s = service.clone();
                Arc::downcast::<RwLock<T>>(s).ok()
            })
    }
}
