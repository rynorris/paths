pub mod camera;
pub mod colour;
pub mod material;
pub mod matrix;
pub mod pixels;
pub mod renderer;
pub mod sampling;
pub mod scene;
pub mod serde;
pub mod vector;

use std::f64::consts::PI;

use rand;
use rand::Rng;

use self::colour::Colour;
use self::matrix::Matrix3;
use self::vector::Vector3;

#[derive(Clone, Copy, Debug)]
pub struct Ray {
    pub origin: Vector3,
    pub direction: Vector3,
}

impl Ray {
    pub fn random_in_hemisphere(&self) -> Ray {
        let mut rng = rand::thread_rng();
        let yaw = (rng.gen::<f64>() - 0.5) * PI;
        let pitch = (rng.gen::<f64>() - 0.5) * PI;
        let roll = (rng.gen::<f64>() - 0.5) * PI;
        let rot = Matrix3::rotation(yaw, pitch, roll);
        Ray {
            origin: self.origin,
            direction: rot * self.direction,
        }
    }
}

pub struct Image {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<Colour>,
}
