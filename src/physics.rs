use specs::prelude::*;

use super::routing::{TrainRoute,TrainIsInStation};
use super::signals::{JunctionSignal, SpeedLimitFromNextSignal};

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


pub struct SpeedLimit {
    pub vmax: f64
}
impl Component for SpeedLimit {
    type Storage = HashMapStorage<Self>;
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
        Entities<'a>,                      // I'm a guy
        ReadStorage<'a, Position>,         // I'm somewhere
        WriteStorage<'a, TrainEngine>,     // I haz an engine that I can play with
        ReadStorage<'a, TrainRoute>,       // I wanna go somewhere
        ReadStorage<'a, TrainIsInStation>, // or I'm in a station
        ReadStorage<'a, JunctionSignal>,   // and I may be looking at a signal
        ReadStorage<'a, SpeedLimit>,
        ReadStorage<'a, SpeedLimitFromNextSignal>,
    );

    fn run(&mut self, sys_data: Self::SystemData) {
        let (
            entities,
            positions,
            mut engines,
            routes,
            trains_in_station,
            junction_signals,
            speed_limits_current,
            speed_limits_upcoming,
        ) = sys_data;
        // Open Road
        for (train, train_pos, mut engine, route) in (&entities, &positions, &mut engines, &routes).join() {
            let next_pos = positions.get(route.next_hop()).unwrap();
            //println!("I'm in ur {:?}, going to {:?}", train_pos, next_pos);
            let direction = next_pos.distance_to(train_pos);

            // Decide where it is that we need to stop next.
            // If we're approaching a signal and that signal shows red, we'll need
            // to stop some ways away in front of it, so that our Navigator doesn't
            // conclude we passed it already and people don't get uncomfortable.
            let distance = direction.length();

            // Make sure we're going in the right direction.
            engine.velocity = direction.scale_to_length(engine.velocity.length());

            // What speed should we be going?
            let v_target =
                // Has Fahrdienstleiter told us anything?
                if let Some(limit) = speed_limits_current.get(train) {
                    limit.vmax
                } else {
                    engine.vmax
                };

            // if we're doing more than that already, no need to bother with anything else -> brake
            if engine.velocity.length() > v_target {
                engine.acceleration = direction.scale_to_length(-engine.amax);
                continue;
            }

            // Let's see if going vmax is safe. It is if we can stop in time for the next signal.
            // Find out if that means braking to zero, or if we just need to slow down.
            let v_upcoming =
                if junction_signals
                    .get(route.next_hop())
                    .map_or(false, |sig| sig.is_halt()) {
                    // First, we look at the signal. If signal says stop, we stop.
                    0.0
                } else if let Some(limit) = speed_limits_upcoming.get(train) {
                    // Then we ask the Fahrdienstleiter.
                    limit.vmax
                } else {
                    // If all else fails, yolo
                    engine.vmax
                };

            // If we're going that speed or below, no need to worry.
            // Otherwise, we need to act.
            if engine.velocity.length() > v_upcoming {
                // Ok, it seems we need to slow down in time for the next signal.
                // Let's find out how far that is away.
                // 1. Number of accelerations I need to brake away is velocity/acceleration;
                //    since acceleration = change in velocity per second, this is in seconds
                // 2. how far I'm travelling in that time is given by the velocity
                // 3. thus t = v/a, s = v * t -> s = v²/a
                //    if we're closer than this distance, brake furiously
                let braking_distance = ((engine.velocity.length() - v_upcoming).powi(2) / engine.amax) + 12.0;
                if distance < braking_distance {
                    engine.acceleration = direction.scale_to_length(-engine.amax);
                    continue;
                }
            }

            // Ok, no need to brake. Let's see if we want to accelerate.
            if engine.velocity.length() < f64::min(v_target, v_upcoming) {
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
