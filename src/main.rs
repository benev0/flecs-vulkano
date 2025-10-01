mod graphics;
mod world;

use std::{error::Error, sync::atomic::{AtomicBool, Ordering::Relaxed}};
use std::thread;

use winit::{event_loop::EventLoop};



fn main() -> Result<(), impl Error>{
    static GRAPHICS_CLOSED: AtomicBool = AtomicBool::new(false);

    let game_thread = thread::spawn(move || {
        let game = crate::world::GameSystems::new();
        while !GRAPHICS_CLOSED.load(Relaxed) && game.world().progress() {}
    });

    let event_loop = EventLoop::new().unwrap();
    let mut app = crate::graphics::GraphicsDisplay::new(&event_loop);
    let result = event_loop.run_app(&mut app);

    GRAPHICS_CLOSED.store(true, Relaxed);

    let _ = game_thread.join();
    result
}
