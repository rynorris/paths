use rand::Rng;

use crate::colour::Colour;
use crate::geom::Ray;
use crate::scene::Scene;

pub fn trace_ray(scene: &Scene, mut ray: Ray) -> Colour {
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

        let new_ray = Ray::new(
            collision.location + collision.normal * 0.0001,  // Add the normal as a hack so it doesn't collide with the same object again.
            material.sample_pdf(ray.direction * -1, collision.normal),
        );

        let pdf = material.weight_pdf(ray.direction * -1, new_ray.direction * -1, collision.normal);

        let attenuation = material.brdf(ray.direction * -1, new_ray.direction * -1, collision.normal) / pdf;
        throughput = throughput * attenuation;

        colour += emittance * throughput;

        // Next Event Estimation.
        let direct_illumination = match scene.random_light() {
            Some(light) => {
                let point = light.random_point();
                let distance_to_light_squared = (point - collision.location).magnitude();
                let direction = (point - collision.location).normed();
                let shadow_ray = Ray::new(
                    collision.location + collision.normal * 0.0001,  // Add the normal as a hack so it doesn't collide with the same object again.
                    direction
                );

                let occluded = match scene.find_intersection(shadow_ray) {
                    Some((col, _)) => {
                        (col.distance * col.distance) < distance_to_light_squared
                    },
                    None => false,
                };

                if occluded {
                    Colour::BLACK
                } else {
                    let base = light.colour * light.intensity;
                    let brdf = material.brdf(ray.direction * -1, shadow_ray.direction * -1, collision.normal);
                    let solid_angle = 1.0 / distance_to_light_squared;
                    base * brdf * solid_angle * (collision.normal.dot(shadow_ray.direction))
                }
            },
            None => Colour::BLACK,
        };

        colour += direct_illumination * throughput;

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
