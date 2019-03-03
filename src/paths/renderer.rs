use rand;
use rand::Rng;

use crate::paths::{Camera, Image};
use crate::paths::colour::{Colour};
use crate::paths::scene::Scene;

pub struct Renderer {
    scene: Scene,
    camera: Camera,
    colour_buffer: Vec<Colour>,
    count_buffer: Vec<u32>,
}

impl Renderer {
    pub fn new(scene: Scene, camera: Camera) -> Renderer {
        let colour_buffer = vec![Colour::BLACK; (camera.width * camera.height) as usize];
        let count_buffer = vec![0; (camera.width * camera.height) as usize];
        Renderer{ scene, camera, colour_buffer, count_buffer, }
    }

    pub fn render(&self) -> Image {
        let mut buffer = Vec::with_capacity(self.colour_buffer.len());
        self.colour_buffer.iter().zip(self.count_buffer.iter()).for_each(|(colour, count)| {
            buffer.push((*colour) / (*count));
        });
        Image {
            width: self.camera.width,
            height: self.camera.height,
            pixels: buffer,
        }
    }

    pub fn trace_rays(&mut self, num_rays: u32) {
        for _ in 0 .. num_rays {
            self.trace_ray_single();
        }
    }

    pub fn trace_ray_single(&mut self) {
        // Random pixel.
        let mut rng = rand::thread_rng();
        let x = rng.gen_range(0, self.camera.width);
        let y = rng.gen_range(0, self.camera.height);

        let ray = self.camera.get_ray_for_pixel(x, y);
        let colour = if let Some((collision, material)) = self.scene.find_intersection(ray) {
            material.emittance
        } else {
            Colour::BLACK
        };

        self.update_pixel(x, y, colour);
    }

    fn update_pixel(&mut self, x: u32, y: u32, colour: Colour) {
        let ix = self.get_index(x, y);
        self.count_buffer[ix] += 1;
        self.colour_buffer[ix] += colour;
    }

    fn get_index(&self, x: u32, y: u32) -> usize {
        ((y * self.camera.width) + x) as usize
    }
}
