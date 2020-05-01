#[macro_use] extern crate nom;
#[macro_use] extern crate serde_derive;

pub mod bvh;
pub mod camera;
pub mod colour;
pub mod geom;
pub mod material;
pub mod matrix;
#[macro_use] pub mod obj;
pub mod pixels;
pub mod renderer;
pub mod sampling;
pub mod scene;
pub mod serde;
pub mod stress;
pub mod trace;
pub mod vector;
pub mod worker;

use std::env;
use std::fs::File;
use std::sync::Arc;
use std::time::Instant;

use crate::renderer::Renderer;
use crate::serde::SceneDescription;

use sdl2;
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

    let scene = scene_description.to_scene();

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
    let mut output_texture = match texture_creator.create_texture_static(Some(sdl2::pixels::PixelFormatEnum::RGB24), width, height) {
        Err(cause) => panic!("Failed to create texture: {}", cause),
        Ok(t) => t,
    };

    let mut yaw: f64 = scene_description.camera.orientation.yaw;
    let mut pitch: f64 = scene_description.camera.orientation.pitch;
    let mut roll: f64 = scene_description.camera.orientation.roll;

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
            let mut should_reset = true;
            match e {
                sdl2::event::Event::KeyDown { keycode, .. } => match keycode {
                   Some(Keycode::Escape) => is_running = false,
                   Some(Keycode::Return) => (),
                   Some(Keycode::O) => roll -= 0.1,
                   Some(Keycode::U) => roll += 0.1,
                   Some(Keycode::I) => pitch -= 0.1,
                   Some(Keycode::K) => pitch += 0.1,
                   Some(Keycode::J) => yaw += 0.1,
                   Some(Keycode::L) => yaw -= 0.1,
                   Some(Keycode::W) => Arc::get_mut(&mut renderer.scene).expect("Can get mutable reference to scene").camera.distance_from_lens += 0.00001,
                   Some(Keycode::Q) => Arc::get_mut(&mut renderer.scene).expect("Can get mutable reference to scene").camera.distance_from_lens -= 0.00001,
                   _ => should_reset = false,
                },
                _ => should_reset = false,
            }

            if should_reset {
                println!("Resetting render");
                Arc::get_mut(&mut renderer.scene).expect("Can get mutable reference to scene").camera.set_orientation(yaw, pitch, roll);
                println!("Yaw: {:.1}, Pitch: {:.1}, Roll: {:.1}", yaw, pitch, roll);
                println!("F: {:.1}, V: {:.1}, A: {:.1}",
                         renderer.scene.camera.focal_length,
                         renderer.scene.camera.distance_from_lens,
                         renderer.scene.camera.aperture);
                renderer.reset();
                num_samples = 0;
            }
        }
    }
}
