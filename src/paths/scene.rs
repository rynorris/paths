use crate::paths::colour::Colour;
use crate::paths::vector::Vector3;
use crate::paths::Ray;

#[derive(Clone, Copy, Debug)]
pub struct Material {
    pub emittance: Colour,
    pub reflectance: Colour,
}

pub struct Object {
    pub shape: Box<dyn Shape>,
    pub material: Material,
}

#[derive(Clone, Copy, Debug)]
pub struct Collision {
    pub distance: f64,
    pub location: Vector3,
    pub normal: Vector3,
}

pub trait Shape {
    fn intersect(&self, ray: Ray) -> Option<Collision>;
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

pub struct Scene {
    pub objects: Vec<Object>,
}

impl Scene {
    pub fn find_intersection(&self, ray: Ray) -> Option<(Collision, Material)> {
        self.objects.iter()
            .map(|o| o.shape.intersect(ray).map(|col| (col, o.material)))
            .filter(|o| o.is_some())
            .map(|o| o.unwrap())
            .min_by(|(c1, _), (c2, _)| c1.distance.partial_cmp(&c2.distance).unwrap_or(std::cmp::Ordering::Equal))
    }
}
