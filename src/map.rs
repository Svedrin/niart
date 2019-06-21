use piston_window::*;
use gfx_device_gl::Device;
use im::{ImageBuffer,Rgba};
use imp::drawing::draw_line_segment_mut;

pub struct Map {
    canvas:          ImageBuffer<Rgba<u8>,Vec<u8>>,
    draw:            bool,
    texture_context: G2dTextureContext,
    texture:         G2dTexture,
    last_pos:        Option<[f64; 2]>,
    start_pos:       Option<[f64; 2]>,
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
            draw:   false,
            texture_context: texture_context,
            texture:   texture,
            last_pos:  Some([0.0; 2]),
            start_pos: None,
        }
    }

    pub fn render(&mut self, c: Context, g: &mut G2d, device: &mut Device) {
        // Update texture before rendering.
        self.texture.update(&mut self.texture_context, &mut self.canvas).unwrap();
        self.texture_context.encoder.flush(device);
        image(&self.texture, c.transform, g);
        if self.draw {
            line_from_to(
                [0., 0., 0., 1.],
                2.0,
                self.start_pos.unwrap(),
                self.last_pos.unwrap(),
                c.transform,
                g
            );
        }
    }

    pub fn start_drawing(&mut self) {
        self.draw = true;
        self.start_pos = self.last_pos.clone();
    }

    pub fn stop_drawing(&mut self) {
        self.draw = false;
        //self.last_pos = None;
        let start_pos = self.start_pos.unwrap();
        let last_pos = self.last_pos.unwrap();
        draw_line_segment_mut(
            &mut self.canvas,
            (start_pos[0] as f32, start_pos[1] as f32),
            (last_pos[0] as f32, last_pos[1] as f32),
            Rgba([0, 0, 0, 255])
        );
    }

    pub fn mouse_moved(&mut self, pos: [f64; 2]) {
        if self.draw {
            if self.start_pos.is_none() {
                self.start_pos = Some(pos.clone());
            }
        }
        self.last_pos = Some(pos);
    }
}
