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

    pub fn next_hop_to_dest(&self, self_ent: Entity, from: Entity, dest: Entity, junctions: &ReadStorage<'a, Junction>) -> Option<(u32, Entity)> {
        // Find connection with shortest distance to destination (if destination is reachable)
        let mut min_distance = u32::max_value();
        let mut candidate = None;
        for &next_hop_ent in &self.connections {
            if next_hop_ent == from {
                continue;
            }
            if next_hop_ent == dest && min_distance > 1 {
                min_distance = 1;
                candidate = Some(next_hop_ent);
                break; // Ain't gonna get better than a direct link
            }
            let next_hop = junctions.get(next_hop_ent).unwrap();
            if let Some((distance, _)) = next_hop.next_hop_to_dest(next_hop_ent, self_ent, dest, junctions) {
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

#[derive(Debug, Clone)]
pub struct TrainIsInStation {
    pub station: Entity
}
impl Component for TrainIsInStation {
    type Storage = HashMapStorage<Self>;
}

#[derive(Debug, Clone)]
pub struct TrainWantsToTravelTo {
    pub destination: Entity
}
impl Component for TrainWantsToTravelTo {
    type Storage = HashMapStorage<Self>;
}

#[derive(Debug, Clone)]
pub struct TrainRoute {
    pub hops: Vec<Entity>
}
impl Component for TrainRoute {
    type Storage = HashMapStorage<Self>;
}


#[derive(Debug)]
pub struct TrainRouting {
    pub destination: Entity,
    pub next_hop:    Entity,
    pub coming_from: Entity,
}

impl TrainRouting {
    pub fn with_destination(last: Entity, next: Entity, dest: Entity) -> Self {
        Self {
            destination: dest,
            next_hop:    next,
            coming_from: last,
        }
    }
}

impl Component for TrainRouting {
    type Storage = VecStorage<Self>;
}


pub struct TrainRouter;

impl<'a> System<'a> for TrainRouter {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, TrainIsInStation>,
        WriteStorage<'a, TrainWantsToTravelTo>,
        WriteStorage<'a, TrainRoute>,
        ReadStorage<'a, Junction>,
    );

    fn run(&mut self, sys_data: Self::SystemData) {
        let (
            entities,
            mut trains_in_station,
            mut trains_that_want_to_travel,
            mut routes,
            junctions
        ) = sys_data;
        let mut trains_that_left_the_building = vec![];
        for (train, station, destination) in (&entities, &trains_in_station, &trains_that_want_to_travel).join() {
            // We're coming from station.station -> Entity -> Junction.
            // We wanna go to destination.destination -> Entity -> Junction.
            // station_junction hopefully has connections that have connections to dest_junction.
            fn walk_the_line(
                junctions: &ReadStorage<Junction>,
                prev: Option<Entity>,
                curr: Entity,
                dest: Entity
            ) -> Vec<Entity> {
                let curr_j = junctions.get(curr).unwrap();
                for &next in &curr_j.connections {
                    if prev.is_some() && next == prev.unwrap() {
                        continue;
                    }
                    if next == dest {
                        return vec![dest];
                    }
                    let path_from_next = walk_the_line(junctions, Some(curr), next, dest);
                    if !path_from_next.is_empty() {
                        let mut path_from_here = Vec::with_capacity(path_from_next.len() + 1);
                        path_from_here.push(next);
                        path_from_here.extend_from_slice(&path_from_next);
                        return path_from_here;
                    }
                }
                vec![]
            }
            let path_to_dest = walk_the_line(
                &junctions,
                None,
                station.station,
                destination.destination
            );
            if !path_to_dest.is_empty() {
                println!("Path to enlightenment: {:?}", path_to_dest);
                routes.insert(train, TrainRoute { hops: path_to_dest })
                    .expect("Mission impossible");
                trains_that_left_the_building.push(train);
            }
        }
        for train in trains_that_left_the_building {
            trains_in_station.remove(train);
            trains_that_want_to_travel.remove(train);
        }
    }
}

pub struct TrainRoutingSystem;

impl<'a> System<'a> for TrainRoutingSystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, super::physics::Position>,
        WriteStorage<'a, super::physics::TrainEngine>,
        WriteStorage<'a, TrainRouting>,
        ReadStorage<'a, Junction>,
    );

    fn run(&mut self, (entities, positions, mut engines, mut routings, junctions): Self::SystemData) {
        let mut arrived = vec![];
        for (ent, position, mut engine, mut routing) in (&entities, &positions, &mut engines, &mut routings).join() {
            let next_hop = routing.next_hop;
            let next_junction = junctions.get(next_hop).unwrap();
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
                    arrived.push(ent);
                    continue;
                } else {
                    if let Some((_, next_hop)) =
                        next_junction
                            .next_hop_to_dest(
                                next_hop,
                                routing.coming_from,
                                routing.destination,
                                &junctions
                            )
                    {
                        routing.next_hop = next_hop;
                        routing.coming_from = ent;
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
        for ent in arrived {
            routings.remove(ent);
        }
    }
}
