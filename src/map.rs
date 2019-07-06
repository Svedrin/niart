use piston_window::*;
use gfx_device_gl::Device;
use im::{ImageBuffer,Rgba};
use imp::drawing::draw_line_segment_mut;
use std::collections::VecDeque;

use super::physics::Position;

pub enum MapEvent {
    NewRail(Position, Position),
    // TODO the RandomEvent is never used and only exists because without it, some "if let"
    // would raise an "Irrefutable Pattern" error.
    // MapEvent probably does not need to exist, and we can just replace it altogether with
    // NewRail. I just don't care enough to do it right now.
    #[allow(dead_code)]
    RandomEvent
}

#[derive(Clone)]
enum State {
    NotDrawing,
    DrawingFrom(Position)
}

pub struct Map {
    canvas:          ImageBuffer<Rgba<u8>,Vec<u8>>,
    state:           State,
    texture_context: G2dTextureContext,
    texture:         G2dTexture,
    mouse_pos:       Position,
    events:          VecDeque<MapEvent>
}

impl Map {
    pub fn new(window: &mut PistonWindow, width: u32, height: u32) -> Self {
        let canvas = ImageBuffer::new(width, height);
        let mut texture_context = window.create_texture_context();
        let texture: G2dTexture = Texture::from_image(
            &mut texture_context,
            &canvas,
            &TextureSettings::new()
        ).unwrap();
        Self {
            canvas: canvas,
            state:  State::NotDrawing,
            texture_context: texture_context,
            texture:   texture,
            mouse_pos: Position::zero(),
            events:    VecDeque::new(),
        }
    }

    pub fn render(&mut self, c: Context, g: &mut G2d, device: &mut Device) {
        // Update texture before rendering.
        self.texture.update(&mut self.texture_context, &mut self.canvas).unwrap();
        self.texture_context.encoder.flush(device);
        image(&self.texture, c.transform, g);
        if let State::DrawingFrom(ref pos) = self.state {
            line_from_to(
                [0., 0., 0., 1.],
                2.0,
                pos.as_f64_array(),
                self.mouse_pos.as_f64_array(),
                c.transform,
                g
            );
        }
    }

    pub fn start_drawing(&mut self) {
        self.state = State::DrawingFrom(self.mouse_pos.clone());
    }

    pub fn stop_drawing(&mut self) {
        if let State::DrawingFrom(start_pos) = self.state.clone() {
            self.draw_rails_at(&start_pos, &self.mouse_pos.clone());
            self.events.push_back(
                MapEvent::NewRail(start_pos.clone(), self.mouse_pos.clone())
            );
        }
        self.state = State::NotDrawing;
    }

    pub fn mouse_moved(&mut self, pos: [f64; 2]) {
        self.mouse_pos = Position::from(pos);
        if let Some(better_pos) = self.find_rails_at(&self.mouse_pos) {
            self.mouse_pos = better_pos;
        }
    }

    pub fn next_event(&mut self) -> Option<MapEvent> {
        self.events.pop_front()
    }

    pub fn draw_rails_at(&mut self, start: &Position, end: &Position) {
        draw_line_segment_mut(
            &mut self.canvas,
            start.as_f32_tuple(),
            end.as_f32_tuple(),
            Rgba([0, 0, 0, 255])
        );
    }

    fn find_rails_at(&self, start: &Position) -> Option<Position> {
        let sx = start.x as u32;
        let sy = start.y as u32;
        let mut min_dst = None; // Minimal distance to our mouse cursor
        let mut min_pos = None; // Position at which we observed min_dst
        for check_x in (sx - 5)..(sx + 5) {
            for check_y in (sy - 5)..(sy + 5) {
                // Make sure the pixel actually exists
                if check_x >= self.canvas.width() ||
                   check_y >= self.canvas.height() {
                    continue;
                }
                let px = self.canvas.get_pixel(check_x, check_y);
                if px[3] == 0xFF {
                    // Pixel is a track
                    let here = Position::new(check_x as f64, check_y as f64);
                    if min_dst.is_none() {
                        min_dst = Some(start.distance_length_to(&here));
                        min_pos = Some(here);
                    } else {
                        let check_dst = start.distance_length_to(&here);
                        if check_dst < min_dst.unwrap() {
                            min_dst = Some(check_dst);
                            min_pos = Some(here);
                        }
                    }
                }
            }
        }
        min_pos
    }
}
