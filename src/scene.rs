use crate::bvh::{construct_bvh_aac, BVH};
use crate::camera::Camera;
use crate::colour::Colour;
use crate::geom::{AABB, BoundedVolume, Collision, Shape, Ray};
use crate::material::Material;
use crate::vector::Vector3;

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


pub trait Skybox : SkyboxClone + Send + Sync {
    fn ambient_light(&self, direction: Vector3) -> Colour;
}

pub trait SkyboxClone {
    fn clone_box(&self) -> Box<dyn Skybox>;
}

impl <T> SkyboxClone for T where T: 'static + Skybox + Clone {
    fn clone_box(&self) -> Box<dyn Skybox> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Skybox> {
    fn clone(&self) -> Box<dyn Skybox> {
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
    pub skybox: Box<dyn Skybox>,
    bvh: BVH<Object>,
}

impl Scene {
    pub fn new(camera: Camera, objects: Vec<Object>, skybox: Box<dyn Skybox>) -> Scene {
        let bvh = construct_bvh_aac(objects);
        Scene { camera, skybox, bvh }
    }

    pub fn find_intersection(&self, ray: Ray) -> Option<(Collision, Box<dyn Material>)> {
        self.bvh.find_intersection(ray).map(|(col, obj)| (col, obj.material.clone()))
    }
}
