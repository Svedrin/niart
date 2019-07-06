use specs::prelude::*;

use super::physics::Position;
use super::cargo::{CargoStorage, CargoProducer, CargoConsumer, CargoKind};
use super::routing::Junction;
use super::{Role, RoleKind};

pub fn populate(world: &mut World) {
    world.create_entity()
        .with(Position { x: 9.0, y: 14.0 })
        .with(CargoStorage::new())
        .with(
            CargoProducer::new()
                .with(CargoKind::Coal, 0.1)
        )
        .with(Role(RoleKind::CoalMine))
        .with(Junction::new_terminal())
        .build();
    world.create_entity()
        .with(Position { x: 590.0, y: 462.5 })
        .with(Role(RoleKind::PowerPlant))
        .with(Junction::new_terminal())
        .build();
    world.create_entity()
        .with(Position { x: 590.0, y: 130.0 })
        .with(Role(RoleKind::PowerPlant))
        .with(Junction::new_terminal())
        .build();
}
