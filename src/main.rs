#[macro_use] extern crate serde_derive;

mod paths;

use std::fs::File;
use std::time::Instant;

use crate::paths::Camera;
use crate::paths::sampling::{CorrelatedMultiJitteredSampler, UniformSampler};
use crate::paths::renderer::Renderer;
use crate::paths::serde::SceneDescription;

use sdl2::{event, pixels};
use sdl2::keyboard::Keycode;
use serde_yaml;

const WIDTH: u32 = 720;
const HEIGHT: u32 = 480;
const SCALE: u32 = 1;

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video = sdl_context.video().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut main_window = video.window("Path Tracer", WIDTH * SCALE as u32, HEIGHT * SCALE as u32)
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
    let mut output_texture = match texture_creator.create_texture_static(Some(pixels::PixelFormatEnum::RGB24), WIDTH, HEIGHT) {
        Err(cause) => panic!("Failed to create texture: {}", cause),
        Ok(t) => t,
    };

    let mut camera = Camera::new(WIDTH, HEIGHT, Box::new(CorrelatedMultiJitteredSampler::new(42, 8, 8)));
   
    // All distances in m;
    camera.location.x = 0.0;
    camera.location.y = -0.01;
    camera.location.z = -15.0;

    // Full frame DSLR sensor.
    camera.sensor_width = 0.036;
    camera.sensor_height = 0.024;

    // 50mm lens.
    camera.focal_length = 0.05;

    // Focus on back sphere
    let focus_distance = 1.0;//(2f64 * 2f64 + 40f64 * 40f64).sqrt();
    camera.distance_from_lens = (camera.focal_length * focus_distance) / (focus_distance - camera.focal_length);

    // f stop
    camera.aperture = 2.0;

    let mut yaw: f64 = -0.0;
    let mut pitch: f64 = -0.0;
    let mut roll: f64 = -0.0;
    camera.set_orientation(yaw, pitch, roll);

    // Load scene.
    let scene_filename = "scenes/bokeh_demo.yml";
    let scene_file = File::open(scene_filename).expect("Could open scene file");
    let scene_description: SceneDescription = serde_yaml::from_reader(scene_file).expect("Could parse scene file");
    let scene = scene_description.to_scene();

    let mut renderer = Renderer::new(scene, camera, 4);

    let mut texture_buffer: Vec<u8> = vec![0; (WIDTH * HEIGHT * 3) as usize];

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
        output_texture.update(None, texture_buffer.as_slice(), (WIDTH * 3) as usize).expect("Failed to update texture");
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
                   Some(Keycode::W) => renderer.camera.distance_from_lens += 0.00001,
                   Some(Keycode::Q) => renderer.camera.distance_from_lens -= 0.00001,
                   _ => (),
                },
                _ => (),
            }
            renderer.camera.set_orientation(yaw, pitch, roll);
            println!("Yaw: {:.1}, Pitch: {:.1}, Roll: {:.1}", yaw, pitch, roll);
            println!("F: {:.1}, V: {:.1}, A: {:.1}", renderer.camera.focal_length, renderer.camera.distance_from_lens, renderer.camera.aperture);
        }
    }
}
