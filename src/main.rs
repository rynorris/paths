#[macro_use] extern crate nom;
#[macro_use] extern crate serde_derive;

mod paths;
mod stress;

use std::env;
use std::fs::File;
use std::sync::Arc;
use std::time::Instant;

use crate::paths::renderer::Renderer;
use crate::paths::serde::SceneDescription;

use sdl2::{event, pixels};
use sdl2::keyboard::Keycode;
use serde_yaml;

const SCALE: u32 = 1;

fn main() {
    let args: Vec<String> = env::args().collect();

    // Load scene.
    let scene_description: SceneDescription = args.get(1).map(|filename| {
        println!("Loading scene from {}", filename);
        let scene_file = File::open(filename).expect("Could open scene file");
        serde_yaml::from_reader(scene_file).expect("Could parse scene file")
    }).unwrap_or_else(|| {
        println!("No scene file passed in, generating random stress scene...");
        stress::generate_stress_scene(500)
    });

    let teapot: paths::obj::Object = {
        let obj_file = File::open("./scenes/objects/teapot.obj").unwrap();
        paths::obj::parse_obj(obj_file)
    };

    let mut triangles: Vec<paths::scene::Object> = teapot.resolve_triangles().iter()
        .map(|t| paths::scene::Object{
            shape: Box::new(*t),
            material: Box::new(paths::material::Gloss::new(paths::colour::Colour::rgb(0.8, 0.3, 0.3), 2.0)),
        }).collect();

    triangles.push(paths::scene::Object{
        shape: Box::new(paths::scene::Sphere{ 
            center: paths::vector::Vector3::new(0.0, 1_000_000.0, 0.0),
            radius: 1_000_000.0,
        }),
        material: Box::new(paths::material::Gloss::new(paths::colour::Colour::rgb(0.5, 0.5, 0.5), 2.0)),
    });

    let mut scene = scene_description.to_scene();
    scene = paths::scene::Scene::new(
        scene.camera,
        triangles,
        scene.skybox,
    );

    println!("Contructing scene...");
    let width = scene_description.camera.image_width;
    let height = scene_description.camera.image_height;

    // Initialize SDL and create window.
    let sdl_context = sdl2::init().unwrap();
    let video = sdl_context.video().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut main_window = video.window("Path Tracer", width * SCALE as u32, height * SCALE as u32)
        .position_centered()
        .opengl()
        .build()
        .unwrap();

    main_window.raise();

    let mut canvas = main_window.into_canvas()
        .accelerated()
        .build()
        .unwrap();

    let texture_creator = canvas.texture_creator();
    let mut output_texture = match texture_creator.create_texture_static(Some(pixels::PixelFormatEnum::RGB24), width, height) {
        Err(cause) => panic!("Failed to create texture: {}", cause),
        Ok(t) => t,
    };

    let mut yaw: f64 = scene_description.camera.yaw;
    let mut pitch: f64 = scene_description.camera.pitch;
    let mut roll: f64 = scene_description.camera.roll;

    let mut renderer = Renderer::new(Arc::new(scene), 4);

    let mut texture_buffer: Vec<u8> = vec![0; (width * height * 3) as usize];

    let mut is_running = true;

    let mut num_samples = 0;

    let start_time = Instant::now();

    while is_running {
        renderer.trace_full_pass();
        let image = renderer.render();

        num_samples += 1;
        println!("[{:.1?}] Num samples: {:?}", start_time.elapsed(), num_samples);

        for ix in 0 .. image.pixels.len() {
            let colour = image.pixels[ix];
            let (r, g, b) = colour.to_bytes();
            texture_buffer[ix * 3] = r;
            texture_buffer[ix * 3 + 1] = g;
            texture_buffer[ix * 3 + 2] = b;
        }

        canvas.clear();
        output_texture.update(None, texture_buffer.as_slice(), (width * 3) as usize).expect("Failed to update texture");
        canvas.copy(&output_texture, None, None).expect("Failed to copy texture to canvas");
        canvas.present();

        while let Some(e) = event_pump.poll_event() {
            match e {
                event::Event::KeyDown { keycode, .. } => match keycode {
                   Some(Keycode::Escape) => is_running = false,
                   Some(Keycode::Return) => {
                       println!("Resetting render");
                       renderer.reset();
                       num_samples = 0;
                   },
                   Some(Keycode::O) => yaw -= 0.1,
                   Some(Keycode::U) => yaw += 0.1,
                   Some(Keycode::I) => pitch -= 0.1,
                   Some(Keycode::K) => pitch += 0.1,
                   Some(Keycode::J) => roll -= 0.1,
                   Some(Keycode::L) => roll += 0.1,
                   Some(Keycode::W) => Arc::get_mut(&mut renderer.scene).expect("Can get mutable reference to scene").camera.distance_from_lens += 0.00001,
                   Some(Keycode::Q) => Arc::get_mut(&mut renderer.scene).expect("Can get mutable reference to scene").camera.distance_from_lens -= 0.00001,
                   _ => (),
                },
                _ => (),
            }

            Arc::get_mut(&mut renderer.scene).expect("Can get mutable reference to scene").camera.set_orientation(yaw, pitch, roll);
            println!("Yaw: {:.1}, Pitch: {:.1}, Roll: {:.1}", yaw, pitch, roll);
            println!("F: {:.1}, V: {:.1}, A: {:.1}",
                     renderer.scene.camera.focal_length,
                     renderer.scene.camera.distance_from_lens,
                     renderer.scene.camera.aperture);
        }
    }
}
