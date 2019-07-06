extern crate piston_window;
extern crate gfx_device_gl;
extern crate image as im;
extern crate imageproc as imp;
extern crate specs;
extern crate rand;

// I need a
// https://raw.githubusercontent.com/PistonDevelopers/piston-examples/master/src/paint.rs
// is what I need!

use std::time::SystemTime;
use specs::prelude::*;
use piston_window::*;
use rand::seq::IteratorRandom;

mod physics;
mod routing;
mod signals;
mod cargo;
mod world;
mod map;

use world::populate;

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
    DarkSignal,
    RedSignal,
    YellowSignal,
    GreenSignal,
    Train
}
#[derive(Debug)]
struct Role(RoleKind);

impl Component for Role {
    type Storage = VecStorage<Self>;
}


struct SignalRenderer;

impl<'a> System<'a> for SignalRenderer {
    type SystemData = (
        ReadStorage<'a, signals::JunctionSignal>,
        WriteStorage<'a, Role>,
    );

    fn run(&mut self, (signals, mut roles): Self::SystemData) {
        for (signal, mut role) in (&signals, &mut roles).join() {
            role.0 = match signal.signal_state {
                signals::SignalState::Dark => RoleKind::DarkSignal,
                signals::SignalState::Halt => RoleKind::RedSignal,
                signals::SignalState::Slow => RoleKind::YellowSignal,
                signals::SignalState::Go   => RoleKind::GreenSignal,
            };
        }
    }
}



fn main() {
    let mut rng = rand::thread_rng();
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
    world.register::<physics::SpeedLimit>();
    world.register::<routing::Junction>();
    world.register::<signals::JunctionSignal>();
    world.register::<signals::ApproachSignal>();
    world.register::<signals::TrainIsBlockingSignal>();
    world.register::<signals::SpeedLimitFromNextSignal>();
    world.register::<cargo::CargoStorage>();
    world.register::<cargo::CargoProducer>();
    world.register::<cargo::CargoConsumer>();
    world.register::<routing::TrainIsInStation>();
    world.register::<routing::TrainWantsToTravelTo>();
    world.register::<routing::TrainRoute>();
    world.register::<Role>();

    world.add_resource(DeltaTime::new());

    populate(&mut world, &mut map);

    let mut dispatcher = DispatcherBuilder::new()
        .with(physics::TrainEngineSystem, "TrainEngineSystem", &[])
        .with(physics::TrainDriver, "TrainDriver", &[])
        .with(routing::TrainRouter, "TrainRouter", &[])
        .with(routing::TrainNavigator, "TrainNavigator", &[])
        .with(signals::Fahrdienstleiter, "SignalStateCalculator", &[])
        .with(cargo::CargoProductionSystem, "CargoProductionSystem", &[])
        .with(cargo::CargoConsumptionSystem, "CargoConsumptionSystem", &[])
        .with(SignalRenderer, "SignalRenderer", &[])
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
                let mut signals = world.write_storage::<signals::JunctionSignal>();
                let lazyupdt = world.read_resource::<LazyUpdate>();
                for (junction, junction_pos, junction_j) in (&entities, &positions, &junctions).join() {
                    if mouse_pos.distance_length_to(junction_pos) < 10.0 {
                        if junction_j.is_terminal {
                            // Choose any random terminal and leave it to the Router to figure out if this works
                            let dest = (&entities, &junctions).join()
                                .filter(|(e, _j)| *e != junction)
                                .filter(|(_e, j)| j.is_terminal)
                                .map(|(e, _j)| e)
                                .choose(&mut rng);
                            if let Some(destination) = dest {
                                println!("Planting train at junction {:?} heading towards {:?}", junction, destination);
                                lazyupdt.create_entity(&entities)
                                    .with(junction_pos.clone())
                                    .with(Role(RoleKind::Train))
                                    .with(routing::TrainIsInStation { station: junction })
                                    .with(routing::TrainWantsToTravelTo { destination: destination })
                                    .with(physics::TrainEngine {
                                        velocity: physics::Vector { x: 0., y: 0. },
                                        acceleration: physics::Vector { x: 0., y: 0. },
                                        vmax: 30.0,
                                        amax: 5.0
                                    })
                                    .build();
                            } else {
                                println!("Planting train at junction {:?} is not possible, junction does not have connections", junction);
                            }
                        } else {
                            println!("We should make a signal at {:?}", junction);
                            signals
                                .insert(junction, signals::JunctionSignal::new())
                                .expect("Sad signalling panda");
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
                        Role(RoleKind::CoalMine)     => [0.2, 0.2, 0.2, 1.],
                        Role(RoleKind::PowerPlant)   => [0.7, 0.7, 0.7, 1.],
                        Role(RoleKind::Train)        => [0.,  0.,  1.,  1.],
                        Role(RoleKind::DarkSignal)   => [0.,  0.,  0.,  1.],
                        Role(RoleKind::RedSignal)    => [1.,  0.,  0.,  1.],
                        Role(RoleKind::YellowSignal) => [0.9, 0.9, 0.,  1.],
                        Role(RoleKind::GreenSignal)  => [0.,  1.,  0.,  1.],
                        Role(RoleKind::WayPoint)     => [0.7, 0.,  0.7, 1.],
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
