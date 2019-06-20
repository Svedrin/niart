extern crate piston_window;

use piston_window::*;
use piston::input::{RenderEvent, UpdateEvent, PressEvent, MouseCursorEvent};
use piston::input::ButtonState;

fn main() {
    let width = 640;
    let height  = 480;

    let mut window: PistonWindow =
        WindowSettings::new("Hello Piston!", [width, height])
        .exit_on_esc(true)
        .build()
        .unwrap();

    let mut texture_context = window.create_texture_context();

    let mut le_bouton_depressed = false;
    let mut dem_rails = vec![false; (width * height) as usize];

    while let Some(e) = window.next() {
        //println!("Random Event! {:?}", e);
        e.mouse_cursor(|pos| {
            println!("Mouse cursor is at {}x{}", pos[0], pos[1]);
            if le_bouton_depressed {
                dem_rails[(pos[1] as usize * width as usize) + pos[0] as usize] = true;
            }

        });
        e.button(|le_bouton| {
            println!("Got a le bouton! {:?}", le_bouton);
            le_bouton_depressed = match le_bouton.state {
                ButtonState::Press   => true,
                ButtonState::Release => false,
            };
        });
        window.draw_2d(&e, |c, g, _device| {
            clear([1.0; 4], g);
            for x in 0..width {
                for y in 0..height {
                    if dem_rails[((y * width) + x) as usize] {
                        ellipse_from_to(
                            [0., 0., 0., 1.],
                            [x as f64, y as f64],
                            [(x+1) as f64, (y+1) as f64],
                            c.transform,
                            g
                        );
                    }
                }
            }
        });
    }
}
