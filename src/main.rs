extern crate piston_window;
extern crate gfx_device_gl;
extern crate image as im;
extern crate imageproc as imp;
extern crate specs;

// I need a
// https://raw.githubusercontent.com/PistonDevelopers/piston-examples/master/src/paint.rs
// is what I need!

use std::time::SystemTime;
use specs::prelude::*;
use piston_window::*;

mod physics;
mod routing;
mod cargo;
mod map;


#[derive(Default)]
pub struct DeltaTime {
    pub fraction: f64,
    last_updated_at: Option<SystemTime>
}

impl DeltaTime {
    pub fn new() -> Self {
        Self {
            fraction: 0.05,
            last_updated_at: Some(SystemTime::now())
        }
    }

    pub fn update(&mut self) {
        let now = SystemTime::now();
        let dura = now.duration_since(self.last_updated_at.unwrap()).unwrap();
        self.fraction = (dura.as_secs() as f64) + (dura.subsec_micros() as f64 / 1_000_000 as f64);
        self.last_updated_at = Some(now);
    }
}


#[derive(Debug)]
enum RoleKind {
    CoalMine,
    PowerPlant,
    WayPoint,
    Train
}
#[derive(Debug)]
struct Role(RoleKind);

impl Component for Role {
    type Storage = VecStorage<Self>;
}



fn main() {
    let (width, height) = (640, 480);
    let mut window: PistonWindow =
        WindowSettings::new("niart", (width, height))
        .exit_on_esc(true)
        .build()
        .unwrap();

    let mut map = map::Map::new(&mut window, width, height);

    let mut world = World::new();
    world.register::<physics::Position>();
    world.register::<physics::TrainEngine>();
    world.register::<routing::Junction>();
    world.register::<cargo::CargoStorage>();
    world.register::<cargo::CargoProducer>();
    world.register::<cargo::CargoConsumer>();
    world.register::<Role>();

    world.add_resource(DeltaTime::new());

    world.create_entity()
        .with(physics::Position { x: 9.0, y: 14.0 })
        .with(cargo::CargoStorage::new())
        .with(
            cargo::CargoProducer::new()
                .with(cargo::CargoKind::Coal, 0.1)
        )
        .with(Role(RoleKind::CoalMine))
        .with(routing::Junction::new())
        .build();
    world.create_entity()
        .with(physics::Position { x: 590.0, y: 462.5 })
        .with(Role(RoleKind::PowerPlant))
        .with(routing::Junction::new())
        .build();
    world.create_entity()
        .with(physics::Position { x: 590.0, y: 130.0 })
        .with(Role(RoleKind::PowerPlant))
        .with(routing::Junction::new())
        .build();

    let mut dispatcher = DispatcherBuilder::new()
        .with(physics::TrainEngineSystem, "TrainEngineSystem", &[])
        .with(cargo::CargoProductionSystem, "CargoProductionSystem", &[])
        .with(cargo::CargoConsumptionSystem, "CargoConsumptionSystem", &[])
        .build();
    dispatcher.setup(&mut world.res);

    let mut mouse_pos = physics::Position::zero();

    while let Some(evt) = window.next() {
        if let Some(button) = evt.press_args() {
            if button == Button::Mouse(MouseButton::Left) {
                map.start_drawing();
            }
            if button == Button::Mouse(MouseButton::Right) {
                let entities = world.entities();
                let positions = world.read_storage::<physics::Position>();
                let junctions = world.read_storage::<routing::Junction>();
                let lazyupdt = world.read_resource::<LazyUpdate>();
                for (ent, pos, junction) in (&entities, &positions, &junctions).join() {
                    if mouse_pos.distance_length_to(pos) < 10.0 {
                        if !junction.connections.is_empty() {
                            println!("Planting train at junction {:?} heading towards {:?}", ent, junction.connections[0]);
                            lazyupdt.create_entity(&entities)
                                .with(pos.clone())
                                .with(Role(RoleKind::Train))
                                .with(routing::TrainRouting::new())
                                .build();
                        } else {
                            println!("Planting train at junction {:?} is not possible, junction does not have connections", ent);
                        }
                    }
                }
            }
        }
        if let Some(button) = evt.release_args() {
            if button == Button::Mouse(MouseButton::Left) {
                map.stop_drawing();
            }
        }
        if let Some(pos) = evt.mouse_cursor_args() {
            mouse_pos = physics::Position::from(pos);
            map.mouse_moved(pos);
        }

        if let Some(map_event) = map.next_event() {
            if let map::MapEvent::NewRail(from, to) = map_event {
                println!("New rail created! Goes los from {:?} to {:?}", from, to);
                let mut start = None;
                let mut end = None;
                {
                    let entities = world.entities();
                    let positions = world.read_storage::<physics::Position>();
                    let junctions = world.read_storage::<routing::Junction>();
                    for (ent, pos, junction) in (&entities, &positions, &junctions).join() {
                        if from.distance_length_to(pos) < 4.0 {
                            println!("Start is near a junction {:?}! {:?}", ent, junction);
                            start = Some(ent);
                        }
                        if to.distance_length_to(pos) < 4.0 {
                            println!("End is near a junction {:?}! {:?}", ent, junction);
                            end = Some(ent);
                        }
                    }
                }
                if start.is_none() {
                    start = Some(
                        world.create_entity()
                            .with(from)
                            .with(Role(RoleKind::WayPoint))
                            .with(routing::Junction::new())
                            .build()
                    );
                }
                if end.is_none() {
                    end = Some(
                        world.create_entity()
                            .with(to)
                            .with(Role(RoleKind::WayPoint))
                            .with(routing::Junction::new())
                            .build()
                    );
                }
                let start = start.unwrap();
                let end = end.unwrap();
                world.write_storage::<routing::Junction>()
                    .get_mut(start).unwrap()
                    .connections.push(end);
                world.write_storage::<routing::Junction>()
                    .get_mut(end).unwrap()
                    .connections.push(start);
            }
        }

        if let Some(_) = evt.update_args() {
            world.write_resource::<DeltaTime>().update();
            dispatcher.dispatch(&mut world.res);
            world.maintain();
        }

        window.draw_2d(&evt, |c, g, device| {
            clear([1.0; 4], g);
            map.render(c, g, device);

            let positions = world.read_storage::<physics::Position>();
            let roles = world.read_storage::<Role>();
            for (pos, role) in (&positions, &roles).join() {
                ellipse_from_to(
                    match role {
                        Role(RoleKind::CoalMine)   => [1.,  0.,  0.,  1.],
                        Role(RoleKind::PowerPlant) => [0.,  1.,  0.,  1.],
                        Role(RoleKind::Train)      => [0.,  0.,  1.,  1.],
                        Role(RoleKind::WayPoint)   => [0.8, 0.8, 0.,  1.],
                    },
                    [pos.x - 5., pos.y - 5.],
                    [pos.x + 5., pos.y + 5.],
                    c.transform,
                    g
                );
            }
        });
    }
}
