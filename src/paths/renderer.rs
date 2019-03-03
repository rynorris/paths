use rand;
use rand::Rng;

use crate::paths::{Camera, Image, Ray};
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

        let colour = self.trace_ray(ray, 0);

        self.update_pixel(x, y, colour);
    }

    fn trace_ray(&mut self, ray: Ray, depth: u32) -> Colour {
        if depth > 2 {
            return Colour::BLACK;
        }

        let (collision, material) = if let Some((c, m)) = self.scene.find_intersection(ray) {
            (c, m)
        } else {
            return Colour::BLACK;
        };

        if depth >= 1 {
            println!("Collision: {:?}, Material: {:?}", collision, material);
        }


        let emittance = material.emittance;

        let new_ray = Ray{
            origin: collision.location + collision.normal,  // Add the normal as a hack so it doesn't collide with the same object again.
            direction: collision.normal,
        };

        let p = 1.0 / (2.0 * std::f64::consts::PI);

        let cos_theta: f64 = new_ray.direction.dot(collision.normal);
        let brdf: Colour = material.reflectance / std::f64::consts::PI;

        let incoming: Colour = self.trace_ray(new_ray, depth + 1);

        return emittance + (brdf * incoming * (cos_theta / p));
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
