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
pub mod timing;
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

    let camera = scene_description.camera();
    let scene = scene_description.scene();

    println!("Contructing scene...");
    let width = scene_description.camera.image_width;
    let height = scene_description.camera.image_height;
    let num_pixels = (width * height) as u64;

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

    let mut renderer = Renderer::new(camera, Arc::new(scene), 4);

    let mut texture_buffer: Vec<u8> = vec![0; (width * height * 3) as usize];

    let mut is_running = true;

    let start_time = Instant::now();

    let mut frame_count: u32 = 0;
    let frames_per_second: u32 = 60;
    let mut governer = timing::Governer::new(60);

    renderer.fill_request_queue();
    while is_running {
        renderer.drain_result_queue();
        renderer.fill_request_queue();
        let image = renderer.render();

        let num_rays = renderer.num_rays_cast();
        let rays_per_pixel = num_rays / num_pixels;
        if frame_count % frames_per_second == 0 {
            println!("[{:.1?}] Num rays: {} (avg {} per pixel)", start_time.elapsed(), num_rays, rays_per_pixel);
        }

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
                   _ => should_reset = false,
                },
                _ => should_reset = false,
            }

            if should_reset {
                renderer.reorient_camera(yaw, pitch, roll);
            }
        }

        // Maintain 60fps.
        governer.end_frame();
        frame_count += 1;
    }

    // Shutdown.
    renderer.shutdown();
}
