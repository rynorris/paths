pub mod colour;
pub mod material;
pub mod matrix;
pub mod pixels;
pub mod renderer;
pub mod sampling;
pub mod scene;
pub mod vector;

use std::f64::consts::PI;

use rand;
use rand::Rng;

use self::colour::Colour;
use self::matrix::Matrix3;
use self::sampling::Sampler;
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
    pub distance_from_lens: f64,
    pub aperture: f64,
    rot: Matrix3,
    pub sensor_width: f64,
    pub sensor_height: f64,
    pub width: u32,
    pub height: u32,
    sampler: Box<dyn Sampler>,

    // Current bundle details.
    bundle_offsets: (f64, f64),
    bundle_lens_point: Vector3,
}

impl Camera {
    pub fn new(width: u32, height: u32, sampler: Box<dyn Sampler>) -> Camera {
        let mut camera = Camera {
            location: Vector3::new(0.0, 0.0, 0.0),
            focal_length: 9.86,
            distance_from_lens: 10.0,
            aperture: 2.0,
            rot: Matrix3::zero(),
            sensor_width: width as f64,
            sensor_height: height as f64,
            width,
            height,
            sampler,

            bundle_offsets: (0.0, 0.0),
            bundle_lens_point: Vector3::new(0.0, 0.0, 0.0),
        };
        camera.init_bundle();
        camera
    }

    pub fn init_bundle(&mut self) {
        let (jx, jy) = self.sampler.sample_square();
        self.bundle_offsets = (jx, jy);

        let aperture_radius = self.focal_length / self.aperture;
        let (lens_x, lens_y) = self.sampler.sample_disk();
        self.bundle_lens_point = Vector3::new(lens_x * aperture_radius, lens_y * aperture_radius, 0.0);
    }

    pub fn get_ray_for_pixel(&mut self, x: u32, y: u32) -> Ray {
        // We'll compute the outbound ray first in lens-space where the centre of 
        // the lens is at the origin.
        // Then transform into world space.
        // This makes the refraction through the lens trivially computable.

        // Calculate distance to focal plane.
        let f = self.focal_length;
        let v = self.distance_from_lens;
        let p = (f * v) / (v - f);

        // k = point on sensor
        let (x_offset, y_offset) = self.bundle_offsets;
        let x_scale = self.sensor_width / (self.width as f64);
        let y_scale = self.sensor_height / (self.height as f64);
        let image_x = (x as f64) - (self.width as f64) / 2.0 + x_offset;
        let image_y = (y as f64) - (self.height as f64) / 2.0 + y_offset;
        let k = Vector3::new(image_x * x_scale, image_y * y_scale, -self.distance_from_lens);

        // l = point on lens
        let l = self.bundle_lens_point;

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
