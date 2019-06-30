use specs::prelude::*;

use super::routing::{TrainRoute,TrainIsInStation};

#[derive(Clone,Debug,PartialEq)]
pub struct Vector {
    pub x: f64,
    pub y: f64,
}

impl Vector {
    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    pub fn length(&self) -> f64 {
        (self.x.powi(2) + self.y.powi(2)).sqrt()
    }

    pub fn scale_to_length(&self, new_length: f64) -> Self {
        let fac = new_length / self.length();
        Self {
            x: self.x * fac,
            y: self.y * fac
        }
    }
}

#[derive(Clone,Debug,PartialEq)]
pub struct Position {
    pub x: f64,
    pub y: f64
}

impl Position {
    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    pub fn new(x: f64, y: f64) -> Self {
        Self { x: x, y: y }
    }

    pub fn distance_to(&self, other: &Position) -> Vector {
        Vector {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }

    pub fn distance_length_to(&self, other: &Position) -> f64 {
        self.distance_to(other).length()
    }

    #[allow(dead_code)]
    pub fn as_f32_array(&self) -> [f32; 2] {
        [self.x as f32, self.y as f32]
    }

    #[allow(dead_code)]
    pub fn as_f64_array(&self) -> [f64; 2] {
        [self.x, self.y]
    }

    #[allow(dead_code)]
    pub fn as_f32_tuple(&self) -> (f32, f32) {
        (self.x as f32, self.y as f32)
    }

    #[allow(dead_code)]
    pub fn as_f64_tuple(&self) -> (f64, f64) {
        (self.x, self.y)
    }
}

impl From<[f64; 2]> for Position {
    fn from(floats: [f64; 2]) -> Self {
        Self {
            x: floats[0],
            y: floats[1]
        }
    }
}

impl From<[f32; 2]> for Position {
    fn from(floats: [f32; 2]) -> Self {
        Self {
            x: floats[0] as f64,
            y: floats[1] as f64
        }
    }
}

impl Component for Position {
    type Storage = VecStorage<Self>;
}


#[derive(Debug)]
pub struct TrainEngine {
    pub velocity:     Vector,
    pub acceleration: Vector,
    pub vmax: f64,
    pub amax: f64,
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

            // TODO: Being a machine that runs on rails, I should probably make sure I'm actually on the tracks

            position.x += engine.velocity.x * delta.fraction;
            position.y += engine.velocity.y * delta.fraction;
        }
    }
}

/**
 * The Driver's job is simple:
 * Go someplace, and be sure to start slowing down in time so you don't go too far and kill everyone.
 */
pub struct TrainDriver;

impl<'a> System<'a> for TrainDriver {
    type SystemData = (
        ReadStorage<'a, Position>,         // I'm somewhere
        WriteStorage<'a, TrainEngine>,     // I haz an engine that I can play with
        ReadStorage<'a, TrainRoute>,       // I wanna go somewhere
        ReadStorage<'a, TrainIsInStation>, // or I'm in a station
    );

    fn run(&mut self, sys_data: Self::SystemData) {
        let (
            positions,
            mut engines,
            routes,
            trains_in_station
        ) = sys_data;
        // Open Road
        for (train_pos, mut engine, route) in (&positions, &mut engines, &routes).join() {
            let next_pos = positions.get(route.next_hop()).unwrap();
            //println!("I'm in ur {:?}, going to {:?}", train_pos, next_pos);
            let direction = next_pos.distance_to(train_pos);
            let distance = direction.length();

            // Make sure we're going in the right direction
            engine.velocity = direction.scale_to_length(engine.velocity.length());

            // Let's see how far before the next_hop we'll have to brake.
            // 1. Number of accelerations I need to brake away is velocity/acceleration;
            //    since acceleration = change in velocity per second, this is in seconds
            // 2. how far I'm travelling in that time is given by the velocity
            // 3. thus t = v/a, distance = v * t -> distance = v^2/a
            //    if we're closer than this distance, brake furiously
            let braking_distance = engine.velocity.length().powi(2) / engine.amax;
            if distance < braking_distance {
                engine.acceleration = direction.scale_to_length(-engine.amax);
                continue;
            }

            // Ok, no need to brake. Let's see if we want to accelerate.
            if engine.velocity.length() < engine.vmax {
                engine.acceleration = direction.scale_to_length(engine.amax);
                continue;
            }

            engine.acceleration = Vector::zero();
        }
        // In a station
        for (train_pos, mut engine, station) in (&positions, &mut engines, &trains_in_station).join() {
            let next_pos = positions.get(station.station).unwrap();
            //println!("I'm in ur {:?}, going to {:?}", train_pos, next_pos);
            let direction = next_pos.distance_to(train_pos);
            let distance = direction.length();

            if distance < 0.2 {
                // We'll consider this "arrived"
                engine.velocity = Vector::zero();
            } else {
                engine.velocity = direction; // -f
            }
            engine.acceleration = Vector::zero();
        }
    }
}
