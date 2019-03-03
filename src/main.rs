mod paths;

use crate::paths::Camera;
use crate::paths::scene::Scene;
use crate::paths::renderer::Renderer;

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

    let camera = Camera::new(WIDTH, HEIGHT);
    let scene: Scene = Scene{ objects: vec![], };
    let mut renderer = Renderer::new(scene, camera);

    let mut texture_buffer: Vec<u8> = vec![0; (WIDTH * HEIGHT * 3) as usize];

    let mut is_running = true;

    while is_running {
        renderer.trace_rays(1000);
        let image = renderer.render();

        for ix in 0 .. image.pixels.len() {
            let colour = image.pixels[ix];
            texture_buffer[ix * 3] = (colour.r * 256.0) as u8;
            texture_buffer[ix * 3 + 1] = (colour.g * 256.0) as u8;
            texture_buffer[ix * 3 + 2] = (colour.b * 256.0) as u8;
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
