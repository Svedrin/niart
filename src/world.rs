use specs::prelude::*;

use super::map::Map;
use super::physics::Position;
use super::cargo::{CargoStorage, CargoProducer, CargoKind};
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
        .with(Position::new(60.0, 65.0))
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
        .with(Position::new(600.0, 130.0))
        .with(Role(RoleKind::PowerPlant))
        .with(Junction::new_terminal())
        .build();

    let _unconnected_power_plant_1 = world.create_entity()
        .with(Position::new(500.0, 300.0))
        .with(Role(RoleKind::PowerPlant))
        .with(Junction::new_terminal())
        .build();

    let _unconnected_power_plant_2 = world.create_entity()
        .with(Position::new(100.0, 430.0))
        .with(Role(RoleKind::PowerPlant))
        .with(Junction::new_terminal())
        .build();

    // Add one junction in front of each of our terminals
    let j_cm = world.create_entity()
        .with(Position::new(70.0, 75.0))
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
        .with(Position::new(570.0, 150.0))
        .with(Junction::new())
        .with(Role(RoleKind::WayPoint))
        .build();

    connect_junctions(world, map, j_tpp, top_power_plant);

    // Build a two-way track between the coal mine and the bottom power plant
    // Half-way in between, we add some junctions to connect a side-track that
    // goes to the top power plant
    // Track CoalMine -> BPP
    let j_1 = world.create_entity()
        .with(Position::new(70.0, 85.0))
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
        .with(Position::new(80.0, 75.0))
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

    // Side Track: Coal Mine -> TPP (split off from CM->BPP track at j_2)

    let j_21 = world.create_entity()
        .with(Position::new(170.0, 220.0))
        .with(Junction::new())
        .with(Role(RoleKind::WayPoint))
        .with(JunctionSignal::new())
        .build();

    connect_junctions(world, map, j_2, j_21);

    let j_22 = world.create_entity()
        .with(Position::new(220.0, 240.0))
        .with(Junction::new())
        .with(Role(RoleKind::WayPoint))
        .build();

    connect_junctions(world, map, j_21, j_22);

    let j_23 = world.create_entity()
        .with(Position::new(330.0, 240.0))
        .with(Junction::new())
        .with(Role(RoleKind::WayPoint))
        .build();

    connect_junctions(world, map, j_22, j_23);

    let j_24 = world.create_entity()
        .with(Position::new(520.0, 180.0))
        .with(Junction::new())
        .with(Role(RoleKind::WayPoint))
        .with(JunctionSignal::new())
        .build();

    connect_junctions(world, map, j_23, j_24);
    connect_junctions(world, map, j_24, j_tpp);

    // Side Track: TPP -> Coal Mine (merged with BPP->CM track at j_6)

    let j_31 = world.create_entity()
        .with(Position::new(220.0, 170.0))
        .with(Junction::new())
        .with(Role(RoleKind::WayPoint))
        .with(JunctionSignal::new())
        .build();

    connect_junctions(world, map, j_6, j_31);

    // TODO: This is the perfect place for an approach signal, because trains can't stop at 21
    // if they need to because they're way too fast. if they slowed down here, life would be good.
    let j_32 = world.create_entity()
        .with(Position::new(270.0, 190.0))
        .with(Junction::new())
        .with(Role(RoleKind::WayPoint))
        .build();

    connect_junctions(world, map, j_31, j_32);

    let j_33 = world.create_entity()
        .with(Position::new(330.0, 190.0))
        .with(Junction::new())
        .with(Role(RoleKind::WayPoint))
        .build();

    connect_junctions(world, map, j_32, j_33);

    let j_34 = world.create_entity()
        .with(Position::new(500.0, 160.0))
        .with(Junction::new())
        .with(Role(RoleKind::WayPoint))
        .build();

    connect_junctions(world, map, j_33, j_34);
    connect_junctions(world, map, j_34, j_tpp);
}
