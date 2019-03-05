use std::f64::consts::PI;

use rand;
use rand::Rng;

use crate::paths::colour::Colour;
use crate::paths::vector::Vector3;

pub trait Material : MaterialClone + Send {
    fn weight_pdf(&self, vec_out: Vector3, normal: Vector3) -> Colour;
    fn sample_pdf(&self, vec_out: Vector3, normal: Vector3) -> Vector3;
    fn emittance(&self, vec_out: Vector3, cos_out: f64) -> Colour;
    fn brdf(&self, vec_out: Vector3, vec_in: Vector3, normal: Vector3) -> Colour;
}

pub trait MaterialClone {
    fn clone_box(&self) -> Box<Material>;
}

impl <T> MaterialClone for T where T: 'static + Material + Clone {
    fn clone_box(&self) -> Box<Material> {
        Box::new(self.clone())
    }
}

impl Clone for Box<Material> {
    fn clone(&self) -> Box<Material> {
        self.clone_box()
    }
}

fn to_basis(v: Vector3, i: Vector3, j: Vector3, k: Vector3) -> Vector3 {
    i* v.x + j * v.y + k * v.z
}

#[derive(Clone, Copy, Debug)]
pub struct Lambertian {
    albedo: Colour,
    emittance: Colour,
}

impl Lambertian {
    pub fn new(albedo: Colour, emittance: Colour) -> Lambertian {
        Lambertian{ albedo, emittance }
    }
}

impl Material for Lambertian {
    fn weight_pdf(&self, _vec_out: Vector3, _normal: Vector3) -> Colour {
        self.albedo
    }

    fn sample_pdf(&self, _vec_out: Vector3, normal: Vector3) -> Vector3 {
        let mut rng = rand::thread_rng();
        let seed = rng.gen::<f64>();  // between 0 and 1.

        let sin2_theta = seed;
        let cos2_theta = 1.0 - sin2_theta;
        let cos_theta = cos2_theta.sqrt();
        let sin_theta = sin2_theta.sqrt();
        let orientation = rng.gen::<f64>() * PI * 2.0;

        let random_direction = Vector3::new(
            sin_theta * orientation.cos(),
            cos_theta,
            sin_theta * orientation.sin(),
            );

        let (i, j, k) = normal.form_basis();
        let world_direction = to_basis(random_direction, i, j, k);

        world_direction
    }

    fn emittance(&self, _vec_out: Vector3, _cos_out: f64) -> Colour {
        self.emittance
    }

    fn brdf(&self, _vec_out: Vector3, _vec_in: Vector3, _normal: Vector3) -> Colour {
        self.albedo
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Mirror {}

impl Mirror {
    fn reflect(vector: Vector3, normal: Vector3) -> Vector3 {
        (normal * normal.dot(vector) * 2) - vector
    }
}

impl Material for Mirror {
    fn weight_pdf(&self, vec_out: Vector3, normal: Vector3) -> Colour {
        let cos_theta = vec_out.dot(normal);

        let n1: f64 = 1.0;  // Air
        let n2: f64 = 1.5;  // Glass

        // Schlick's approximation for the fresnel factor.
        let r0 = ((n1 - n2) / (n1 + n2)).powf(2.0);
        let r = r0 + (1.0 - r0) * (1.0 - cos_theta).powf(5.0);
        Colour::rgb(1.0, 1.0, 1.0) * (1.0 - r)
    }

    fn sample_pdf(&self, vec_out: Vector3, normal: Vector3) -> Vector3 {
        Mirror::reflect(vec_out, normal)
    }

    fn emittance(&self, _vec_out: Vector3, _cos_out: f64) -> Colour {
        Colour::BLACK
    }

    fn brdf(&self, _vec_out: Vector3, _vec_in: Vector3, _normal: Vector3) -> Colour {
        Colour::BLACK
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Test {
    lambertian: Lambertian,
    mirror: Mirror,
    reflectance: f64,
}

impl Test {
    pub fn new(albedo: Colour, reflectance: f64) -> Test {
        Test {
            lambertian: Lambertian{ albedo, emittance: Colour::BLACK },
            mirror: Mirror{},
            reflectance: reflectance,
        }
    }
}

impl Material for Test {
    fn weight_pdf(&self, vec_out: Vector3, normal: Vector3) -> Colour {
        let cos_theta = vec_out.dot(normal);

        let n1: f64 = 1.0;  // Air
        let n2: f64 = 1.5;  // Glass

        // Schlick's approximation for the fresnel factor.
        let r0 = ((n1 - n2) / (n1 + n2)).powf(2.0);
        let r = r0 + (1.0 - r0) * (1.0 - cos_theta).powf(5.0);
        self.lambertian.albedo * (1.0 - r) + Colour::rgb(1.0, 1.0, 1.0) * r
    }

    fn sample_pdf(&self, vec_out: Vector3, normal: Vector3) -> Vector3 {
        let random_ray = self.lambertian.sample_pdf(vec_out, normal);
        let mirror_ray = self.mirror.sample_pdf(vec_out, normal);
        let p = self.reflectance;
        ((random_ray * (1.0 - p)) + (mirror_ray * p)).normed()
    }

    fn emittance(&self, _vec_out: Vector3, _cos_out: f64) -> Colour {
        Colour::BLACK
    }

    fn brdf(&self, _vec_out: Vector3, _vec_in: Vector3, _normal: Vector3) -> Colour {
        self.lambertian.albedo
    }
}
