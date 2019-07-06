use specs::prelude::*;

use super::physics::Position;
use super::routing::{TrainRoute, TrainIsInStation};

#[derive(Clone,Debug,PartialEq)]
pub enum SignalState {
    Dark,
    Halt,
    Slow,
    Go,
}

/**
 * Approach Signals are placed on open road way ahead of a junction, so that if the
 * Junction Signal is red, the driver gets some advance notice. This is necessary
 * because stopping a train takes way longer than stopping a car, and by the time
 * the driver sees the junction signal, it's already too late.
 *
 * Since our trains get to cheat and can just query the Junction Signal Entity for its
 * state instead of physically having to look at it, this signal serves only cosmetic
 * purposes by showing up on the map.
 */
#[derive(Debug, Clone, PartialEq)]
pub struct ApproachSignal {
    pub junction_signal: Entity,
}
impl Component for ApproachSignal {
    type Storage = HashMapStorage<Self>;
}

/**
 * Junction signals are placed directly at a junction. They signal to the driver where
 * the train needs to stop, and when it can continue on its journey.
 */
#[derive(Debug, Clone, PartialEq)]
pub struct JunctionSignal {
    pub signal_state: SignalState,
    pub appr_signals: Vec<Entity>,
}
impl JunctionSignal {
    pub fn new() -> Self {
        Self {
            signal_state: SignalState::Halt,
            appr_signals: vec![]
        }
    }
    pub fn is_halt(&self) -> bool {
        self.signal_state == SignalState::Halt || self.signal_state == SignalState::Dark
    }
}
impl Component for JunctionSignal {
    type Storage = HashMapStorage<Self>;
}

pub struct TrainIsBlockingSignal {
    pub signal: Entity
}
impl Component for TrainIsBlockingSignal {
    type Storage = HashMapStorage<Self>;
}

pub struct SignalIsReservedByTrain {
    pub train: Entity
}
impl Component for SignalIsReservedByTrain {
    type Storage = HashMapStorage<Self>;
}

pub struct SignalIsBlockedByTrain {
    pub train: Entity
}
impl Component for SignalIsBlockedByTrain {
    type Storage = HashMapStorage<Self>;
}

pub struct SpeedLimitFromNextSignal {
    pub vmax: f64
}
impl Component for SpeedLimitFromNextSignal {
    type Storage = HashMapStorage<Self>;
}


const ASSUMED_VMAX:  f64 = 30.0;
const VMAX_FOR_SLOW: f64 = 20.0;
const ASSUMED_AMAX:  f64 =  4.0;


/**
 * In the German railway system, the person controlling signals and directing
 * trains to their destination is called the "Fahrdienstleiter".
 */
pub struct Fahrdienstleiter;

impl<'a> System<'a> for Fahrdienstleiter {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, JunctionSignal>,
        ReadStorage<'a,  TrainRoute>,
        WriteStorage<'a, SignalIsReservedByTrain>,
        WriteStorage<'a, SignalIsBlockedByTrain>,
        ReadStorage<'a,  Position>,
        WriteStorage<'a, SpeedLimitFromNextSignal>,
    );

    fn run(&mut self, sys_data: Self::SystemData) {
        let (
            entities,
            mut junction_signals,
            routes,
            mut reservations,
            blockages,
            positions,
            mut speed_limits_upcoming,
        ) = sys_data;
        // Phase one: Let's go over all'a dem trains and see what we can do for them in terms
        // of signal reservations.
        // Each train that is en route wants to have two reservations: One for the signal where
        // it's currently located (it should have that one inherently), and another one for the
        // signal that comes _after_ the first one, so that the first one can turn green and
        // allow the train to set forth on its journey.
        let mut signals_on_go = vec![];
        for (train, route) in (&entities, &routes).join() {
            let two_signals: Vec<Entity> = route.hops.iter()
                .filter(|&&e| junction_signals.contains(e))
                .take(2)
                .cloned()
                .collect();
            let mut rsvp_count = 0;
            for &signal in &two_signals {
                if let Some(rsvp) = reservations.get(signal) {
                    // Make sure we don't try to reserve the second signal if we don't have a
                    // valid reservation for the first one.
                    if rsvp.train != train {
                        break;
                    }
                } else {
                    reservations
                        .insert(signal, SignalIsReservedByTrain { train: train })
                        .expect("rsvp denied");
                }
                rsvp_count += 1;
            }
            if rsvp_count == 2 {
                // We're clear, allow the first signal to turn green...
                signals_on_go.push(two_signals[0]);
                // ... but let's also see if we need to inflict a speed limit on the train.
                if !speed_limits_upcoming.contains(train) {
                    let distance = positions.get(two_signals[0]).unwrap().distance_to(
                        positions.get(two_signals[1]).unwrap()
                    ).length() - 15.0;
                    // s = v²/a => v² = s*a => v = sqrt(s*a)
                    let vmax = (distance * ASSUMED_AMAX).sqrt();
                    if vmax < ASSUMED_VMAX {
                        let _ = speed_limits_upcoming
                            .insert(train, SpeedLimitFromNextSignal { vmax: vmax });
                    }
                }
            }
        }

        // So now that we have the reservations booked, let's see what those signals need
        // to be telling our trains.
        for (signal, mut signal_s) in (&entities, &mut junction_signals).join() {
            // First of all: If I'm blocked, I'll show red.
            if blockages.contains(signal) {
                signal_s.signal_state = SignalState::Halt;
                continue;
            }

            // I'm not. Do I have reason to still be on?
            // I do if a train has reserved _me_, because I'll need to tell that train what to do.
            // I'll tell them to stop, unless they have a valid reservation for the next signal
            // down the line.
            if let Some(rsvp) = reservations.get(signal) {
                if signals_on_go.contains(&signal) {
                    let slow = speed_limits_upcoming.get(rsvp.train)
                        .and_then(|limit| Some(limit.vmax < VMAX_FOR_SLOW))
                        .unwrap_or(false);
                    if slow {
                        signal_s.signal_state = SignalState::Slow;
                    } else {
                        signal_s.signal_state = SignalState::Go;
                    }
                } else {
                    signal_s.signal_state = SignalState::Halt;
                }
                continue;
            }

            // Looks like I've got nothing to do :)
            signal_s.signal_state = SignalState::Dark;
        }
    }
}

/**
 * Once trains arrived in station, the Fahrdienstleiter doesn't really care about them
 * all that much anymore. Let's have a companion cleaning lady that removes all the
 * reservations those trains held.
 */
pub struct Fahrdienstputzfrau;

impl<'a> System<'a> for Fahrdienstputzfrau {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a,  TrainIsInStation>,
        WriteStorage<'a, TrainIsBlockingSignal>,
        WriteStorage<'a, SignalIsBlockedByTrain>,
        WriteStorage<'a, SignalIsReservedByTrain>,
    );

    fn run(&mut self, sys_data: Self::SystemData) {
        let (
            entities,
            trains_in_station,
            mut train_blockages,
            mut signal_blockages,
            mut signal_reservations,
        ) = sys_data;
        // Clean up signal blockages held by trains that arrived in station.
        for (train, _) in (&entities, &trains_in_station).join() {
            if let Some(blockage) = train_blockages.remove(train) {
                signal_blockages.remove(blockage.signal)
                    .expect("blockage mismatch");
            }
        }
        // Clean up signal reservations held by trains that arrived in station.
        let mut outdated_rsvps = vec![];
        for (signal, rsvp) in (&entities, &signal_reservations).join() {
            if trains_in_station.contains(rsvp.train) {
                outdated_rsvps.push(signal);
            }
        }
        for signal in outdated_rsvps {
            signal_reservations.remove(signal).expect("derp");
        }
    }
}
