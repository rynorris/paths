pub mod colour;
pub mod material;
pub mod matrix;
pub mod pixels;
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

#[derive(Clone)]
pub struct Camera {
    pub location: Vector3,  // Center of camera sensor.
    pub focal_length: f64,
    pub lens_radius: f64,
    pub distance_from_lens: f64,
    rot: Matrix3,
    pub sensor_width: f64,
    pub sensor_height: f64,
    pub width: u32,
    pub height: u32,
}

impl Camera {
    pub fn new(width: u32, height: u32) -> Camera {
        let camera = Camera {
            location: Vector3::new(0.0, 0.0, 0.0),
            focal_length: 1000.0,
            lens_radius: 1000.0,
            distance_from_lens: 100.0,
            rot: Matrix3::zero(),
            sensor_width: width as f64,
            sensor_height: height as f64,
            width,
            height,
        };
        camera
    }

    pub fn get_ray_for_pixel(&self, x: u32, y: u32) -> Ray {
        let mut rng = rand::thread_rng();

        // We'll compute the outbound ray first in lens-space where the centre of 
        // the lens is at the origin.
        // Then transform into world space.
        // This makes the refraction through the lens trivially computable.
        let x_offset: f64 = (x as f64) - ((self.width as f64) / 2.0) + rng.gen::<f64>();
        let y_offset: f64 = (y as f64) - ((self.height as f64) / 2.0) + rng.gen::<f64>();

        // Calculate distance to focal plane.
        let f = self.focal_length;
        let v = self.distance_from_lens;
        let p = (f * v) / (v - f);

        // k = point on sensor
        let x_scale = self.sensor_width / (self.width as f64);
        let y_scale = self.sensor_height / (self.height as f64);
        let k = Vector3::new(x_offset * x_scale, y_offset * y_scale, -self.distance_from_lens);

        // l = point on lens
        let theta = rng.gen::<f64>();
        let r = rng.gen::<f64>() * self.lens_radius;
        let l = Vector3::new(r * theta.cos(), r * theta.sin(), 0.0);

        // this equation for ray direction precomputed by hand to collapse all the terms that go away.
        let dir = ((k * (p/v)) + l) * -1;

        // Now transform into world space.
        let origin = self.rot.clone() * l + self.location;
        let direction = (self.rot.clone() * dir).normed();

        Ray { origin, direction }
    }

    pub fn set_orientation(&mut self, yaw: f64, pitch: f64, roll: f64) {
        self.rot = Matrix3::rotation(yaw, pitch, roll);
    }
}
