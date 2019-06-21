use specs::prelude::*;

#[derive(Debug)]
pub struct Position {
    pub x: f64,
    pub y: f64
}

impl Component for Position {
    type Storage = VecStorage<Self>;
}

#[derive(Debug)]
pub struct Coords {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug)]
pub struct TrainEngine {
    pub velocity:     Coords,
    pub acceleration: Coords,
    pub vmin: f64,
    pub vmax: f64,
}

impl Component for TrainEngine {
    type Storage = HashMapStorage<Self>;
}

pub struct TrainEngineSystem;

impl<'a> System<'a> for TrainEngineSystem {
    type SystemData = (
        WriteStorage<'a, Position>,
        WriteStorage<'a, TrainEngine>,
        Read<'a, super::DeltaTime>,
    );

    fn run(&mut self, (mut positions, mut engines, delta): Self::SystemData) {
        for (position, engine) in (&mut positions, &mut engines).join() {
            engine.velocity.x += engine.acceleration.x * delta.fraction;
            engine.velocity.y += engine.acceleration.y * delta.fraction;

            position.x += engine.velocity.x * delta.fraction;
            position.y += engine.velocity.y * delta.fraction;
        }
    }
}
