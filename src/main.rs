extern crate piston_window;
extern crate image as im;
extern crate vecmath;
extern crate specs;

// I need a
// https://raw.githubusercontent.com/PistonDevelopers/piston-examples/master/src/paint.rs
// is what I need!

use piston_window::*;
use specs::prelude::*;
use vecmath::*;


#[derive(Debug)]
struct Velocity {
    x: f32,
    y: f32
}

impl Component for Velocity {
    type Storage = VecStorage<Self>;
}

#[derive(Debug)]
struct Position {
    x: f32,
    y: f32
}

impl Component for Position {
    type Storage = VecStorage<Self>;
}

struct SysPhys;

impl<'a> System<'a> for SysPhys {
    // These are the resources required for execution.
    // You can also define a struct and `#[derive(SystemData)]`,
    // see the `full` example.
    type SystemData = (WriteStorage<'a, Position>, ReadStorage<'a, Velocity>);

    fn run(&mut self, (mut pos, vel): Self::SystemData) {
        // The `.join()` combines multiple component storages,
        // so we get access to all entities which have
        // both a position and a velocity.
        for (pos, vel) in (&mut pos, &vel).join() {
            pos.x += vel.x;
            pos.y += vel.y;
        }
    }
}


fn main() {
    let (width, height) = (640, 480);
    let mut window: PistonWindow =
        WindowSettings::new("piston: paint", (width, height))
        .exit_on_esc(true)
        .build()
        .unwrap();

    let mut canvas = im::ImageBuffer::new(width, height);
    let mut draw = false;
    let mut texture_context = window.create_texture_context();
    let mut texture: G2dTexture = Texture::from_image(
        &mut texture_context,
        &canvas,
        &TextureSettings::new()
    ).unwrap();

    let mut last_pos: Option<[f64; 2]> = None;

    let mut world = World::new();
    world.register::<Position>();
    world.register::<Velocity>();

    world.create_entity()
        .with(Position { x: 1.6, y: 10.0 })
    .build();

    let mut dispatcher = DispatcherBuilder::new()
        .with(SysPhys,"sys_phys",&[])
        .build();
    dispatcher.setup(&mut world.res);

    while let Some(evt) = window.next() {
        if let Some(_) = evt.update_args() {
            dispatcher.dispatch(&mut world.res);
            world.maintain();
        }
        window.draw_2d(&evt, |c, g, device| {
            texture.update(&mut texture_context, &canvas).unwrap();
            // Update texture before rendering.
            texture_context.encoder.flush(device);

            clear([1.0; 4], g);
            image(&texture, c.transform, g);
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
                            canvas.put_pixel(new_x, new_y, im::Rgba([0, 0, 0, 255]));
                        };
                    };
                };

                last_pos = Some(pos)
            };
        }
    }
}
