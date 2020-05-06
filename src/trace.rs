use rand::Rng;

use crate::colour::Colour;
use crate::geom::Ray;
use crate::scene::{Entity, Scene};

pub fn trace_ray(scene: &Scene, mut ray: Ray) -> Colour {
    let mut throughput = Colour::WHITE;
    let mut colour = Colour::BLACK;
    let mut loops = 0;
    let mut last_bounce_specular = true;

    loop {
        if loops > 10 {
            break;
        }

        let (collision, entity) = if let Some((c, e)) = scene.find_intersection(ray) {
            (c, e)
        } else {
            colour += throughput * scene.skybox.ambient_light(ray.direction * -1);
            break;
        };

        let cos_out: f64 = ray.direction.dot(collision.normal);

        match entity {
            Entity::Light(l) => {
                // If we hit a light on a specular bounce, just accumulate the light energy and
                // we're done.
                // Otherwise we've already taken lights into account via NEE, so don't
                // accumulate.
                if last_bounce_specular {
                    colour += throughput * l.colour * l.intensity;
                }
                break;
            },
            Entity::Object(o) => {
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
                            Some((col, e)) => {
                                e.id() != light.entity_id() && (col.distance * col.distance) < distance_to_light_squared
                            },
                            None => false,
                        };

                        if occluded {
                            Colour::BLACK
                        } else {
                            let base = light.colour * light.intensity;
                            let brdf = o.material.brdf(ray.direction * -1, shadow_ray.direction * -1, collision.normal);
                            let solid_angle = 1.0 / distance_to_light_squared;
                            base * brdf * solid_angle * (collision.normal.dot(shadow_ray.direction))
                        }
                    },
                    None => Colour::BLACK,
                };

                colour += direct_illumination * throughput;

                let (direction, pdf, brdf, is_specular) = o.material.sample(ray.direction * -1, collision.normal);
                last_bounce_specular = is_specular;

                // Next bounce.
                let new_ray = Ray::new(
                    collision.location + collision.normal * 0.0001,  // Add the normal as a hack so it doesn't collide with the same object again.
                    direction,
                );

                let attenuation = brdf / pdf;
                throughput = throughput * attenuation;

                let emittance = o.material.emittance(ray.direction * -1, cos_out);
                colour += emittance * throughput;
                
                // Chance for the material to eat the ray.
                let survival_chance = throughput.max();
                if rand::thread_rng().gen::<f64>() > survival_chance {
                    break;
                }

                throughput = throughput / survival_chance;

                ray = new_ray;
            },
        };

        loops += 1;
    }

    colour
}
