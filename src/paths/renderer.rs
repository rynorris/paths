use std::f64::consts::PI;

use rand;
use rand::Rng;

use crate::paths::{Camera, Image, Ray};
use crate::paths::colour::{Colour};
use crate::paths::matrix::Matrix3;
use crate::paths::scene::{Collision, Scene};

pub struct Renderer {
    ambient_light: Colour,
    scene: Scene,
    camera: Camera,
    colour_buffer: Vec<Colour>,
    count_buffer: Vec<u32>,
}

impl Renderer {
    pub fn new(scene: Scene, camera: Camera) -> Renderer {
        let colour_buffer = vec![Colour::BLACK; (camera.width * camera.height) as usize];
        let count_buffer = vec![0; (camera.width * camera.height) as usize];
        Renderer{ ambient_light: Colour::BLACK, scene, camera, colour_buffer, count_buffer, }
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

    pub fn trace_full_pass(&mut self) {
        for x in 0 .. self.camera.width {
            for y in 0 .. self.camera.height {
                let ray = self.camera.get_ray_for_pixel(x, y);
                let colour = self.trace_ray(ray, 0);
                self.update_pixel(x, y, colour);
            }
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

    pub fn set_ambient_light(&mut self, light: Colour) {
        self.ambient_light = light;
    }

    fn trace_ray(&mut self, ray: Ray, depth: u32) -> Colour {
        if depth > 4 {
            return self.ambient_light;
        }

        let (collision, material) = if let Some((c, m)) = self.scene.find_intersection(ray) {
            (c, m)
        } else {
            return self.ambient_light;
        };

        let emittance = material.emittance;

        let new_ray = Renderer::new_ray(collision);

        let p = 1.0 / (2.0 * PI);

        let cos_theta: f64 = new_ray.direction.dot(collision.normal);
        let brdf: Colour = material.reflectance / PI;

        let incoming: Colour = self.trace_ray(new_ray, depth + 1);

        return emittance + (brdf * incoming * (cos_theta / p));
    }

    fn new_ray(collision: Collision) -> Ray {
        let ray = Ray{
            origin: collision.location + collision.normal,  // Add the normal as a hack so it doesn't collide with the same object again.
            direction: collision.normal,
        };
        ray.random_in_hemisphere()
    }

    fn update_pixel(&mut self, x: u32, y: u32, mut colour: Colour) {
        colour.r = Renderer::clamp(colour.r);
        colour.g = Renderer::clamp(colour.g);
        colour.b = Renderer::clamp(colour.b);

        let ix = self.get_index(x, y);
        self.count_buffer[ix] += 1;
        self.colour_buffer[ix] += colour;
    }

    fn clamp(x: f64) -> f64 {
        if x > 1.0 { 1.0 } else if x < 0.0 { 0.0 } else { x }
    }

    fn get_index(&self, x: u32, y: u32) -> usize {
        ((y * self.camera.width) + x) as usize
    }
}
