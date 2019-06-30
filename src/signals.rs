use specs::prelude::*;

#[derive(Clone,Debug,PartialEq)]
pub enum RailBlockState {
    Open,
    Occupied(Entity),
}

#[derive(Clone,Debug,PartialEq)]
pub enum SignalState {
    Off,
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
    pub block_state: RailBlockState,
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
    pub block_state:  RailBlockState,
    pub signal_state: SignalState,
    pub appr_signals: Vec<Entity>,
}
impl JunctionSignal {
    pub fn new() -> Self {
        Self {
            block_state:  RailBlockState::Open,
            signal_state: SignalState::Halt,
            appr_signals: vec![]
        }
    }
}
impl Component for JunctionSignal {
    type Storage = HashMapStorage<Self>;
}


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
