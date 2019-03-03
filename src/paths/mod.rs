pub mod colour;
pub mod renderer;
pub mod scene;
pub mod vector;

use self::colour::Colour;
use self::vector::Vector3;

#[derive(Clone, Copy, Debug)]
pub struct Ray {
    pub origin: Vector3,
    pub direction: Vector3,
}

pub struct Image {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<Colour>,
}

pub struct Camera {
    pub x: f64,
    pub y: f64,
    pub width: u32,
    pub height: u32,
}

impl Camera {
    pub fn new(width: u32, height: u32) -> Camera {
        Camera {
            x: 0.0,
            y: 0.0,
            width,
            height,
        }
    }

    pub fn trace_ray(&self, x: u32, y: u32) -> Colour {
        Colour::random()
    }
}
