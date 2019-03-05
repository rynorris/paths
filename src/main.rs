mod paths;

use crate::paths::Camera;
use crate::paths::colour::Colour;
use crate::paths::material::{Lambertian, Mirror};
use crate::paths::scene::{Object, Scene, Sphere};
use crate::paths::renderer::Renderer;
use crate::paths::vector::Vector3;

use sdl2::{event, pixels};
use sdl2::keyboard::Keycode;

const WIDTH: u32 = 640;
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

    let mut camera = Camera::new(WIDTH, HEIGHT);
    camera.location.x = 0.0;
    camera.location.y = -300.0;
    camera.location.z = -300.0;
    camera.focal_length = 300.0;
    camera.set_orientation(0.0, -0.1, -0.6);

    let objects = vec![
        // Objects
        Object {
            shape: Box::new(Sphere{ center: Vector3::new(100.0, -0.0, -0.0), radius: 100.0 }),
            material: Box::new(Mirror{}),
        },

        // Walls
        Object {
            shape: Box::new(Sphere{ center: Vector3::new(100_500.0, 0.0, 0.0), radius: 100_000.0 }),
            material: Box::new(Lambertian::new(Colour::rgb(0.3, 0.3, 0.7), Colour::rgb(0.0, 0.0, 0.0))),
        },
        Object {
            shape: Box::new(Sphere{ center: Vector3::new(-100_500.0, 0.0, 0.0), radius: 100_000.0 }),
            material: Box::new(Lambertian::new(Colour::rgb(0.7, 0.3, 0.3), Colour::rgb(0.0, 0.0, 0.0))),
        },
        Object {
            shape: Box::new(Sphere{ center: Vector3::new(0.0, 100_500.0, 0.0), radius: 100_000.0 }),
            material: Box::new(Lambertian::new(Colour::rgb(0.7, 0.7, 0.7), Colour::rgb(0.0, 0.0, 0.0))),
        },
        Object {
            shape: Box::new(Sphere{ center: Vector3::new(0.0, -100_500.0, 0.0), radius: 100_000.0 }),
            material: Box::new(Lambertian::new(Colour::rgb(0.7, 0.7, 0.7), Colour::rgb(0.0, 0.0, 0.0))),
        },
        Object {
            shape: Box::new(Sphere{ center: Vector3::new(0.0, 0.0, 100_500.0), radius: 100_000.0 }),
            material: Box::new(Lambertian::new(Colour::rgb(0.7, 0.7, 0.7), Colour::rgb(0.0, 0.0, 0.0))),
        },
        Object {
            shape: Box::new(Sphere{ center: Vector3::new(0.0, 0.0, -100_500.0), radius: 100_000.0 }),
            material: Box::new(Lambertian::new(Colour::rgb(0.7, 0.7, 0.7), Colour::rgb(0.0, 0.0, 0.0))),
        },

        // Lights
        Object {
            shape: Box::new(Sphere{ center: Vector3::new(0.0, -500.0, -0.0), radius: 150.0 }),
            material: Box::new(Lambertian::new(Colour::rgb(0.0, 0.0, 0.0), Colour::rgb(7.0, 7.0, 7.0))),
        },
        ];

    let scene: Scene = Scene{ objects, ambient_light: Colour::rgb(0.0, 0.0, 0.0) };
    let mut renderer = Renderer::new(scene, camera, 4);

    let mut texture_buffer: Vec<u8> = vec![0; (WIDTH * HEIGHT * 3) as usize];

    let mut is_running = true;

    let mut num_samples = 0;

    while is_running {
        renderer.trace_full_pass();
        let image = renderer.render();

        num_samples += 1;
        println!("Num samples: {:?}", num_samples);

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
                   _ => (),
                },
                _ => (),
            }
        }
    }
}
