#[macro_use] extern crate nom;
#[macro_use] extern crate serde_derive;

pub mod bvh;
pub mod camera;
pub mod colour;
pub mod controller;
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

use crate::controller::Controller;
use crate::renderer::Renderer;
use crate::serde::SceneDescription;

use sdl2;
use sdl2::keyboard::{Keycode, Scancode};
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
    let mouse = sdl_context.mouse();

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

    let location = camera.location;
    let orientation = camera.rot;
    let renderer = Renderer::new(camera, Arc::new(scene), 4);
    let mut controller = Controller::new(renderer, location, orientation);

    let mut texture_buffer: Vec<u8> = vec![0; (width * height * 3) as usize];

    let mut is_running = true;

    let start_time = Instant::now();

    let mut frame_count: u32 = 0;
    let frames_per_second: u32 = 60;
    let mut governer = timing::Governer::new(60);

    let mut camera_locked = true;

    controller.reset();
    while is_running {
        controller.update();
        let image = controller.render();

        let num_rays = controller.num_rays_cast();
        let rays_per_pixel = num_rays / num_pixels;
        let fps = governer.current_fps();
        if frame_count % frames_per_second == 0 {
            println!("[{:.1?}][{:.1}] Num rays: {} (avg {} per pixel)", start_time.elapsed(), fps, num_rays, rays_per_pixel);
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

        // Handle point-in-time key events.
        while let Some(e) = event_pump.poll_event() {
            match e {
                sdl2::event::Event::KeyDown { keycode, .. } => match keycode {
                   Some(Keycode::Escape) => is_running = false,
                   Some(Keycode::Return) => { 
                       camera_locked = !camera_locked;
                       mouse.set_relative_mouse_mode(!camera_locked);
                       mouse.show_cursor(camera_locked);
                       if !camera_locked {
                           controller.reset();
                       }
                   },
                   Some(Keycode::Q) => controller.rotate(0.0, 0.0, -0.1),
                   Some(Keycode::E) => controller.rotate(0.0, 0.0, 0.1),
                   _ => (),
                },
                sdl2::event::Event::MouseMotion { xrel, yrel, .. } => {
                    if !camera_locked {
                        controller.rotate((xrel as f64) / 100.0, (yrel as f64) / 100.0, 0.0);
                    }
                },
                _ => (),
            }
        }

        // Handle held keys.
        let mut velocity = vector::Vector3::new(0.0, 0.0, 0.0);
        let mut vw = 0.0;
        event_pump.keyboard_state().pressed_scancodes().for_each(|scancode| {
            match scancode {
                Scancode::W => velocity.z += 0.5,
                Scancode::S => velocity.z -= 0.5,
                Scancode::A => velocity.x -= 0.5,
                Scancode::D => velocity.x += 0.5,
                Scancode::Q => vw += 0.1,
                Scancode::E => vw -= 0.1,
                Scancode::LShift => velocity.y -= 0.5,
                Scancode::Space => velocity.y += 0.5,
                _ => (),
            }
        });
        if !camera_locked {
            controller.move_camera(velocity);
            if vw != 0.0 {
                controller.rotate(0.0, 0.0, vw);
            }
        }

        // Maintain 60fps.
        governer.end_frame();
        frame_count += 1;
    }

    // Shutdown.
    controller.shutdown();
}
