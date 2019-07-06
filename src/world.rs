use specs::prelude::*;

use super::map::Map;
use super::physics::Position;
use super::cargo::{CargoStorage, CargoProducer, CargoConsumer, CargoKind};
use super::routing::Junction;
use super::signals::JunctionSignal;
use super::{Role, RoleKind};

fn connect_junctions(world: &mut World, map: &mut Map, left: Entity, right: Entity) {
    let mut junctions = world.write_storage::<Junction>();
    junctions.get_mut(left).unwrap().connections.push(right);
    junctions.get_mut(right).unwrap().connections.push(left);

    let positions = world.read_storage::<Position>();
    map.draw_rails_at(
        positions.get(left).unwrap(),
        positions.get(right).unwrap()
    );
}

pub fn populate(world: &mut World, map: &mut Map) {
    let coal_mine = world.create_entity()
        .with(Position::new(10.0, 15.0))
        .with(CargoStorage::new())
        .with(
            CargoProducer::new()
                .with(CargoKind::Coal, 0.1)
        )
        .with(Role(RoleKind::CoalMine))
        .with(Junction::new_terminal())
        .build();
    let bottom_power_plant = world.create_entity()
        .with(Position::new(600.0, 460.0))
        .with(Role(RoleKind::PowerPlant))
        .with(Junction::new_terminal())
        .build();
    let top_power_plant = world.create_entity()
        .with(Position::new(600.0, 180.0))
        .with(Role(RoleKind::PowerPlant))
        .with(Junction::new_terminal())
        .build();

    // Add one junction in front of each of our terminals
    let j_cm = world.create_entity()
        .with(Position::new(20.0, 25.0))
        .with(Junction::new())
        .with(Role(RoleKind::WayPoint))
        .build();

    connect_junctions(world, map, j_cm, coal_mine);

    let j_bpp = world.create_entity()
        .with(Position::new(590.0, 450.0))
        .with(Junction::new())
        .with(Role(RoleKind::WayPoint))
        .build();

    connect_junctions(world, map, j_bpp, bottom_power_plant);

    let j_tpp = world.create_entity()
        .with(Position::new(590.0, 190.0))
        .with(Junction::new())
        .with(Role(RoleKind::WayPoint))
        .build();

    connect_junctions(world, map, j_tpp, top_power_plant);

    // Build a two-way track between the coal mine and the bottom power plant
    // Half-way in between, we add some junctions to connect a side-track that
    // goes to the top power plant
    // Track CoalMine -> BPP
    let j_1 = world.create_entity()
        .with(Position::new(20.0, 35.0))
        .with(Junction::new())
        .with(Role(RoleKind::WayPoint))
        .build();

    connect_junctions(world, map, j_cm, j_1);

    let j_2 = world.create_entity()
        .with(Position::new(140.0, 160.0))
        .with(Junction::new())
        .with(Role(RoleKind::WayPoint))
        .with(JunctionSignal::new())
        .build();

    connect_junctions(world, map, j_1, j_2);

    let j_3 = world.create_entity()
        .with(Position::new(190.0, 200.0))
        .with(Junction::new())
        .with(Role(RoleKind::WayPoint))
        .with(JunctionSignal::new())
        .build();

    connect_junctions(world, map, j_2, j_3);

    let j_4 = world.create_entity()
        .with(Position::new(580.0, 450.0))
        .with(Junction::new())
        .with(Role(RoleKind::WayPoint))
        .with(JunctionSignal::new())
        .build();

    connect_junctions(world, map, j_3, j_4);
    connect_junctions(world, map, j_4, j_bpp);

    // Track BPP -> CoalMine, but counting reverse (so 5 is next to 1, 6->2, 7->3, 8->4)
    let j_5 = world.create_entity()
        .with(Position::new(30.0, 25.0))
        .with(Junction::new())
        .with(Role(RoleKind::WayPoint))
        .with(JunctionSignal::new())
        .build();

    connect_junctions(world, map, j_cm, j_5);

    let j_6 = world.create_entity()
        .with(Position::new(150.0, 150.0))
        .with(Junction::new())
        .with(Role(RoleKind::WayPoint))
        .with(JunctionSignal::new())
        .build();

    connect_junctions(world, map, j_5, j_6);

    let j_7 = world.create_entity()
        .with(Position::new(200.0, 190.0))
        .with(Junction::new())
        .with(Role(RoleKind::WayPoint))
        .with(JunctionSignal::new())
        .build();

    connect_junctions(world, map, j_6, j_7);

    let j_8 = world.create_entity()
        .with(Position::new(590.0, 440.0))
        .with(Junction::new())
        .with(Role(RoleKind::WayPoint))
        .build();

    connect_junctions(world, map, j_7, j_8);
    connect_junctions(world, map, j_8, j_bpp);
}
