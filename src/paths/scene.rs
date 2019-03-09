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

#[derive(Clone, Copy, Debug)]
pub struct Collision {
    pub distance: f64,
    pub location: Vector3,
    pub normal: Vector3,
}

pub trait Shape : ShapeClone + Send {
    fn intersect(&self, ray: Ray) -> Option<Collision>;
}

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

pub trait Skybox : SkyboxClone + Send {
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

#[derive(Clone)]
pub struct Scene {
    pub camera: Camera,
    pub objects: Vec<Object>,
    pub skybox: Box<Skybox>,
}

#[derive(Clone, Copy, Debug)]
pub struct Sphere {
    pub center: Vector3,
    pub radius: f64,
}

impl Shape for Sphere {
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
}

impl Scene {
    pub fn find_intersection(&self, ray: Ray) -> Option<(Collision, Box<Material>)> {
        self.objects.iter()
            .map(|o| o.shape.intersect(ray).map(|col| (col, o.material.clone())))
            .filter(|o| o.is_some())
            .map(|o| o.unwrap())
            .min_by(|(c1, _), (c2, _)| c1.distance.partial_cmp(&c2.distance).unwrap_or(std::cmp::Ordering::Equal))
    }
}
