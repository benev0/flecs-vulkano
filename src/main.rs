mod graphics;
mod world;

use std::{error::Error, sync::atomic::{AtomicBool, Ordering::Relaxed}};
use std::thread;

use winit::{event_loop::EventLoop};



fn main() -> Result<(), impl Error>{
    let graphics_closed: AtomicBool = AtomicBool::new(false);

    thread::scope(|s| {
        s.spawn(|| {
            let game = crate::world::GameSystems::new();
            while !graphics_closed.load(Relaxed) && game.world().progress() {}
            println!("graphics closed shutting down engine")
        });

        let event_loop = EventLoop::new().unwrap();
        let mut app = crate::graphics::GraphicsDisplay::new(&event_loop);
        let result = event_loop.run_app(&mut app);
        graphics_closed.store(true, Relaxed);
        result
    })
}
