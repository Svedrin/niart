use specs::prelude::*;

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
                if position != next_pos {
                    let dist = next_pos.distance_to(position);
                    let distlen = dist.length();
                    if distlen < 0.1 {
                        // We'll consider this "arrived"
                        engine.velocity = super::physics::Vector::zero();
                        routing.next_hop = None;
                        routing.destination = None;
                    }
                    if distlen < engine.vmax {
                        engine.velocity = dist;
                    } else {
                        engine.velocity = dist.scale_to_length(engine.vmax);
                    }
                }
            }
        }
    }
}
