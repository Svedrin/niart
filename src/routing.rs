use specs::prelude::*;

use super::physics::Vector;

#[derive(Debug, Clone, PartialEq)]
pub struct Junction {
    pub connections: Vec<Entity>,
    pub is_terminal: bool,
}

impl Junction {
    pub fn new() -> Self {
        Self {
            connections: vec![],
            is_terminal: false,
        }
    }

    pub fn new_terminal() -> Self {
        Self {
            connections: vec![],
            is_terminal: true,
        }
    }
}

impl<'a> Junction {
    pub fn find_any_other_terminal(&self, junctions: &ReadStorage<'a, Junction>) -> Option<(Entity, Entity)> {
        for &next_hop_ent in &self.connections {
            let next_hop = junctions.get(next_hop_ent).unwrap();
            if next_hop.is_terminal {
                return Some((next_hop_ent, next_hop_ent));
            }
            if let Some((_, dest_ent)) = next_hop.find_any_other_terminal(junctions) {
                return Some((next_hop_ent, dest_ent));
            }
        }
        None
    }

    pub fn next_hop_to_dest(&self, from: &Junction, dest: Entity, junctions: &ReadStorage<'a, Junction>) -> Option<(u32, Entity)> {
        // Find connection with shortest distance to destination (if destination is reachable)
        let mut min_distance = u32::max_value();
        let mut candidate = None;
        for &next_hop_ent in &self.connections {
            if next_hop_ent == dest && min_distance > 1 {
                min_distance = 1;
                candidate = Some(next_hop_ent);
                break; // Ain't gonna get better than a direct link
            }
            let next_hop = junctions.get(next_hop_ent).unwrap();
            if next_hop == from {
                continue;
            }
            if let Some((distance, _)) = next_hop.next_hop_to_dest(&self, dest, junctions) {
                if distance + 1 < min_distance {
                    min_distance = distance + 1;
                    candidate = Some(next_hop_ent);
                }
            }
        }
        if let Some(candidate) = candidate {
            return Some((min_distance, candidate));
        }
        None
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

    pub fn with_destination(next: Entity, dest: Entity) -> Self {
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
        ReadStorage<'a, Junction>,
    );

    fn run(&mut self, (positions, mut engines, mut routing, junctions): Self::SystemData) {
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
                    // are we at the final destination?
                    if routing.next_hop == routing.destination {
                        routing.next_hop = None;
                        routing.destination = None;
                    } else {
                        let next_hop_junction = junctions.get(next_hop).unwrap();
                        if let Some((_, next_hop)) =
                            next_hop_junction
                                .next_hop_to_dest(
                                    next_hop_junction, // this should be our previous hop, but we don't know it anymore
                                    routing.destination.unwrap(),
                                    &junctions
                                )
                        {
                            routing.next_hop = Some(next_hop);
                        } else {
                            routing.next_hop = None;
                        }
                    }
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
