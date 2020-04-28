use std::f64::consts::PI;

use rand;
use rand::Rng;

use crate::matrix::Matrix3;
use crate::vector::Vector3;

#[derive(Clone, Copy, Debug)]
pub struct Ray {
    pub origin: Vector3,
    pub direction: Vector3,
    pub inv_direction: Vector3,
    pub sign: [bool; 3],
}

impl Ray {
    pub fn new(origin: Vector3, direction: Vector3) -> Ray {
        Ray {
            origin,
            direction,
            inv_direction: direction.invert(),
            sign: [direction.x >= 0.0, direction.y >= 0.0, direction.z >= 0.0],
        }
    }

    pub fn random_in_hemisphere(&self) -> Ray {
        let mut rng = rand::thread_rng();
        let yaw = (rng.gen::<f64>() - 0.5) * PI;
        let pitch = (rng.gen::<f64>() - 0.5) * PI;
        let roll = (rng.gen::<f64>() - 0.5) * PI;
        let rot = Matrix3::rotation(yaw, pitch, roll);
        Ray::new(self.origin, rot * self.direction)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Collision {
    pub distance: f64,
    pub location: Vector3,
    pub normal: Vector3,
}

pub struct AABB {
    pub min: Vector3,
    pub max: Vector3,
    pub center: Vector3,
}

impl AABB {
    pub fn new(min: Vector3, max: Vector3) -> AABB {
        let center = (min + max) * 0.5;
        AABB { min, max, center }
    }
}

pub trait BoundedVolume {
    fn aabb(&self) -> AABB;
    fn intersect(&self, ray: Ray) -> Option<Collision>;
}

#[derive(Clone, Copy, Debug)]
pub enum Shape {
    Sphere(SphereShape),
    Triangle(TriangleShape),
}

impl Shape {
    pub fn sphere(center: Vector3, radius: f64) -> Shape {
        Shape::Sphere(SphereShape{ center, radius })
    }

    pub fn triangle(vertices: [Vector3; 3], surface_normal: Vector3, vertex_normals: [Vector3; 3]) -> Shape {
        Shape::Triangle(TriangleShape{ vertices, surface_normal, vertex_normals })
    }

    pub fn transform(&self, translation: Vector3, rotation: Matrix3, scale: f64) -> Shape {
        match self {
            Shape::Sphere(sphere) => Shape::Sphere(sphere.transform(translation, rotation, scale)),
            Shape::Triangle(triangle) => Shape::Triangle(triangle.transform(translation, rotation, scale)),
        }
    }
}

impl BoundedVolume for Shape {
    fn aabb(&self) -> AABB {
        match self {
            Shape::Sphere(sphere) => sphere.aabb(),
            Shape::Triangle(triangle) => triangle.aabb(),
        }
    }

    fn intersect(&self, ray: Ray) -> Option<Collision> {
        match self {
            Shape::Sphere(sphere) => sphere.intersect(ray),
            Shape::Triangle(triangle) => triangle.intersect(ray),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SphereShape {
    pub center: Vector3,
    pub radius: f64,
}

impl SphereShape {
    pub fn transform(&self, translation: Vector3, _: Matrix3, scale: f64) -> SphereShape {
        SphereShape {
            center: self.center + translation,
            radius: self.radius * scale,
        }
    }
}

impl BoundedVolume for SphereShape {
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
pub struct TriangleShape {
    pub vertices: [Vector3; 3],
    pub surface_normal: Vector3,
    pub vertex_normals: [Vector3; 3],
}

impl TriangleShape {
    pub fn transform(&self, translation: Vector3, rotation: Matrix3, scale: f64) -> TriangleShape {
        TriangleShape {
            vertices: [
                rotation * self.vertices[0] * scale + translation,
                rotation * self.vertices[1] * scale + translation,
                rotation * self.vertices[2] * scale + translation,
            ],
            surface_normal: rotation.clone() * self.surface_normal,
            vertex_normals: [
                rotation * self.vertex_normals[0],
                rotation * self.vertex_normals[1],
                rotation * self.vertex_normals[2],
            ],
        }
    }
}

impl BoundedVolume for TriangleShape {
    fn intersect(&self, ray: Ray) -> Option<Collision> {
        let a = self.vertices[0];
        let b = self.vertices[1];
        let c = self.vertices[2];
        let n = self.surface_normal;
        let an = self.vertex_normals[0];
        let bn = self.vertex_normals[1];
        let cn = self.vertex_normals[2];

        let cos_theta = n.dot(ray.direction);

        // d = constant term of triangle plane
        let d = n.dot(a);

        // t = distance along ray of intersection with plane
        let t = (d - n.dot(ray.origin)) / cos_theta;

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

        let mut smooth_normal = an * bx + bn * by + cn * bz;

        // If the smoothed face of the triangle curves away from the ray then scale it back so it
        // barely doesn't.
        if smooth_normal.dot(ray.direction) * cos_theta < 0.0 {
            let epsilon = 0.05;  // Chosen experimentally.
            let cos_alpha = smooth_normal.dot(ray.direction);
            let scale = (cos_alpha - epsilon) / (cos_theta + cos_alpha);
            smooth_normal = (n * scale + smooth_normal * (1.0 - scale)).normed();
        }
        
        if bx < 0.0 || by < 0.0 || bz < 0.0 {
            None
        } else {
            // Flip the normal if we're hitting the triangle from the back;
            let back_side_multiplier = if cos_theta > 0.0 { -1.0 } else { 1.0 };
            Some(Collision{ distance: t, location: p, normal: smooth_normal * back_side_multiplier })
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
