use std::f64::consts::PI;

use rand;
use rand::Rng;

use crate::colour::Colour;
use crate::geom;
use crate::vector::Vector3;


trait MaterialInterface {
    fn weight_pdf(&self, vec_out: Vector3, vec_in: Vector3, normal: Vector3) -> f64;
    fn sample_pdf(&self, vec_out: Vector3, normal: Vector3) -> Vector3;
    fn emittance(&self, vec_out: Vector3, cos_out: f64) -> Colour;
    fn brdf(&self, vec_out: Vector3, vec_in: Vector3, normal: Vector3) -> Colour;
}


#[derive(Clone, Copy, Debug)]
pub enum Material {
    Lambertian(LambertianMaterial),
    Mirror(MirrorMaterial),
    Gloss(GlossMaterial),
    CookTorrance(CookTorranceMaterial),
    FresnelCombination(FresnelCombinationMaterial),
}

#[derive(Clone, Copy, Debug)]
pub enum BasicMaterial {
    Lambertian(LambertianMaterial),
    Mirror(MirrorMaterial),
    Gloss(GlossMaterial),
    CookTorrance(CookTorranceMaterial),
}

impl Material {
    pub fn to_basic(self) -> BasicMaterial {
        match self {
            Material::Lambertian(mat) => BasicMaterial::Lambertian(mat),
            Material::Mirror(mat) => BasicMaterial::Mirror(mat),
            Material::Gloss(mat) => BasicMaterial::Gloss(mat),
            Material::CookTorrance(mat) => BasicMaterial::CookTorrance(mat),
            Material::FresnelCombination(_) => panic!("FresnelCombination material cannot be downcast to BasicMaterial"),
        }
    }

    pub fn lambertian(albedo: Colour, emittance: Colour) -> Material {
        Material::Lambertian(LambertianMaterial{ albedo, emittance })
    }

    pub fn mirror() -> Material {
        Material::Mirror(MirrorMaterial{})
    }

    pub fn gloss(albedo: Colour, reflectance: f64, metalness: f64) -> Material {
        Material::Gloss(GlossMaterial::new(albedo, reflectance, metalness))
    }

    pub fn cook_torrance(albedo: Colour, roughness: f64) -> Material {
        Material::CookTorrance(CookTorranceMaterial { roughness,  albedo })
    }

    pub fn fresnel_combination(diffuse: BasicMaterial, specular: BasicMaterial, refractive_index: f64) -> Material {
        Material::FresnelCombination(FresnelCombinationMaterial::new(diffuse, specular, refractive_index))
    }

    pub fn sample(&self, vec_out: Vector3, normal: Vector3) -> (Vector3, f64, Colour, bool) {
        match self {
            Material::Lambertian(mat) => mat.sample(vec_out, normal),
            Material::Mirror(mat) => mat.sample(vec_out, normal),
            Material::Gloss(mat) => mat.sample(vec_out, normal),
            _ => panic!("Not implemented"),
        }
    }

    pub fn weight_pdf(&self, vec_out: Vector3, vec_in: Vector3, normal: Vector3) -> f64 {
        match self {
            Material::Lambertian(mat) => mat.weight_pdf(vec_out, vec_in, normal),
            Material::Mirror(mat) => mat.weight_pdf(vec_out, vec_in, normal),
            Material::Gloss(mat) => mat.weight_pdf(vec_out, vec_in, normal),
            Material::CookTorrance(mat) => mat.weight_pdf(vec_out, vec_in, normal),
            Material::FresnelCombination(mat) => mat.weight_pdf(vec_out, vec_in, normal),
        }
    }

    pub fn sample_pdf(&self, vec_out: Vector3, normal: Vector3) -> Vector3 {
        match self {
            Material::Lambertian(mat) => mat.sample_pdf(vec_out, normal),
            Material::Mirror(mat) => mat.sample_pdf(vec_out, normal),
            Material::Gloss(mat) => mat.sample_pdf(vec_out, normal),
            Material::CookTorrance(mat) => mat.sample_pdf(vec_out, normal),
            Material::FresnelCombination(mat) => mat.sample_pdf(vec_out, normal),
        }
    }

    pub fn emittance(&self, vec_out: Vector3, cos_out: f64) -> Colour {
        match self {
            Material::Lambertian(mat) => mat.emittance(vec_out, cos_out),
            Material::Mirror(mat) => mat.emittance(vec_out, cos_out),
            Material::Gloss(mat) => mat.emittance(vec_out, cos_out),
            Material::CookTorrance(mat) => mat.emittance(vec_out, cos_out),
            Material::FresnelCombination(mat) => mat.emittance(vec_out, cos_out),
        }
    }

    pub fn brdf(&self, vec_out: Vector3, vec_in: Vector3, normal: Vector3) -> Colour {
        match self {
            Material::Lambertian(mat) => mat.brdf(vec_out, vec_in, normal),
            Material::Mirror(mat) => mat.brdf(vec_out, vec_in, normal),
            Material::Gloss(mat) => mat.brdf(vec_out, vec_in, normal),
            Material::CookTorrance(mat) => mat.brdf(vec_out, vec_in, normal),
            Material::FresnelCombination(mat) => mat.brdf(vec_out, vec_in, normal),
        }
    }
}

impl BasicMaterial {
    fn weight_pdf(&self, vec_out: Vector3, vec_in: Vector3, normal: Vector3) -> f64 {
        match self {
            BasicMaterial::Lambertian(mat) => mat.weight_pdf(vec_out, vec_in, normal),
            BasicMaterial::Mirror(mat) => mat.weight_pdf(vec_out, vec_in, normal),
            BasicMaterial::Gloss(mat) => mat.weight_pdf(vec_out, vec_in, normal),
            BasicMaterial::CookTorrance(mat) => mat.weight_pdf(vec_out, vec_in, normal),
        }
    }

    fn sample_pdf(&self, vec_out: Vector3, normal: Vector3) -> Vector3 {
        match self {
            BasicMaterial::Lambertian(mat) => mat.sample_pdf(vec_out, normal),
            BasicMaterial::Mirror(mat) => mat.sample_pdf(vec_out, normal),
            BasicMaterial::Gloss(mat) => mat.sample_pdf(vec_out, normal),
            BasicMaterial::CookTorrance(mat) => mat.sample_pdf(vec_out, normal),
        }
    }

    fn emittance(&self, vec_out: Vector3, cos_out: f64) -> Colour {
        match self {
            BasicMaterial::Lambertian(mat) => mat.emittance(vec_out, cos_out),
            BasicMaterial::Mirror(mat) => mat.emittance(vec_out, cos_out),
            BasicMaterial::Gloss(mat) => mat.emittance(vec_out, cos_out),
            BasicMaterial::CookTorrance(mat) => mat.emittance(vec_out, cos_out),
        }
    }

    fn brdf(&self, vec_out: Vector3, vec_in: Vector3, normal: Vector3) -> Colour {
        match self {
            BasicMaterial::Lambertian(mat) => mat.brdf(vec_out, vec_in, normal),
            BasicMaterial::Mirror(mat) => mat.brdf(vec_out, vec_in, normal),
            BasicMaterial::Gloss(mat) => mat.brdf(vec_out, vec_in, normal),
            BasicMaterial::CookTorrance(mat) => mat.brdf(vec_out, vec_in, normal),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct LambertianMaterial {
    albedo: Colour,
    emittance: Colour,
}

impl LambertianMaterial {
    pub fn sample(&self, vec_out: Vector3, normal: Vector3) -> (Vector3, f64, Colour, bool) {
        let direction = self.sample_pdf(vec_out, normal);
        let pdf = self.weight_pdf(vec_out, direction * -1, normal);
        let brdf = self.brdf(vec_out, direction * -1, normal);
        (direction, pdf, brdf, false)
    }
}

impl MaterialInterface for LambertianMaterial {
    fn weight_pdf(&self, _vec_out: Vector3, vec_in: Vector3, normal: Vector3) -> f64 {
        normal.dot(vec_in * -1) / PI
    }

    fn sample_pdf(&self, _vec_out: Vector3, normal: Vector3) -> Vector3 {
        let random_direction = geom::cosine_sample_hemisphere();

        let (i, j, k) = normal.form_basis();
        let world_direction = geom::switch_basis(random_direction, i, j, k);

        world_direction.normed()
    }

    fn emittance(&self, _vec_out: Vector3, _cos_out: f64) -> Colour {
        self.emittance
    }

    fn brdf(&self, _vec_out: Vector3, vec_in: Vector3, normal: Vector3) -> Colour {
        self.albedo * normal.dot(vec_in * -1) / PI
    }
}

#[derive(Clone, Copy, Debug)]
pub struct MirrorMaterial {}

impl MirrorMaterial {
    fn reflect(vector: Vector3, normal: Vector3) -> Vector3 {
        ((normal * normal.dot(vector) * 2) - vector).normed()
    }

    pub fn sample(&self, vec_out: Vector3, normal: Vector3) -> (Vector3, f64, Colour, bool) {
        (self.sample_pdf(vec_out, normal), 1.0, Colour::WHITE, true)
    }
}

impl MaterialInterface for MirrorMaterial {
    fn weight_pdf(&self, _vec_out: Vector3, _vec_in: Vector3, _normal: Vector3) -> f64 {
        1.0
    }

    fn sample_pdf(&self, vec_out: Vector3, normal: Vector3) -> Vector3 {
        MirrorMaterial::reflect(vec_out, normal)
    }

    fn emittance(&self, _vec_out: Vector3, _cos_out: f64) -> Colour {
        Colour::BLACK
    }

    fn brdf(&self, _vec_out: Vector3, _vec_in: Vector3, _normal: Vector3) -> Colour {
        // 0 chance of any particular ray.
        Colour::BLACK
    }
}

#[derive(Clone, Copy, Debug)]
pub struct GlossMaterial {
    lambertian: LambertianMaterial,
    mirror: MirrorMaterial,
    fresnel_r0: f64,
    metalness: f64,
}

impl GlossMaterial {
    pub fn new(albedo: Colour, reflectance: f64, metalness: f64) -> GlossMaterial {
        GlossMaterial {
            lambertian: LambertianMaterial{ albedo, emittance: Colour::BLACK },
            mirror: MirrorMaterial{},
            fresnel_r0: reflectance,
            metalness,
        }
    }

    // Returns (direction, pdf, brdf, is_specular)
    pub fn sample(&self, vec_out: Vector3, normal: Vector3) -> (Vector3, f64, Colour, bool) {
        let cos_theta = vec_out.dot(normal);
        let r0 = self.fresnel_r0;
        let r = r0 + (1.0 - r0) * (1.0 - cos_theta).powf(5.0);

        // For very reflective materials (e.g. metals) sample relative to reflectivity.
        // For less reflective materials (e.g. plastics) sample at 0.5 to capture highlights.
        let specular_chance = if r0 > 0.5 { r } else { 0.5 };
        let is_specular = rand::thread_rng().gen::<f64>() <= specular_chance;

        if is_specular {
            let direction = self.mirror.sample_pdf(vec_out, normal);
            let vec_in = direction * -1.0;
            let pdf = self.mirror.weight_pdf(vec_out, vec_in, normal);
            let brdf = self.lambertian.albedo * self.metalness + Colour::WHITE * (1.0 - self.metalness);
            (direction, pdf * specular_chance, brdf * r, is_specular)
        } else {
            let direction = self.lambertian.sample_pdf(vec_out, normal);
            let vec_in = direction * -1.0;
            let pdf = self.lambertian.weight_pdf(vec_out, vec_in, normal);
            let brdf = self.lambertian.brdf(vec_out, vec_in, normal) * (1.0 - self.metalness);
            (direction, pdf * (1.0 - specular_chance), brdf * (1.0 - r), is_specular)
        }
    }
}

impl MaterialInterface for GlossMaterial {
    fn weight_pdf(&self, vec_out: Vector3, vec_in: Vector3, normal: Vector3) -> f64 {
        let cos_theta = vec_out.dot(normal);

        let r0 = self.fresnel_r0;
        let r = r0 + (1.0 - r0) * (1.0 - cos_theta).powf(5.0);

        // Probability of this ray given diffuse scattering.
        let diffuse = self.lambertian.weight_pdf(vec_out, vec_in, normal);

        // Probability of this ray given specular reflection.
        let specular = self.mirror.weight_pdf(vec_out, vec_in, normal);

        diffuse * (1.0 - r) * specular * r
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

    fn brdf(&self, vec_out: Vector3, vec_in: Vector3, normal: Vector3) -> Colour {
        let cos_theta = vec_out.dot(normal);

        let r0 = self.fresnel_r0;
        let r = r0 + (1.0 - r0) * (1.0 - cos_theta).powf(5.0);

        let diffuse = self.lambertian.brdf(vec_out, vec_in, normal) * (1.0 - self.metalness);
        let specular = self.mirror.brdf(vec_out, vec_in, normal);

        diffuse * (1.0 - r) + specular * r
    }
}

#[derive(Clone, Copy, Debug)]
pub struct FresnelCombinationMaterial {
    diffuse: BasicMaterial,
    specular: BasicMaterial,
    fresnel_r0: f64,
}

impl FresnelCombinationMaterial {
    pub fn new(diffuse: BasicMaterial, specular: BasicMaterial, refractive_index: f64) -> FresnelCombinationMaterial {
        // Schlick's approximation for the fresnel factor.
        let n1: f64 = 1.0;  // Air
        let n2: f64 = refractive_index;
        let fresnel_r0 = ((n1 - n2) / (n1 + n2)).powf(2.0);

        FresnelCombinationMaterial { diffuse, specular, fresnel_r0 }
    }

    fn fresnel_weight(&self, vec_out: Vector3, normal: Vector3) -> f64 {
        let cos_theta = vec_out.dot(normal);
        let r0 = self.fresnel_r0;
        r0 + (1.0 - r0) * (1.0 - cos_theta).powf(5.0)
    }
}

impl MaterialInterface for FresnelCombinationMaterial {
    fn weight_pdf(&self, vec_out: Vector3, vec_in: Vector3, normal: Vector3) -> f64 {
        let r = self.fresnel_weight(vec_out, normal);
        let diffuse_weight = self.diffuse.weight_pdf(vec_out, vec_in, normal);
        let specular_weight = self.specular.weight_pdf(vec_out, vec_in, normal);
        diffuse_weight * (1.0 - r) + specular_weight * r
    }

    fn sample_pdf(&self, vec_out: Vector3, normal: Vector3) -> Vector3 {
        let r = self.fresnel_weight(vec_out, normal);

        if rand::thread_rng().gen::<f64>() > r {
            self.diffuse.sample_pdf(vec_out, normal)
        } else {
            self.specular.sample_pdf(vec_out, normal)
        }
    }

    fn emittance(&self, vec_out: Vector3, cos_out: f64) -> Colour {
        // Specular emittance doesn't make sense?
        self.diffuse.emittance(vec_out, cos_out)
    }

    fn brdf(&self, vec_out: Vector3, vec_in: Vector3, normal: Vector3) -> Colour {
        let r = self.fresnel_weight(vec_out, normal);

        let diffuse_brdf = self.diffuse.brdf(vec_out, vec_in, normal);
        let specular_brdf = self.specular.brdf(vec_out, vec_in, normal);

        diffuse_brdf * (1.0 - r) + specular_brdf * r
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CookTorranceMaterial {
    roughness: f64,
    albedo: Colour,
}

impl CookTorranceMaterial {
    fn ndf(&self, n: Vector3, h: Vector3) -> f64 {
        // Beckmann NDF.
        let alpha = h.dot(n).acos();
        let cos_alpha = alpha.cos();
        let tan_alpha = alpha.tan();
        let m = self.roughness;

        let exp = (-1.0 * (tan_alpha * tan_alpha) / (m * m)).exp();
        let d0 = exp / (PI * m * m * cos_alpha.powf(4.0));
        0f64.max(d0 * n.dot(h))
    }
}

impl MaterialInterface for CookTorranceMaterial {
    fn weight_pdf(&self, vec_out: Vector3, vec_in: Vector3, normal: Vector3) -> f64 {
        let h = (vec_out - vec_in).normed();
        let d = self.ndf(normal, h);
        let p = (d * normal.dot(h).abs()) / (4.0 * vec_out.dot(h).abs());

        if p < 0.0 {
            println!("Negative PDF! d={:.1}, p={:.1}", d, p);
            panic!();
        }

        p

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
        let world_facet_normal = geom::switch_basis(facet_normal, i, j, k).normed();

        let tmp = world_facet_normal.dot(normal);
        if tmp < 0.0 {
            println!("Basis transform fucked up: cos_theta before={:.1}, after={:.1}", cos_theta, tmp);
            panic!();
        }

        MirrorMaterial::reflect(vec_out, world_facet_normal)
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

        let d = self.ndf(normal, h);

        // Geometric term.
        let ndl = normal.dot(vec_in * -1.0);
        let vdh = vec_out.dot(h);
        let ndh = normal.dot(h);
        let ndv = normal.dot(vec_out);
        let g = 0f64.max(1f64.min(((2.0 * ndh * ndv) / vdh).min((2.0 * ndh * ndl) / vdh)));

        // Specular component.
        self.albedo * (d * g) / (4.0 * ndv * ndl)
    }
}

