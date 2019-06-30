use std::collections::VecDeque;
use specs::prelude::*;

use super::signals::{
    JunctionSignal,
    SignalIsReservedByTrain,
    SignalIsBlockedByTrain,
    TrainIsBlockingSignal,
};

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
    pub hops: VecDeque<Entity>
}
impl TrainRoute {
    pub fn new(path: Vec<Entity>) -> Self {
        Self {
            hops: VecDeque::from(path)
        }
    }
    pub fn next_hop(&self) -> Entity {
        self.hops[0]
    }
    pub fn arrived_at_hop(&mut self) -> Entity {
        self.hops.pop_front().expect("Sad panda")
    }
    pub fn is_empty(&self) -> bool {
        self.hops.is_empty()
    }
}
impl Component for TrainRoute {
    type Storage = HashMapStorage<Self>;
}


/**
 * TrainRouter is on the lookout for trains that are in a station and that intend to travel
 * to some destination. Then it calculates a route and sends the train on the road.
 *
 * In the real world, this is probably done by a dispatcher and/or traffic superintendent.
 * "Dispatcher" is just way too generic a word that I feel comfortable using it here,
 * and the traffic superintendent does way more than just routing. Routing is just their
 * first step, so calling this struct a "TrafficSuperintendent" feels wrong.
 */
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
            junctions,
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
                routes
                    .insert(train, TrainRoute::new(path_to_dest ))
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


/**
 * TrainNavigator sits beside the TrainDriver and decides if we're close enough to the
 * next hop that it makes sense to start thinking one step further.
 * He (or she) also makes sure that our train always correctly announces which signals
 * it's currently interacting with.
 */
pub struct TrainNavigator;

impl<'a> System<'a> for TrainNavigator {
    type SystemData = (
        Entities<'a>,                               // I'm a guy
        ReadStorage<'a, super::physics::Position>,  // I'm somewhere
        WriteStorage<'a, TrainRoute>,               // I need to make sure my damn map is correct
        WriteStorage<'a, TrainIsInStation>,         // I may or may not have gotten somewhere
        WriteStorage<'a, JunctionSignal>,
        WriteStorage<'a, TrainIsBlockingSignal>,
        WriteStorage<'a, SignalIsBlockedByTrain>,
        WriteStorage<'a, SignalIsReservedByTrain>,
    );

    fn run(&mut self, sys_data: Self::SystemData) {
        let (
            entities,
            positions,
            mut routes,
            mut trains_in_station,
            junction_signals,
            mut train_blockages,
            mut signal_blockages,
            mut reservations,
        ) = sys_data;
        let mut arrived_trains = vec![];
        for (train, train_pos, route) in (&entities, &positions, &mut routes).join() {
            let next_pos = positions.get(route.next_hop()).unwrap();
            //println!("I'm in ur {:?}, going to {:?}", train_pos, next_pos);
            let direction = next_pos.distance_to(train_pos);
            let distance = direction.length();
            if distance < 2.0 {
                // We'll consider this "arrived"
                let here = route.arrived_at_hop();
                // The signal that we once approached, we are now blocking
                if junction_signals.contains(here) {
                    if let Some(train_blockage) = train_blockages.remove(train) {
                        signal_blockages.remove(train_blockage.signal);
                    }
                    reservations
                        .remove(here)
                        .expect("couldn't remove registration o_0");
                    signal_blockages
                        .insert(here, SignalIsBlockedByTrain { train: train })
                        .expect("couldn't block next signal");
                    train_blockages
                        .insert(train, TrainIsBlockingSignal { signal: here })
                        .expect("we're doomed, aren't we");
                }
                // are we at the final destination?
                if route.is_empty() {
                    arrived_trains.push(train);
                    trains_in_station
                        .insert(train, TrainIsInStation { station: here })
                        .expect("station is full");
                }
            }
        }
        for train in arrived_trains {
            routes.remove(train);
        }
    }
}
