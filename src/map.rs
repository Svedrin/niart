use piston_window::*;
use gfx_device_gl::Device;
use im::{ImageBuffer,Rgba};
use vecmath::*;

pub struct Map {
    width:           u32,
    height:          u32,
    canvas:          ImageBuffer<Rgba<u8>,Vec<u8>>,
    draw:            bool,
    texture_context: G2dTextureContext,
    texture:         G2dTexture,
    last_pos:        Option<[f64; 2]>
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
            width:  width,
            height: height,
            canvas: canvas,
            draw:   false,
            texture_context: texture_context,
            texture: texture,
            last_pos: None
        }
    }

    pub fn render(&mut self, c: Context, g: &mut G2d, device: &mut Device) {
        // Update texture before rendering.
        self.texture.update(&mut self.texture_context, &mut self.canvas).unwrap();
        self.texture_context.encoder.flush(device);
        image(&self.texture, c.transform, g);
    }

    pub fn start_drawing(&mut self) {
        self.draw = true;
    }

    pub fn stop_drawing(&mut self) {
        self.draw = false;
        self.last_pos = None;
    }

    pub fn mouse_moved(&mut self, pos: [f64; 2]) {
        if self.draw {
            let (x, y) = (pos[0] as f32, pos[1] as f32);

            if let Some(p) = self.last_pos {
                let (last_x, last_y) = (p[0] as f32, p[1] as f32);
                let distance = vec2_len(vec2_sub(p, pos)) as u32;

                for i in 0..distance {
                    let diff_x = x - last_x;
                    let diff_y = y - last_y;
                    let delta = i as f32 / distance as f32;
                    let new_x = (last_x + (diff_x * delta)) as u32;
                    let new_y = (last_y + (diff_y * delta)) as u32;
                    if new_x < self.width && new_y < self.height {
                        self.canvas.put_pixel(new_x, new_y, Rgba([0, 0, 0, 255]));
                    };
                };
            };

            self.last_pos = Some(pos)
        }

    }
}
