use crate::bvh::{construct_bvh_aac, BVH};
use crate::camera::Camera;
use crate::colour::Colour;
use crate::geom::{AABB, BoundedVolume, Collision, Shape, Ray};
use crate::material::Material;
use crate::vector::Vector3;

#[derive(Clone)]
pub struct Object {
    pub shape: Box<dyn Shape>,
    pub material: Material,
}

impl BoundedVolume for Object {
    fn aabb(&self) -> AABB {
        self.shape.aabb()
    }

    fn intersect(&self, ray: Ray) -> Option<Collision> {
        self.shape.intersect(ray)
    }
}


#[derive(Clone, Copy, Debug)]
pub enum Skybox {
    Flat(FlatSky),
    Gradient(GradientSky),
}

impl Skybox {
    pub fn flat(colour: Colour) -> Skybox {
        Skybox::Flat(FlatSky{ colour })
    }

    pub fn gradient(overhead_colour: Colour, horizon_colour: Colour) -> Skybox {
        Skybox::Gradient(GradientSky{ overhead_colour, horizon_colour })
    }

    pub fn ambient_light(&self, direction: Vector3) -> Colour {
        match self {
            Skybox::Flat(sky) => sky.colour,
            Skybox::Gradient(sky) => {
                let cos_theta = direction.dot(Vector3::new(0.0, 1.0, 0.0));
                sky.overhead_colour * cos_theta + sky.horizon_colour * (1.0 - cos_theta)
            },
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct FlatSky {
    pub colour: Colour,
}

#[derive(Clone, Copy, Debug)]
pub struct GradientSky {
    pub overhead_colour: Colour,
    pub horizon_colour: Colour,
}

pub struct Scene {
    pub camera: Camera,
    pub skybox: Skybox,
    bvh: BVH<Object>,
}

impl Scene {
    pub fn new(camera: Camera, objects: Vec<Object>, skybox: Skybox) -> Scene {
        let bvh = construct_bvh_aac(objects);
        Scene { camera, skybox, bvh }
    }

    pub fn find_intersection(&self, ray: Ray) -> Option<(Collision, Material)> {
        self.bvh.find_intersection(ray).map(|(col, obj)| (col, obj.material))
    }
}
