pub mod colour;
pub mod matrix;
pub mod renderer;
pub mod scene;
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

pub struct Camera {
    pub location: Vector3,  // Center of camera sensor.
    pub focal_length: f64,
    pub yaw: f64,  // Radians
    pub pitch: f64,    // Radians
    pub roll: f64,   // Radians
    x_vec: Vector3,
    y_vec: Vector3,
    direction: Vector3,
    pub width: u32,
    pub height: u32,
}

impl Camera {
    pub fn new(width: u32, height: u32) -> Camera {
        let mut camera = Camera {
            location: Vector3::new(0.0, 0.0, 0.0),
            focal_length: 10.0,
            yaw: 0.0,
            pitch: 0.0,
            roll: 0.0,
            x_vec: Vector3::new(0.0, 0.0, 0.0),
            y_vec: Vector3::new(0.0, 0.0, 0.0),
            direction: Vector3::new(0.0, 0.0, 0.0),
            width,
            height,
        };
        camera.recompute();
        camera
    }

    pub fn get_ray_for_pixel(&self, x: u32, y: u32) -> Ray {
        let x_offset = (x as i32) - (self.width as i32) / 2;
        let y_offset = (y as i32) - (self.height as i32) / 2;

        Ray { 
            origin: self.location,
            direction: ((self.direction * self.focal_length) + (self.x_vec * x_offset) + (self.y_vec * y_offset)).normed(),
        }
    }

    pub fn set_orientation(&mut self, yaw: f64, pitch: f64, roll: f64) {
        self.yaw = yaw;
        self.pitch = pitch;
        self.roll = roll;
        self.recompute();
    }

    fn recompute(&mut self) {
        let i = Vector3::new(1.0, 0.0, 0.0);
        let j = Vector3::new(0.0, 1.0, 0.0);
        let k = Vector3::new(0.0, 0.0, 1.0);
        let rot = Matrix3::rotation(self.yaw, self.pitch, self.roll);
        self.x_vec = rot.clone() * i;
        self.y_vec = rot.clone() * j;
        self.direction = rot.clone() * k;
    }
}
