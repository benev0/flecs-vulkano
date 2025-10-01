use flecs_ecs::prelude::*;
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
    // todo add IOC/services
}

impl GameSystems {
    pub fn new() -> Self {
        let world = World::new();

        // init ioc/services

        // create systems
        world
            .system_named::<(&mut Position, &Velocity)>("move")
            .each(|(p, v)| {
                p.x += v.dx;
                p.y += v.dy;
                println!("{{ {}, {} }}", p.x, p.y);
            });

        world
            .system_named::<(&Position, &Sprite)>("render")
            .each(|(_p, _)| {
                // accumulate data into buffer
            });

            world
            .entity()
            .set(Position { x: 10.0, y: 20.0 })
            .set(Velocity { dx: 0.0, dy: 2.0 });

        world
            .entity()
            .set(Position { x: 10.0, y: 20.0 })
            .set(Velocity { dx: 3.0, dy: 4.0 });


        // set tick rate
        // to do make configurable
        world.set_target_fps(20.0);

        Self { world }
    }

    pub fn world(&self) -> &World {
        &self.world
    }
}