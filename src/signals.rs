use specs::prelude::*;

use super::routing::TrainRoute;

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
    pub occupied_by:  Option<Entity>,
    pub reserved_for: Option<Entity>,
    pub appr_signals: Vec<Entity>,
}
impl JunctionSignal {
    pub fn new() -> Self {
        Self {
            signal_state: SignalState::Halt,
            occupied_by:  None,
            reserved_for: None,
            appr_signals: vec![]
        }
    }
    pub fn is_halt(&self) -> bool {
        self.signal_state == SignalState::Halt || self.signal_state == SignalState::Dark
    }
    pub fn passed_by(&mut self, train: Entity) {
        assert_eq!(self.reserved_for, Some(train));
        assert_eq!(self.occupied_by,  None);
        self.reserved_for = None;
        self.occupied_by = Some(train);
    }
}
impl Component for JunctionSignal {
    type Storage = HashMapStorage<Self>;
}

/**
 * These two components exist for a train to announce what it thinks it's doing.
 * Note that signals have their own idea of what's going on, and a signal will
 * only turn green if what the trains announce matches its own understanding.
 */
pub struct TrainIsApproachingSignal {
    pub signal: Entity
}
impl Component for TrainIsApproachingSignal {
    type Storage = HashMapStorage<Self>;
}


pub struct TrainIsInBlockOfSignal {
    pub signal: Entity
}
impl Component for TrainIsInBlockOfSignal {
    type Storage = HashMapStorage<Self>;
}


pub struct Fahrdienstleiter;

impl<'a> System<'a> for Fahrdienstleiter {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, JunctionSignal>,
        ReadStorage<'a, TrainIsApproachingSignal>,
        ReadStorage<'a, TrainIsInBlockOfSignal>,
        ReadStorage<'a, TrainRoute>,
    );

    fn run(&mut self, (entities, mut jsignals, trains_approaching, trains_blocking, routes): Self::SystemData) {
        // Unfortunately, Rust's borrowing semantics forbid me from joining the Signals in here,
        // because I need to access two signals at once, so I'd borrow the store twice.
        for signal in (&entities).join() {
            if !jsignals.contains(signal) { // this is not a signal
                continue;
            }

            // See what the world is doing
            let approaching_train = (&entities, &trains_approaching).join()
                .filter(|(_, ta)| ta.signal == signal)
                .map(|(t, _)| t)
                .nth(0);

            let blocking_train = (&entities, &trains_blocking).join()
                .filter(|(_, bt_bt)| bt_bt.signal == signal)
                .map(|(bt, _)| bt)
                .nth(0);

            // Now decide what this signal's state should be.
            // First see if we need to make a reservation at the next signal on a train's route
            let mut reserved_for_us = false;
            if let Some(approaching_train) = approaching_train {
                let route = routes.get(approaching_train).unwrap();
                assert_eq!(route.hops[0], signal);
                for &hop in route.hops.iter().skip(1) {
                    if let Some(mut next_signal_j) = jsignals.get_mut(hop) {
                        if let Some(reserved_for_train) = next_signal_j.reserved_for {
                            reserved_for_us = reserved_for_train == approaching_train;
                        } else {
                            next_signal_j.reserved_for = Some(approaching_train);
                            reserved_for_us = true;
                        }
                        break;
                    }
                }
            }

            // Now see if we're blocked by someone
            let signal_s = jsignals.get_mut(signal).unwrap();
            if let Some(train) = blocking_train {
                if signal_s.occupied_by.is_none() {
                    signal_s.occupied_by = Some(train);
                } else {
                    assert_eq!(signal_s.occupied_by, Some(train));
                }
            } else {
                signal_s.occupied_by = None;
            }

            if signal_s.reserved_for.is_some() || signal_s.occupied_by.is_some() {
                if !signal_s.occupied_by.is_some() && reserved_for_us {
                    signal_s.signal_state = SignalState::Go;
                }
                else {
                    signal_s.signal_state = SignalState::Halt;
                }
            } else {
                signal_s.signal_state = SignalState::Dark;
            }
        }
    }
}
