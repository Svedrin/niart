use specs::prelude::*;
use std::collections::HashMap;

#[derive(Debug,PartialEq,Eq,Hash,Clone,Copy)]
pub enum CargoKind {
    Coal,
    Oil,
    Fuel,
    Grain,
    Livestock,
    Logs,
    Planks,
}

#[derive(Debug)]
pub struct CargoStorage {
    pub quantities: HashMap<CargoKind, f64>,
}

impl CargoStorage {
    pub fn new() -> Self {
        Self {
            quantities: HashMap::new()
        }
    }
}

impl Component for CargoStorage {
    type Storage = VecStorage<Self>;
}


#[derive(Debug)]
pub struct CargoProducer {
    pub quantities: HashMap<CargoKind, f64>,
}

impl CargoProducer {
    pub fn new() -> Self {
        Self {
            quantities: HashMap::new()
        }
    }

    pub fn from_array(production: &[(CargoKind, f64)]) -> Self {
        Self {
            quantities: production.iter().cloned().collect()
        }
    }
}

impl Component for CargoProducer {
    type Storage = VecStorage<Self>;
}


#[derive(Debug)]
pub struct CargoConsumer {
    pub quantities: HashMap<CargoKind, f64>,
}

impl Component for CargoConsumer {
    type Storage = VecStorage<Self>;
}


pub struct CargoProductionSystem;

impl<'a> System<'a> for CargoProductionSystem {
    type SystemData = (
        WriteStorage<'a, CargoStorage>,
        ReadStorage<'a, CargoProducer>,
        Read<'a, super::DeltaTime>,
    );

    fn run(&mut self, (mut storages, producers, delta): Self::SystemData) {
        for (storage, producer) in (&mut storages, &producers).join() {
            for (kind, quantity) in &producer.quantities {
                let stored_qty = storage.quantities.entry(*kind).or_insert(0.0);
                *stored_qty += quantity * delta.fraction;
            }
        }
    }
}


pub struct CargoConsumptionSystem;

impl<'a> System<'a> for CargoConsumptionSystem {
    type SystemData = (
        WriteStorage<'a, CargoStorage>,
        ReadStorage<'a, CargoConsumer>,
        Read<'a, super::DeltaTime>,
    );

    fn run(&mut self, (mut storages, producers, delta): Self::SystemData) {
        for (storage, producer) in (&mut storages, &producers).join() {
            for (kind, quantity) in &producer.quantities {
                let stored_qty = storage.quantities.entry(*kind).or_insert(0.0);
                if *stored_qty > 0.0 {
                    *stored_qty -= quantity * delta.fraction;
                }
            }
        }
    }
}
