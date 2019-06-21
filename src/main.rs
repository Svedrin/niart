extern crate piston_window;
extern crate gfx_device_gl;
extern crate image as im;
extern crate vecmath;
extern crate specs;

// I need a
// https://raw.githubusercontent.com/PistonDevelopers/piston-examples/master/src/paint.rs
// is what I need!

use std::time::SystemTime;
use specs::prelude::*;
use piston_window::*;

mod physics;
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
    world.register::<cargo::CargoStorage>();
    world.register::<cargo::CargoProducer>();
    world.register::<cargo::CargoConsumer>();
    world.register::<Role>();

    world.add_resource(DeltaTime::new());

    world.create_entity()
        .with(physics::Position { x: 9.0, y: 14.0 })
        .with(cargo::CargoStorage::new())
        .with(cargo::CargoProducer::from_array(&[(cargo::CargoKind::Coal, 10.0)]))
        .with(Role(RoleKind::CoalMine))
    .build();
    world.create_entity()
        .with(physics::Position { x: 590.0, y: 462.5 })
        .with(Role(RoleKind::PowerPlant))
    .build();
    world.create_entity()
        .with(physics::Position { x: 10.0, y: 12.5 })
        .with(physics::TrainEngine {
            velocity:     physics::Coords { x:  30.0, y: 15. },
            acceleration: physics::Coords { x: -3., y: 0. },
            vmin: 0.0,
            vmax: 10.0
        })
        .with(Role(RoleKind::Train))
    .build();

    let mut dispatcher = DispatcherBuilder::new()
        .with(physics::TrainEngineSystem, "TrainEngineSystem", &[])
        .with(cargo::CargoProductionSystem, "CargoProductionSystem", &[])
        .with(cargo::CargoConsumptionSystem, "CargoConsumptionSystem", &[])
        .build();
    dispatcher.setup(&mut world.res);

    while let Some(evt) = window.next() {
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
                        Role(RoleKind::CoalMine)   => [1., 0., 0., 1.],
                        Role(RoleKind::PowerPlant) => [0., 1., 0., 1.],
                        Role(RoleKind::Train)      => [0., 0., 1., 1.],
                    },
                    [pos.x - 5., pos.y - 5.],
                    [pos.x + 5., pos.y + 5.],
                    c.transform,
                    g
                );
            }
        });
        if let Some(button) = evt.press_args() {
            if button == Button::Mouse(MouseButton::Left) {
                map.start_drawing();
            }
        }
        if let Some(button) = evt.release_args() {
            if button == Button::Mouse(MouseButton::Left) {
                map.stop_drawing();
            }
        }
        if let Some(pos) = evt.mouse_cursor_args() {
            map.mouse_moved(pos);
        }
    }
}
