use flecs_ecs::prelude::*;
use vulkano::pipeline::graphics;
use std::{any::{Any, TypeId}, collections::HashMap, fmt::Debug, mem, sync::{atomic::{AtomicBool, AtomicUsize}, mpsc::Sender, Arc, Mutex, RwLock}};
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
}

impl GameSystems {
    pub fn new() -> Self {
        let world = World::new();

        // init ioc/services
        let mut services = ServiceManager::new();
        services.init_service::<ExampleService>();
        services.init_service::<GraphicsService<()>>();

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
            .run(move |_| {
                let binding = ex_service.read().unwrap();
                let count = binding.count();
                println!("frame: {}", count);
            });

        let graphics_service = services.get_service::<GraphicsService<()>>().unwrap();
        world
            .system_named::<(&Position, &Sprite)>("graphic prep")
            .run(move |mut it| {
                // create or open new buffer

                while it.next() {
                    // g
                    let pos = it.field::<&Position>(0).unwrap(); //at index 0 in (&Position, &Sprite)
                    for i in it.iter() {
                        // place pos data in buffer
                    }
                }

                // store
                let mux = graphics_service.read().unwrap();
                let data = mux.write_next();
                let mut data_guard = data.lock().unwrap();
                let _ = mem::replace(&mut *data_guard, ());
            });

        // register default entities
        world
            .entity()
            .set(Position { x: 10.0, y: 20.0 })
            .set(Velocity { dx: 0.0, dy: 2.0 });

        world
            .entity()
            .set(Position { x: 10.0, y: 20.0 })
            .set(Velocity { dx: 0.0, dy: -2.0 });


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

    pub fn get_service<T>(&mut self) -> Option<Arc<RwLock<Box<T>>>> where T: Service + Send + Sync {
        let binding = self.services
            .write()
            .expect("service manager poisoned");

        binding
            .get(&TypeId::of::<T>())
            .and_then(|service| {
                let s = service.clone();
                Arc::downcast::<RwLock<Box<T>>>(s).ok()
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
        Self { frame: AtomicUsize::new(0) }
    }

    fn post_init(&mut self) {}
}

trait Construct {
    fn new() -> Self;
}

impl Construct for () {
    fn new() -> Self {
        ()
    }
}

#[derive(Debug)]
struct GraphicsService<T> {
    current_write: AtomicBool,
    data1: Arc<Mutex<T>>,
    data2: Arc<Mutex<T>>,
}


impl<T> GraphicsService<T> {
    pub fn read_next(&self) -> Arc<Mutex<T>> {
        let selector = self.current_write.fetch_not(std::sync::atomic::Ordering::AcqRel);
        match selector {
            true => self.data1.clone(),
            false => self.data2.clone(),
        }
    }

    pub fn write_next(&self) -> Arc<Mutex<T>> {
        let selector = self.current_write.load(std::sync::atomic::Ordering::Acquire);
        match selector {
            true => self.data2.clone(),
            false => self.data1.clone(),
        }
    }


}

impl<T> Service for GraphicsService<T>
where T: Debug + Send + Construct + 'static {
    fn init() -> Self where Self: Sized {
        Self {
            current_write: AtomicBool::new(true),
            data1: Arc::new(Mutex::new(T::new())),
            data2: Arc::new(Mutex::new(T::new())),
        }
    }

    fn post_init(&mut self) { }
}