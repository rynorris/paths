pub mod colour;
pub mod renderer;

use self::colour::Colour;

pub struct Vector3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

pub struct Ray {
    pub origin: Vector3,
    pub direction: Vector3,
}

pub struct Image {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<Colour>,
}

pub struct Material {
    pub emittance: Colour,
    pub reflectance: Colour,
}

pub struct Object {
    pub material: Material,
}

pub struct Scene {
    pub objects: Vec<Object>,
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
        Colour::BLACK
    }
}
