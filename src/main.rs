mod paths;

use crate::paths::Camera;
use crate::paths::colour::Colour;
use crate::paths::scene::{Material, Object, Scene, Sphere};
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
    camera.location.y = -450.0;
    camera.location.z = -450.0;
    camera.focal_length = 400.0;
    camera.set_orientation(0.0, 0.0, -0.8);

    let sphere1 = Object {
        shape: Box::new(Sphere{ center: Vector3::new(0.0, 1000.0, 200.0), radius: 1000.0 }),
        material: Material{
            emittance: Colour::BLACK,
            reflectance: Colour::rgb(0.8, 0.8, 0.8),
        },
    };

    let sphere2 = Object {
        shape: Box::new(Sphere{ center: Vector3::new(100.0, -100.0, 50.0), radius: 80.0 }),
        material: Material{
            emittance: Colour::BLACK,
            reflectance: Colour{ r: 1.0, g: 0.2, b: 0.2 },
        },
    };

    let light1 = Object {
        shape: Box::new(Sphere{ center: Vector3::new(1000.0, -1000.0, 100.0), radius: 800.0 }),
        material: Material{
            emittance: Colour{ r: 5.0, g: 5.0, b: 5.0 },
            reflectance: Colour::BLACK,
        },
    };

    let scene: Scene = Scene{ objects: vec![sphere1, sphere2, light1], ambient_light: Colour::rgb(0.05, 0.05, 0.05) };
    let mut renderer = Renderer::new(scene, camera);

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
