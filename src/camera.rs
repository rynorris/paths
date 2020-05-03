use crate::colour::Colour;
use crate::geom::Ray;
use crate::matrix::Matrix3;
use crate::vector::Vector3;

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
}

impl Camera {
    pub fn new(width: u32, height: u32) -> Camera {
        let camera = Camera {
            location: Vector3::new(0.0, 0.0, 0.0),
            focal_length: 9.86,
            distance_from_lens: 10.0,
            aperture: 2.0,
            rot: Matrix3::zero(),
            sensor_width: width as f64,
            sensor_height: height as f64,
            width,
            height,
        };
        camera
    }

    fn point_on_lens(&self, point_on_disk: (f64, f64)) -> Vector3 {
        let aperture_radius = self.focal_length / self.aperture;
        let (lens_x, lens_y) = point_on_disk;
        Vector3::new(lens_x * aperture_radius, lens_y * aperture_radius, 0.0)
    }

    pub fn get_ray_for_pixel(
        &self,
        mut x: u32,
        mut y: u32,
        point_on_square: (f64, f64),
        point_on_disk: (f64, f64)
    ) -> (Ray, f64) {
        // The image on a camera lens is flipped, so reflect x and y so that
        // we get the pixel that was actually asked for.
        x = self.width - x - 1;
        y = self.height - y - 1;
        
        // We'll compute the outbound ray first in lens-space where the centre of 
        // the lens is at the origin.
        // Then transform into world space.
        // This makes the refraction through the lens trivially computable.

        // Calculate distance to focal plane.
        let f = self.focal_length;
        let v = self.distance_from_lens;
        let p = (f * v) / (v - f);

        // k = point on sensor
        let (x_offset, y_offset) = point_on_square;
        let x_scale = self.sensor_width / (self.width as f64);
        let y_scale = self.sensor_height / (self.height as f64);
        let image_x = (x as f64) - (self.width as f64) / 2.0 + x_offset;
        // Flip y since image pixel coordinates start in the top-left but we want y pointing
        // upwards in 3d space.
        let image_y = (self.height as f64) / 2.0 - (y as f64) - y_offset;
        let k = Vector3::new(image_x * x_scale, image_y * y_scale, -self.distance_from_lens);

        // l = point on lens
        let l = self.point_on_lens(point_on_disk);

        // this equation for ray direction precomputed by hand to collapse all the terms that go away.
        let dir = ((k * (p/v)) + l) * -1;

        // Now transform into world space.
        let origin = self.rot.clone() * l + self.location;
        let direction = (self.rot.clone() * dir).normed();

        // Weight is d.n, but sinze n is just (0,0,1) we can shortcut.
        let weight = direction.z;

        (Ray::new(origin, direction), weight)
    }

    pub fn set_orientation(&mut self, yaw: f64, pitch: f64, roll: f64) {
        self.rot = Matrix3::rotation(yaw, pitch, roll);
    }
}
