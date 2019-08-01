use std::f64::consts::PI;

use rand;
use rand::Rng;

use crate::colour::Colour;
use crate::vector::Vector3;

pub trait Material : MaterialClone + Send + Sync {
    fn weight_pdf(&self, vec_out: Vector3, vec_in: Vector3, normal: Vector3) -> f64;
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
    fn weight_pdf(&self, _vec_out: Vector3, _vec_in: Vector3, _normal: Vector3) -> f64 {
        1.0
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
        ((normal * normal.dot(vector) * 2) - vector).normed()
    }
}

impl Material for Mirror {
    fn weight_pdf(&self, _vec_out: Vector3, _vec_in: Vector3, _normal: Vector3) -> f64 {
        1.0
    }

    fn sample_pdf(&self, vec_out: Vector3, normal: Vector3) -> Vector3 {
        Mirror::reflect(vec_out, normal)
    }

    fn emittance(&self, _vec_out: Vector3, _cos_out: f64) -> Colour {
        Colour::BLACK
    }

    fn brdf(&self, _vec_out: Vector3, _vec_in: Vector3, _normal: Vector3) -> Colour {
        Colour::rgb(1.0, 1.0, 1.0)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Gloss {
    lambertian: Lambertian,
    mirror: Mirror,
    fresnel_r0: f64,
}

impl Gloss {
    pub fn new(albedo: Colour, reflectance: f64) -> Gloss {
        let n1: f64 = 1.0;  // Air
        let n2: f64 = reflectance;

        // Schlick's approximation for the fresnel factor.
        let r0 = ((n1 - n2) / (n1 + n2)).powf(2.0);
        Gloss {
            lambertian: Lambertian{ albedo, emittance: Colour::BLACK },
            mirror: Mirror{},
            fresnel_r0: r0,
        }
    }
}

impl Material for Gloss {
    fn weight_pdf(&self, vec_out: Vector3, _vec_in: Vector3, normal: Vector3) -> f64 {
        1.0
    }

    fn sample_pdf(&self, vec_out: Vector3, normal: Vector3) -> Vector3 {
        let cos_theta = vec_out.dot(normal);
        let r0 = self.fresnel_r0;
        let r = r0 + (1.0 - r0) * (1.0 - cos_theta).powf(5.0);

        if rand::thread_rng().gen::<f64>() > r {
            self.lambertian.sample_pdf(vec_out, normal)
        } else {
            self.mirror.sample_pdf(vec_out, normal)
        }
    }

    fn emittance(&self, _vec_out: Vector3, _cos_out: f64) -> Colour {
        Colour::BLACK
    }

    fn brdf(&self, vec_out: Vector3, _vec_in: Vector3, normal: Vector3) -> Colour {
        let cos_theta = vec_out.dot(normal);

        let r0 = self.fresnel_r0;
        let r = r0 + (1.0 - r0) * (1.0 - cos_theta).powf(5.0);

        self.lambertian.albedo * (1.0 - r) + Colour::rgb(1.0, 1.0, 1.0) * r
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CookTorrance {
    roughness: f64,
    albedo: Colour,
    refractive_index: f64,
}

impl CookTorrance {
    pub fn new(albedo: Colour, roughness: f64, refractive_index: f64) -> CookTorrance {
        CookTorrance { roughness,  albedo, refractive_index }
    }
}

impl Material for CookTorrance {
    fn weight_pdf(&self, vec_out: Vector3, vec_in: Vector3, normal: Vector3) -> f64 {
        let h = (vec_out - vec_in).normed();
        let cos_theta = h.dot(normal);
        let cos_phi = vec_out.dot(h);

        if cos_theta < 0.0 || cos_phi < 0.0 {
            return 0.0;
        }

        let theta = cos_theta.acos();
        let sin_theta = theta.sin();
        let tan_theta = theta.tan();
        let a = self.roughness;

        let exp = (-1.0 * (tan_theta * tan_theta) / (a * a)).exp();
        let p = (2.0 * sin_theta / (a * a * cos_theta.powf(3.0))) * exp;

        let pdf = p / (4.0 * vec_out.dot(h));

        if pdf < 0.0 {
            println!("ERROR: pdf={:.1}, p={:.1}, exp={:.1}, cos_theta={:.1}, sin_theta={:.1}, theta={:.1}", pdf, p, exp, cos_theta, sin_theta, theta);
            panic!();
        }

        pdf
    }

    fn sample_pdf(&self, vec_out: Vector3, normal: Vector3) -> Vector3 {
        // Sample a microfacet normal from the Beckmann distribution.
        // See https://agraphicsguy.wordpress.com/2015/11/01/sampling-microfacet-brdf/ for a
        // derivation.
        let mut rng = rand::thread_rng();
        let e = rng.gen::<f64>();
        let a = self.roughness;
        let theta = (a.powf(2.0) * (1.0 - e).ln() * -1.0).sqrt().atan();
        let phi = rng.gen::<f64>()  * 2.0 * PI;

        let sin_theta =  theta.sin();
        let cos_theta = theta.cos();

        let facet_normal = Vector3::new(
            sin_theta * phi.cos(),
            cos_theta,
            sin_theta * phi.sin(),
            );

        if cos_theta < 0.0 {
            println!("Invalid sample: cos_theta={:.1}", cos_theta);
            panic!();
        }

        let (i, j, k) = normal.form_basis();
        let world_facet_normal = to_basis(facet_normal, i, j, k).normed();

        let tmp = world_facet_normal.dot(normal);
        if tmp < 0.0 {
            println!("Basis transform fucked up: cos_theta before={:.1}, after={:.1}", cos_theta, tmp);
            panic!();
        }

        Mirror::reflect(vec_out, world_facet_normal)
    }

    fn emittance(&self, _vec_out: Vector3, _cos_out: f64) -> Colour {
        Colour::BLACK
    }

    fn brdf(&self, vec_out: Vector3, vec_in: Vector3, normal: Vector3) -> Colour {
        // In this function:
        //   h = half-angle = microfacet normal
        //   theta = angle between microfacet normal and surface normal
        //   phi = angle of incidence with microfacet normal
        let h = (vec_out - vec_in).normed();
        let cos_theta = h.dot(normal);
        let cos_phi = vec_out.dot(h);

        if vec_in.dot(h) > 0.0  || vec_in.dot(normal) > 0.0{
            return Colour::BLACK;
        }

        // Schlick's approximation for the fresnel factor.
        let n1 = 1.0;
        let n2 = self.refractive_index;
        let f0 = ((n1 - n2) / (n1 + n2)).powf(2.0);
        let f = f0 + (1.0 - f0) * (1.0 - cos_phi).powf(5.0);

        // Beckmann NDF.
        let theta = cos_theta.acos();
        let tan_theta = theta.tan();
        let a = self.roughness;

        let exp = (-1.0 * (tan_theta * tan_theta) / (a * a)).exp();
        let d = (1.0 / (PI * a * a * cos_theta.powf(4.0))) * exp;

        // Geometric term.
        let ndl = normal.dot(vec_out);
        let vdh = (vec_in * -1.0).dot(h);
        let ndh = normal.dot(h);
        let ndv = (vec_in * -1.0).dot(normal);
        let g = 1f64.min(((2.0 * ndh * ndv) / vdh).min((2.0 * ndh * ndl) / vdh));

        let c = self.albedo * (f * d * g) / (4.0 * ndv * ndh);

        /*
        if c.max() > 1.0 {
            println!("ERROR. f={:.1}, d={:.1}, g={:.1}, ndv={:.1}, ndh={:.1}, c={:?}", f, d, g, ndv, ndh, c);
            panic!();
        }
        */

        c
    }
}

