use std::f64::consts::PI;

use crate::paths::{Camera, Image, Ray};
use crate::paths::colour::{Colour};
use crate::paths::pixels::Estimator;
use crate::paths::scene::{Collision, Scene};

pub struct Renderer {
    scene: Scene,
    camera: Camera,
    estimator: Estimator,
}

impl Renderer {
    pub fn new(scene: Scene, camera: Camera) -> Renderer {
        let estimator = Estimator::new(camera.width as usize, camera.height as usize);
        Renderer{ scene, camera, estimator, }
    }

    pub fn render(&self) -> Image {
        self.estimator.render()
    }

    pub fn trace_full_pass(&mut self) {
        for x in 0 .. self.camera.width {
            for y in 0 .. self.camera.height {
                let ray = self.camera.get_ray_for_pixel(x, y);
                let colour = self.trace_ray(ray, 0);
                self.estimator.update_pixel(x as usize, y as usize, colour);
            }
        }
    }

    pub fn trace_batch(&mut self, num_rays: usize) {
        for _ in 0 .. num_rays {
            let (x, y) = self.estimator.choose_pixel();
            let ray = self.camera.get_ray_for_pixel(x as u32, y as u32);
            let colour = self.trace_ray(ray, 0);
            self.estimator.update_pixel(x as usize, y as usize, colour);
        }
    }

    fn trace_ray(&mut self, ray: Ray, depth: u32) -> Colour {
        if depth > 4 {
            return self.scene.ambient_light;
        }

        let (collision, material) = if let Some((c, m)) = self.scene.find_intersection(ray) {
            (c, m)
        } else {
            return self.scene.ambient_light;
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
}
