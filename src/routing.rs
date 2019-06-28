use specs::prelude::*;

use super::physics::Vector;

#[derive(Debug, Clone)]
pub struct Junction {
    pub connections: Vec<Entity>
}

impl Junction {
    pub fn new() -> Self {
        Self {
            connections: vec![]
        }
    }
}

impl Component for Junction {
    type Storage = VecStorage<Self>;
}


#[derive(Debug)]
pub struct TrainRouting {
    pub destination: Option<Entity>,
    pub next_hop:    Option<Entity>,
}

impl TrainRouting {
    pub fn new() -> Self {
        Self {
            destination: None,
            next_hop:    None,
        }
    }

    pub fn with_destination(dest: Entity, next: Entity) -> Self {
        Self {
            destination: Some(dest),
            next_hop:    Some(next),
        }
    }
}

impl Component for TrainRouting {
    type Storage = VecStorage<Self>;
}

pub struct TrainRoutingSystem;


impl<'a> System<'a> for TrainRoutingSystem {
    type SystemData = (
        ReadStorage<'a, super::physics::Position>,
        WriteStorage<'a, super::physics::TrainEngine>,
        WriteStorage<'a, TrainRouting>,
        Read<'a, super::DeltaTime>,
    );

    fn run(&mut self, (positions, mut engines, mut routing, delta): Self::SystemData) {
        for (position, mut engine, mut routing) in (&positions, &mut engines, &mut routing).join() {
            if let Some(next_hop) = routing.next_hop {
                let next_pos = positions.get(next_hop).unwrap();
                println!("I'm in ur {:?}, going to {:?}", position, next_pos);
                let direction = next_pos.distance_to(position);
                let distance = direction.length();
                if distance < 0.1 {
                    // We'll consider this "arrived"
                    engine.velocity = Vector::zero();
                    engine.acceleration = Vector::zero();
                    routing.next_hop = None;
                    routing.destination = None;
                    continue;
                }

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
        }
    }
}
