extern crate piston_window;
extern crate image;
extern crate vecmath;
extern crate specs;

// I need a
// https://raw.githubusercontent.com/PistonDevelopers/piston-examples/master/src/paint.rs
// is what I need!

use std::time::{SystemTime, Duration};
use piston_window::*;
use image::{ImageBuffer,Rgba};
use specs::prelude::*;
use vecmath::*;

mod physics;
mod cargo;


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

    let mut canvas = ImageBuffer::new(width, height);
    let mut draw = false;
    let mut texture_context = window.create_texture_context();
    let mut texture: G2dTexture = Texture::from_image(
        &mut texture_context,
        &canvas,
        &TextureSettings::new()
    ).unwrap();

    let mut last_pos: Option<[f64; 2]> = None;

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
            // Update texture before rendering.
            texture.update(&mut texture_context, &canvas).unwrap();
            texture_context.encoder.flush(device);

            clear([1.0; 4], g);
            image(&texture, c.transform, g);

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
                draw = true;
            }
        };
        if let Some(button) = evt.release_args() {
            if button == Button::Mouse(MouseButton::Left) {
                draw = false;
                last_pos = None
            }
        };
        if draw {
            if let Some(pos) = evt.mouse_cursor_args() {
                let (x, y) = (pos[0] as f32, pos[1] as f32);

                if let Some(p) = last_pos {
                    let (last_x, last_y) = (p[0] as f32, p[1] as f32);
                    let distance = vec2_len(vec2_sub(p, pos)) as u32;

                    for i in 0..distance {
                        let diff_x = x - last_x;
                        let diff_y = y - last_y;
                        let delta = i as f32 / distance as f32;
                        let new_x = (last_x + (diff_x * delta)) as u32;
                        let new_y = (last_y + (diff_y * delta)) as u32;
                        if new_x < width && new_y < height {
                            canvas.put_pixel(new_x, new_y, Rgba([0, 0, 0, 255]));
                        };
                    };
                };

                last_pos = Some(pos)
            };
        }
    }
}
