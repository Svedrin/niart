use std::collections::VecDeque;
use specs::prelude::*;

use super::physics::SpeedLimit;
use super::signals::{
    JunctionSignal,
    SignalIsReservedByTrain,
    SignalIsBlockedByTrain,
    TrainIsBlockingSignal,
    SpeedLimitFromNextSignal,
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
        ReadStorage<'a, JunctionSignal>,
    );

    fn run(&mut self, sys_data: Self::SystemData) {
        let (
            entities,
            mut trains_in_station,
            mut trains_that_want_to_travel,
            mut routes,
            junctions,
            signals,
        ) = sys_data;
        let mut trains_that_left_the_building = vec![];
        for (train, station, destination) in (&entities, &trains_in_station, &trains_that_want_to_travel).join() {
            // We're coming from station.station -> Entity -> Junction.
            // We wanna go to destination.destination -> Entity -> Junction.
            // station_junction hopefully has connections that have connections to dest_junction.
            // Note that we build the path in reverse because it saves a ton of copying.
            fn walk_the_line(
                junctions: &ReadStorage<Junction>,
                signals: &ReadStorage<JunctionSignal>,
                prev: Option<Entity>,
                curr: Entity,
                dest: Entity,
                ttl: u8
            ) -> Vec<Entity> {
                // Infinite loop protection
                if ttl == 0 {
                    return vec![];
                }
                let curr_j = junctions.get(curr).unwrap();
                // When looking for a path to the destination, make two passes.
                // It may or may not be the case, that there are two paths forward that
                // lead to our destination: One goes via a signal, the other one doesn't.
                // This frequently happens when we're leaving a station: We arrived at a
                // junction where the incoming rail joins the outgoing one, and we'd like
                // to keep that part of the rails free. Since usually the outgoing path
                // does not have a signal while the incoming one does, let's see if we
                // can avoid taking that route and still get to our destination.
                // If we cannot find a path when ignoring those that go via signals, we
                // make another pass that does not have this constraint. If that pass
                // finds a path, we reluctantly take it because it's really the only
                // option we have.
                for &i_hate_signals in &[true, false] {
                    for &next in &curr_j.connections {
                        if prev.is_some() && next == prev.unwrap() {
                            continue;
                        }
                        if i_hate_signals && signals.contains(next) {
                            continue;
                        }
                        if next == dest {
                            return vec![dest];
                        }
                        let mut path_from_next = walk_the_line(
                            junctions, signals,
                            Some(curr), next, dest,
                            ttl - 1
                        );
                        if !path_from_next.is_empty() {
                            path_from_next.push(next);
                            return path_from_next;
                        }
                    }
                }
                vec![]
            }
            let mut path_to_dest = walk_the_line(
                &junctions,
                &signals,
                None,
                station.station,
                destination.destination,
                32
            );
            if !path_to_dest.is_empty() {
                path_to_dest.reverse();
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
        WriteStorage<'a, SpeedLimitFromNextSignal>,
        WriteStorage<'a, SpeedLimit>,
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
            mut speed_limits_upcoming,
            mut speed_limits_current,
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
                    let _ = speed_limits_current.remove(train);
                    if let Some(speed_limit) = speed_limits_upcoming.remove(train) {
                        let _ = speed_limits_current.insert(
                            train,
                            SpeedLimit { vmax: speed_limit.vmax }
                        );
                    }
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
