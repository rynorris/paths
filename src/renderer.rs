use std::sync::{Arc};
use std::sync::mpsc::channel;

use rand;
use rand::Rng;
use threadpool::ThreadPool;

use crate::camera::Image;
use crate::colour::Colour;
use crate::geom::Ray;
use crate::pixels::Estimator;
use crate::scene::Scene;

pub struct Renderer {
    pub scene: Arc<Scene>,
    estimator: Estimator,
    pool: ThreadPool,
}

impl Renderer {
    pub fn new(scene: Arc<Scene>, num_workers: usize) -> Renderer {
        let estimator = Estimator::new(scene.camera.width as usize, scene.camera.height as usize);
        let pool = ThreadPool::new(num_workers);
        Renderer{ scene, estimator, pool}
    }

    pub fn render(&self) -> Image {
        self.estimator.render()
    }

    pub fn trace_full_pass(&mut self) {
        let (tx, rx) = channel::<(u32, u32, Colour)>();

        Arc::get_mut(&mut self.scene).expect("Can get mutable reference to scene").camera.init_bundle();

        for x in 0 .. self.scene.camera.width {
            let tx = tx.clone();
            let scene = self.scene.clone();
            let mut camera = self.scene.camera.clone();

            self.pool.execute(move|| {
                for y in 0 .. camera.height {
                    let (ray, weight) = camera.get_ray_for_pixel(x, y);
                    let colour = Renderer::trace_ray(&scene, ray, 0) * weight;
                    tx.send((x, y, colour)).expect("can send result back");
                }
            });
        }

        self.pool.join();

        if self.pool.panic_count() > 0 {
            panic!("{} rendering threads panicked while rendering.", self.pool.panic_count());
        }

        let num_pixels = self.scene.camera.height * self.scene.camera.width;
        rx.iter().take(num_pixels as usize).for_each(|(x, y, colour)| {
            self.estimator.update_pixel(x as usize, y as usize, colour);
        });
    }

    pub fn reset(&mut self) {
        self.estimator = Estimator::new(self.scene.camera.width as usize, self.scene.camera.height as usize);
    }

    fn trace_ray(scene: &Scene, mut ray: Ray, depth: u32) -> Colour {
        let mut throughput = Colour::WHITE;
        let mut colour = Colour::BLACK;
        let mut loops = 0;

        loop {
            if loops > 10 {
                break;
            }

            let (collision, material) = if let Some((c, m)) = scene.find_intersection(ray) {
                (c, m)
            } else {
                colour += throughput * scene.skybox.ambient_light(ray.direction * -1);
                break;
            };

            let cos_out: f64 = ray.direction.dot(collision.normal);

            let emittance = material.emittance(ray.direction * -1, cos_out);

            let new_ray = Ray{
                origin: collision.location + collision.normal * 0.0001,  // Add the normal as a hack so it doesn't collide with the same object again.
                direction: material.sample_pdf(ray.direction * -1, collision.normal),
            };

            let pdf = material.weight_pdf(ray.direction * -1, new_ray.direction * -1, collision.normal);

            let attenuation = material.brdf(ray.direction * -1, new_ray.direction * -1, collision.normal) / pdf;
            throughput = throughput * attenuation.clamped();

            colour += emittance * throughput;

            // Chance for the material to eat the ray.
            let survival_chance = throughput.max();
            if rand::thread_rng().gen::<f64>() > survival_chance {
                break;
            }

            throughput = throughput / survival_chance;

            ray = new_ray;
            loops += 1;
        }

        colour
    }
}
