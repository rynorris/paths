use std::sync::mpsc::channel;

use rand;
use rand::Rng;
use threadpool::ThreadPool;

use crate::paths::{Image, Ray};
use crate::paths::colour::{Colour};
use crate::paths::pixels::Estimator;
use crate::paths::scene::Scene;
use crate::paths::vector::Vector3;

pub struct Renderer {
    pub scene: Scene,
    estimator: Estimator,
    pool: ThreadPool,
}

impl Renderer {
    pub fn new(scene: Scene, num_workers: usize) -> Renderer {
        let estimator = Estimator::new(scene.camera.width as usize, scene.camera.height as usize);
        let pool = ThreadPool::new(num_workers);
        Renderer{ scene, estimator, pool}
    }

    pub fn render(&self) -> Image {
        self.estimator.render()
    }

    pub fn trace_full_pass(&mut self) {
        let (tx, rx) = channel::<(u32, u32, Colour)>();

        self.scene.camera.init_bundle();

        for x in 0 .. self.scene.camera.width {
            let tx = tx.clone();
            let scene = self.scene.clone();
            let mut camera = self.scene.camera.clone();

            self.pool.execute(move|| {
                for y in 0 .. camera.height {
                    let ray = camera.get_ray_for_pixel(x, y);
                    let weight = ray.direction.dot(Vector3::new(0.0, 0.0, 1.0));
                    let colour = Renderer::trace_ray(&scene, ray, 0) * weight;
                    tx.send((x, y, colour)).expect("can send result back");
                }
            });
        }

        let num_pixels = self.scene.camera.height * self.scene.camera.width;
        rx.iter().take(num_pixels as usize).for_each(|(x, y, colour)| {
            self.estimator.update_pixel(x as usize, y as usize, colour);
        });
    }

    pub fn trace_batch(&mut self, num_rays: usize) {
        for _ in 0 .. num_rays {
            let (x, y) = self.estimator.choose_pixel();
            let ray = self.scene.camera.get_ray_for_pixel(x as u32, y as u32);
            let colour = Renderer::trace_ray(&self.scene, ray, 0);
            self.estimator.update_pixel(x as usize, y as usize, colour);
        }
    }

    pub fn reset(&mut self) {
        self.estimator = Estimator::new(self.scene.camera.width as usize, self.scene.camera.height as usize);
    }

    fn trace_ray(scene: &Scene, ray: Ray, depth: u32) -> Colour {
        if depth > 10 {
            return Colour::BLACK;
        }

        let (collision, material) = if let Some((c, m)) = scene.find_intersection(ray) {
            (c, m)
        } else {
            return scene.skybox.ambient_light(ray.direction * -1);
        };

        let cos_out: f64 = ray.direction.dot(collision.normal);

        let emittance = material.emittance(ray.direction * -1, cos_out);

        let reflectance = material.weight_pdf(ray.direction * -1, collision.normal);

        // Chance for the material to eat the ray.
        let absorption_chance = reflectance.max();
        if rand::thread_rng().gen::<f64>() > absorption_chance {
            return emittance;
        }

        let new_ray = Ray{
            origin: collision.location + collision.normal * 0.0001,  // Add the normal as a hack so it doesn't collide with the same object again.
            direction: material.sample_pdf(ray.direction * -1, collision.normal),
        };

        let incoming: Colour = Renderer::trace_ray(scene, new_ray, depth + 1);

        return emittance + (reflectance * incoming / absorption_chance);
    }
}
