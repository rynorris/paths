use rand::Rng;

use crate::colour::Colour;
use crate::geom::{Geometry, Ray};
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

        let cos_in: f64 = ray.direction.dot(collision.normal * -1);
        if cos_in <= 0.0 {
            break;
        }

        match entity {
            Entity::Light(l) => {
                // If we hit a light on a specular bounce, just accumulate the light energy and
                // we're done.
                // Otherwise we've already taken lights into account via NEE, so don't
                // accumulate.
                if last_bounce_specular {
                    colour += throughput * l.colour * l.intensity;
                    colour.check();
                }
                break;
            },
            Entity::Object(o) => {
                // Resolve material at point.
                let material = match o.geometry {
                    Geometry::Mesh(mesh) => {
                        let model = scene.models.get(&mesh.model);
                        o.material.resolve(&collision, model)
                    },
                    _ => o.material,
                };

                // Next Event Estimation.
                let direct_illumination = match scene.random_light() {
                    Some(light) => {
                        let (in_dir, inv_pdf) = light.sample(collision.location);
                        let shadow_ray = Ray::new(
                            collision.location + collision.normal * 0.0001,  // Add the normal as a hack so it doesn't collide with the same object again.
                            in_dir * -1,
                        );

                        let occluded = match scene.find_intersection(shadow_ray) {
                            Some((_, e)) => {
                                e.id() != light.entity_id()
                            },
                            None => false,
                        };

                        let cos_theta = f64::max(0.0, collision.normal.dot(shadow_ray.direction));
                        if occluded || cos_theta <= 0.0 {
                            Colour::BLACK
                        } else {
                            let base = light.colour * light.intensity;
                            let brdf = material.brdf(ray.direction * -1, shadow_ray.direction * -1, collision.normal);
                            base * brdf * inv_pdf
                        }
                    },
                    None => Colour::BLACK,
                };

                direct_illumination.check();
                colour += direct_illumination * throughput;
                colour.check();

                let (direction, pdf, brdf, is_specular) = material.sample(ray.direction * -1, collision.normal);
                last_bounce_specular = is_specular;

                // Next bounce.
                let new_ray = Ray::new(
                    collision.location + collision.normal * 0.0001,  // Add the normal as a hack so it doesn't collide with the same object again.
                    direction,
                );

                let attenuation = brdf / pdf;
                throughput = throughput * attenuation;

                if throughput.max() <= 0.0 {
                    break;
                }

                let emittance = material.emittance(ray.direction * -1, cos_in);
                colour += emittance * throughput;
                
                // Chance for the material to eat the ray.
                if loops >= 2 {
                    let survival_chance = throughput.max();
                    if rand::thread_rng().gen::<f64>() > survival_chance {
                        break;
                    }

                    throughput = throughput / survival_chance;
                }

                ray = new_ray;
            },
        };

        loops += 1;
    }

    colour
}
