use crate::paths::bvh::{construct_bvh_aac, AABB, BoundedVolume, BVH, Collision};
use crate::paths::camera::Camera;
use crate::paths::colour::Colour;
use crate::paths::material::Material;
use crate::paths::vector::Vector3;
use crate::paths::Ray;

#[derive(Clone)]
pub struct Object {
    pub shape: Box<dyn Shape>,
    pub material: Box<dyn Material>,
}

impl BoundedVolume for Object {
    fn aabb(&self) -> AABB {
        self.shape.aabb()
    }

    fn intersect(&self, ray: Ray) -> Option<Collision> {
        self.shape.intersect(ray)
    }
}

pub trait Shape : BoundedVolume + ShapeClone + Send + Sync {}

impl <T : 'static + BoundedVolume + Clone + Send + Sync> Shape for T {}

pub trait ShapeClone {
    fn clone_box(&self) -> Box<Shape>;
}

impl <T> ShapeClone for T where T: 'static + Shape + Clone {
    fn clone_box(&self) -> Box<Shape> {
        Box::new(self.clone())
    }
}

impl Clone for Box<Shape> {
    fn clone(&self) -> Box<Shape> {
        self.clone_box()
    }
}

pub trait Skybox : SkyboxClone + Send + Sync {
    fn ambient_light(&self, direction: Vector3) -> Colour;
}

pub trait SkyboxClone {
    fn clone_box(&self) -> Box<Skybox>;
}

impl <T> SkyboxClone for T where T: 'static + Skybox + Clone {
    fn clone_box(&self) -> Box<Skybox> {
        Box::new(self.clone())
    }
}

impl Clone for Box<Skybox> {
    fn clone(&self) -> Box<Skybox> {
        self.clone_box()
    }
}

#[derive(Clone, Debug)]
pub struct FlatSky {
    pub colour: Colour,
}

impl Skybox for FlatSky {
    fn ambient_light(&self, _direction: Vector3) -> Colour {
        self.colour
    }
}

#[derive(Clone, Debug)]
pub struct GradientSky {
    pub overhead_colour: Colour,
    pub horizon_colour: Colour,
}

impl Skybox for GradientSky {
    fn ambient_light(&self, direction: Vector3) -> Colour {
        let cos_theta = direction.dot(Vector3::new(0.0, 1.0, 0.0));
        return self.overhead_colour * cos_theta + self.horizon_colour * (1.0 - cos_theta);
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Sphere {
    pub center: Vector3,
    pub radius: f64,
}

impl BoundedVolume for Sphere {
    fn intersect(&self, ray: Ray) -> Option<Collision> {
        let c = self.center;
        let r = self.radius;
        let o = ray.origin;
        let l = ray.direction;

        let discriminant = (l.dot(o - c) * l.dot(o - c)) - (o - c).dot(o - c) + (r * r);

        if discriminant < 0.0 {
            return None
        }

        let tmp = -l.dot(o - c);
        let sqrt = discriminant.sqrt();
        let d1 = tmp + sqrt;
        let d2 = tmp - sqrt;

        if d1 < 0.0 {
            // Both intersections are "behind" the ray.
            return None
        }

        let distance = if d2 > 0.0 { d2 } else { d1 };
        let location = o + (l * distance);
        let normal = (location - c).normed();
        Some(Collision{ distance, location, normal, })
    }

    fn aabb(&self) -> AABB {
        let rad_vec = Vector3::new(self.radius, self.radius, self.radius);
        AABB::new(self.center - rad_vec, self.center + rad_vec)
    }
}

pub struct Scene {
    pub camera: Camera,
    pub skybox: Box<Skybox>,
    bvh: BVH<Object>,
}

impl Scene {
    pub fn new(camera: Camera, objects: Vec<Object>, skybox: Box<Skybox>) -> Scene {
        let bvh = construct_bvh_aac(objects);
        Scene { camera, skybox, bvh }
    }

    pub fn find_intersection(&self, ray: Ray) -> Option<(Collision, Box<Material>)> {
        self.bvh.find_intersection(ray).map(|(col, obj)| (col, obj.material.clone()))
    }
}
