use flecs_ecs::prelude::*;
use std::{any::{Any, TypeId}, collections::HashMap, fmt::Debug, sync::{atomic::AtomicUsize, mpsc::Sender, Arc, RwLock}};
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
        let mut services = ServiceManager::new();
        services.init_service::<ExampleService>();

        // create systems
        world
            .system_named::<(&mut Position, &Velocity)>("move")
            .each(|(p, v)| {
                p.x += v.dx;
                p.y += v.dy;
                println!("{{ {}, {} }}", p.x, p.y);
            });

        let ex_service = services.get_service::<ExampleService>().unwrap();
        world
            .system_named::<()>("run count")
            .each_iter(move |_, _, _| {
                let binding = ex_service.read().unwrap();
                let count = binding.count();
                println!("frame: {}", count);
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
pub trait Service: Any + Send + Sync + Debug {
    fn init() -> Self where Self: Sized;
    fn post_init(&mut self);
}

struct ServiceManager {
    // type Any `RwLock<Box<dyn Service>>`
    services:  RwLock<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>
}

impl ServiceManager {
    pub fn new() -> Self {
        Self { services: RwLock::new(HashMap::new()) }
    }

    pub fn init_service<T: Service>(&mut self) {
        let s: T = T::init();
        let ds: Arc<_> = Arc::new( RwLock::new( Box::new(s) ) );

        self.services
            .write()
            .expect("service manager poisoned")
            .insert(dbg!(TypeId::of::<T>()), ds);
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

    pub fn get_service<T>(&mut self) -> Option<Arc<RwLock<Box<T>>>> where T: Service + Send + Sync {
        let binding = self.services
            .write()
            .expect("service manager poisoned");

        println!("hello1");
        binding
            .get(dbg!(&TypeId::of::<T>()))
            .and_then(|service| {
                println!("hello2");
                let s = service.clone();
                dbg!(std::any::type_name_of_val(&s));
                match Arc::downcast::<RwLock<Box<T>>>(s) {
                    Ok(c) => {
                        Some(dbg!(c))
                    },
                    Err(c) => {
                        dbg!(c);
                        None
                    },
                }
            })
    }
}


#[derive(Debug)]
struct ExampleService {
    frame: AtomicUsize,
}

impl ExampleService {
    pub fn count(&self) -> usize {
        self.frame.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    }
}

impl Service for ExampleService {
    fn init() -> Self where Self: Sized {
        println!("create");
        Self { frame: AtomicUsize::new(0) }
    }

    fn post_init(&mut self) {}
}