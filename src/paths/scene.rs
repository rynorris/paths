use crate::paths::bvh::{construct_bvh_aac, AABB, BoundedVolume, BVH, Collision};
use crate::paths::camera::Camera;
use crate::paths::colour::Colour;
use crate::paths::material::Material;
use crate::paths::matrix::Matrix3;
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

#[derive(Clone, Copy, Debug)]
pub struct Triangle {
    pub vertices: [Vector3; 3],
    pub surface_normal: Vector3,
    pub vertex_normals: [Vector3; 3],
}

impl Triangle {
    pub fn transform(&self, translation: Vector3, rotation: Matrix3, scale: f64) -> Triangle {
        Triangle {
            vertices: [
                rotation.clone() * self.vertices[0] * scale + translation,
                rotation.clone() * self.vertices[1] * scale + translation,
                rotation.clone() * self.vertices[2] * scale + translation,
            ],
            surface_normal: rotation.clone() * self.surface_normal,
            vertex_normals: [
                rotation.clone() * self.vertex_normals[0],
                rotation.clone() * self.vertex_normals[1],
                rotation.clone() * self.vertex_normals[2],
            ],
        }
    }
}

impl BoundedVolume for Triangle {
    fn intersect(&self, ray: Ray) -> Option<Collision> {
        let a = self.vertices[0];
        let b = self.vertices[1];
        let c = self.vertices[2];
        let n = self.surface_normal;
        let an = self.vertex_normals[0];
        let bn = self.vertex_normals[1];
        let cn = self.vertex_normals[2];

        // d = constant term of triangle plane
        let d = n.dot(a);

        // t = distance along ray of intersection with plane
        let t = (d - n.dot(ray.origin)) / (n.dot(ray.direction));

        if t.is_nan() || t < 0.0 {
            return None;
        }

        // p = intersection with plane
        let p = ray.origin + (ray.direction * t);
        
        // Convert to barycentric coordinates.
        let area_abc = n.dot((b - a).cross(c - a));
        let area_pbc = n.dot((b - p).cross(c - p));
        let area_pca = n.dot((c - p).cross(a - p));

        let bx = area_pbc / area_abc;
        let by = area_pca / area_abc;
        let bz = 1.0 - bx - by;

        let smooth_normal = an * bx + bn * by + cn * bz;
        
        if bx < 0.0 || by < 0.0 || bz < 0.0 {
            None
        } else {
            Some(Collision{ distance: t, location: p, normal: smooth_normal })
        }
    }

    fn aabb(&self) -> AABB {
        // Just the min/max of each coordinate.
        let v1 = self.vertices[0];
        let v2 = self.vertices[1];
        let v3 = self.vertices[2];

        let min_x = v1.x.min(v2.x.min(v3.x));
        let min_y = v1.y.min(v2.y.min(v3.y));
        let min_z = v1.z.min(v2.z.min(v3.z));

        let max_x = v1.x.max(v2.x.max(v3.x));
        let max_y = v1.y.max(v2.y.max(v3.y));
        let max_z = v1.z.max(v2.z.max(v3.z));

        AABB::new(Vector3::new(min_x, min_y, min_z), Vector3::new(max_x, max_y, max_z))
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
