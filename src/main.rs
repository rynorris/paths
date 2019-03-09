#[macro_use] extern crate serde_derive;

mod paths;

use std::fs::File;
use std::time::Instant;

use crate::paths::renderer::Renderer;
use crate::paths::serde::SceneDescription;

use sdl2::{event, pixels};
use sdl2::keyboard::Keycode;
use serde_yaml;

const SCALE: u32 = 1;

fn main() {
    // Load scene.
    let scene_filename = "scenes/bokeh_demo.yml";
    let scene_description: SceneDescription = {
        let scene_file = File::open(scene_filename).expect("Could open scene file");
        serde_yaml::from_reader(scene_file).expect("Could parse scene file")
    };
    let scene = scene_description.to_scene();

    // Initialize SDL and create window.
    let sdl_context = sdl2::init().unwrap();
    let video = sdl_context.video().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut main_window = video.window("Path Tracer", scene.camera.width * SCALE as u32, scene.camera.height * SCALE as u32)
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
    let mut output_texture = match texture_creator.create_texture_static(Some(pixels::PixelFormatEnum::RGB24), scene.camera.width, scene.camera.height) {
        Err(cause) => panic!("Failed to create texture: {}", cause),
        Ok(t) => t,
    };

    let mut yaw: f64 = scene_description.camera.yaw;
    let mut pitch: f64 = scene_description.camera.pitch;
    let mut roll: f64 = scene_description.camera.roll;

    let mut renderer = Renderer::new(scene, 4);

    let mut texture_buffer: Vec<u8> = vec![0; (scene.camera.width * scene.camera.height * 3) as usize];

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
        output_texture.update(None, texture_buffer.as_slice(), (scene.camera.height * 3) as usize).expect("Failed to update texture");
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
                   Some(Keycode::W) => renderer.scene.camera.distance_from_lens += 0.00001,
                   Some(Keycode::Q) => renderer.scene.camera.distance_from_lens -= 0.00001,
                   _ => (),
                },
                _ => (),
            }
            renderer.scene.camera.set_orientation(yaw, pitch, roll);
            println!("Yaw: {:.1}, Pitch: {:.1}, Roll: {:.1}", yaw, pitch, roll);
            println!("F: {:.1}, V: {:.1}, A: {:.1}",
                     renderer.scene.camera.focal_length,
                     renderer.scene.camera.distance_from_lens,
                     renderer.scene.camera.aperture);
        }
    }
}
