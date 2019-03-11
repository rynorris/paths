use rand;
use rand::Rng;

use crate::paths::serde;

pub fn generate_stress_scene(num_spheres: usize) -> serde::SceneDescription {
    let objects = (0 .. num_spheres)
        .map(|_| serde::ObjectDescription{ shape: random_sphere(), material: random_material() })
        .collect();

    let camera = serde::CameraDescription {
        image_width: 720,
        image_height: 480,
        location: serde::VectorDescription{ x: 0.0, y: -5.0, z: -13.0 },
        yaw: 0.0,
        pitch: 0.0,
        roll: -0.3,
        sensor_width: 0.036,
        sensor_height: 0.024,
        focal_length: 0.05,
        focus_distance: 10.0,
        aperture: 8.0,
    };

    let skybox = serde::SkyboxDescription::Flat(serde::FlatSkyboxDescription{
        colour: serde::ColourDescription{ r: 0.8, g: 0.8, b: 0.8 },
    });

    serde::SceneDescription{ camera, skybox, objects }
}

fn random_sphere() -> serde::ShapeDescription {
    let mut rng = rand::thread_rng();
    let center = serde::VectorDescription{
        x: rng.gen::<f64>() * 100.0 - 50.0,
        y: rng.gen::<f64>() * 100.0 - 50.0,
        z: rng.gen::<f64>() * 100.0 };
    let radius = rng.gen::<f64>() * 5.0;
    serde::ShapeDescription::Sphere(serde::SphereDescription{ center, radius })
}

fn random_material() -> serde::MaterialDescription {
    let mut rng = rand::thread_rng();
    let choice = rng.gen_range(0, 3);
    match choice {
        0 => serde::MaterialDescription::Gloss(serde::GlossMaterialDescription{
            albedo: random_colour(),
            reflectance: 1.0 + rng.gen::<f64>() * 2.0,
        }),
        1 => serde::MaterialDescription::Lambertian(serde::LambertianMaterialDescription{
            albedo: random_colour(),
        }),
        _ => serde::MaterialDescription::Mirror(serde::MirrorMaterialDescription{}),
    }
}

fn random_colour() -> serde::ColourDescription {
    let mut rng = rand::thread_rng();
    serde::ColourDescription {
        r: rng.gen(),
        g: rng.gen(),
        b: rng.gen(),
    }
}
