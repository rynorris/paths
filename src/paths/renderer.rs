use std::f64::consts::PI;
use std::sync::mpsc::channel;
use std::sync::Arc;

use threadpool::ThreadPool;

use crate::paths::{Camera, Image, Ray};
use crate::paths::colour::{Colour};
use crate::paths::pixels::Estimator;
use crate::paths::scene::{Collision, Scene};

pub struct Renderer {
    scene: Scene,
    camera: Camera,
    estimator: Estimator,
    pool: ThreadPool,
}

impl Renderer {
    pub fn new(scene: Scene, camera: Camera, num_workers: usize) -> Renderer {
        let estimator = Estimator::new(camera.width as usize, camera.height as usize);
        let pool = ThreadPool::new(num_workers);
        Renderer{ scene, camera, estimator, pool}
    }

    pub fn render(&self) -> Image {
        self.estimator.render()
    }

    pub fn trace_full_pass(&mut self) {
        let (tx, rx) = channel::<(u32, u32, Colour)>();

        for x in 0 .. self.camera.width {
            let tx = tx.clone();
            let scene = self.scene.clone();
            let camera = self.camera.clone();

            self.pool.execute(move|| {
                for y in 0 .. camera.height {
                    let ray = camera.get_ray_for_pixel(x, y);
                    let colour = Renderer::trace_ray(&scene, ray, 0);
                    tx.send((x, y, colour)).expect("can send result back");
                }
            });
        }

        let num_pixels = self.camera.height * self.camera.width;
        rx.iter().take(num_pixels as usize).for_each(|(x, y, colour)| {
            self.estimator.update_pixel(x as usize, y as usize, colour);
        });
    }

    pub fn trace_batch(&mut self, num_rays: usize) {
        for _ in 0 .. num_rays {
            let (x, y) = self.estimator.choose_pixel();
            let ray = self.camera.get_ray_for_pixel(x as u32, y as u32);
            let colour = Renderer::trace_ray(&self.scene, ray, 0);
            self.estimator.update_pixel(x as usize, y as usize, colour);
        }
    }

    fn trace_ray(scene: &Scene, ray: Ray, depth: u32) -> Colour {
        if depth > 4 {
            return scene.ambient_light;
        }

        let (collision, material) = if let Some((c, m)) = scene.find_intersection(ray) {
            (c, m)
        } else {
            return scene.ambient_light;
        };

        let emittance = material.emittance;

        let new_ray = Renderer::new_ray(collision);

        let p = 1.0 / (2.0 * PI);

        let cos_theta: f64 = new_ray.direction.dot(collision.normal);

        let brdf: Colour = material.reflectance / PI;

        let incoming: Colour = Renderer::trace_ray(scene, new_ray, depth + 1);

        return emittance + (brdf * incoming * (cos_theta / p));
    }

    fn new_ray(collision: Collision) -> Ray {
        let ray = Ray{
            origin: collision.location + collision.normal,  // Add the normal as a hack so it doesn't collide with the same object again.
            direction: collision.normal,
        };
        ray.random_in_hemisphere()
    }
}
