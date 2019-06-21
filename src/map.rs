use piston_window::*;
use gfx_device_gl::Device;
use im::{ImageBuffer,Rgba};
use imp::drawing::draw_line_segment_mut;

enum State {
    NotDrawing,
    DrawingFrom([f64; 2])
}

pub struct Map {
    canvas:          ImageBuffer<Rgba<u8>,Vec<u8>>,
    state:           State,
    texture_context: G2dTextureContext,
    texture:         G2dTexture,
    mouse_pos:       [f64; 2],
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
            mouse_pos: [0.0; 2],
        }
    }

    pub fn render(&mut self, c: Context, g: &mut G2d, device: &mut Device) {
        // Update texture before rendering.
        self.texture.update(&mut self.texture_context, &mut self.canvas).unwrap();
        self.texture_context.encoder.flush(device);
        image(&self.texture, c.transform, g);
        if let State::DrawingFrom(pos) = self.state {
            line_from_to(
                [0., 0., 0., 1.],
                2.0,
                pos,
                self.mouse_pos,
                c.transform,
                g
            );
        }
    }

    pub fn start_drawing(&mut self) {
        self.state = State::DrawingFrom(self.mouse_pos.clone());
    }

    pub fn stop_drawing(&mut self) {
        if let State::DrawingFrom(start_pos) = self.state {
            self.state = State::NotDrawing;
            draw_line_segment_mut(
                &mut self.canvas,
                (start_pos[0] as f32, start_pos[1] as f32),
                (self.mouse_pos[0] as f32, self.mouse_pos[1] as f32),
                Rgba([0, 0, 0, 255])
            );
        }
    }

    pub fn mouse_moved(&mut self, pos: [f64; 2]) {
        self.mouse_pos = pos.clone();
    }
}
