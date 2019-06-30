use specs::prelude::*;

#[derive(Clone,Debug,PartialEq)]
pub enum RailBlockState {
    Open,
    Occupied(Entity),
}

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
    pub fn is_halt(&self) -> bool {
        self.signal_state == SignalState::Halt
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


pub struct SignalStateCalculator;

impl<'a> System<'a> for SignalStateCalculator {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, JunctionSignal>,
        ReadStorage<'a, TrainIsApproachingSignal>,
        ReadStorage<'a, TrainIsInBlockOfSignal>,
    );

    fn run(&mut self, (entities, mut jsignals, trains_approaching, trains_blocking): Self::SystemData) {
        for (signal, mut signal_s) in (&entities, &mut jsignals).join() {
            // See what the world is doing
            let approaching_train = (&trains_approaching).join()
                .filter(|train_t| train_t.signal == signal)
                .nth(0);

            let blocking_train = (&entities, &trains_blocking).join()
                .filter(|(bt, bt_bt)| bt_bt.signal == signal)
                .map(|(bt, bt_bt)| bt)
                .nth(0);

            // Now decide what this signal's state should be
            if let Some(train) = blocking_train {
                signal_s.block_state = RailBlockState::Occupied(train);
                signal_s.signal_state = SignalState::Halt;
            } else if let Some(train) = approaching_train {
                signal_s.block_state = RailBlockState::Open;
                signal_s.signal_state = SignalState::Go;
            } else {
                signal_s.block_state = RailBlockState::Open;
                signal_s.signal_state = SignalState::Dark;
            }
        }
    }
}
