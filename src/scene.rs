use std::collections::HashMap;
use rand;
use rand::Rng;

use crate::bvh::{construct_bvh_aac, BVH};
use crate::colour::Colour;
use crate::geom::{Collision, Geometry, Primitive, Ray};
use crate::material::Material;
use crate::vector::Vector3;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntityID {
    Object(usize),
    Light(usize),
}

#[derive(Clone)]
pub struct Object {
    pub id: usize,
    pub geometry: Geometry,
    pub material: Material,
}

#[derive(Clone, Copy, Debug)]
pub struct Light {
    pub id: usize,
    pub point: Vector3,
    pub colour: Colour,
    pub intensity: f64,
}

pub type Model = Vec<Primitive>;

pub type ModelLibrary = HashMap<String, Model>;

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
    pub skybox: Skybox,
    objects: Vec<Object>,
    lights: Vec<Light>,
    bvh: BVH<EntityID>,
}

impl Scene {
    pub fn new(model_library: ModelLibrary, objects: Vec<Object>, lights: Vec<Light>, skybox: Skybox) -> Scene {
        let primitive_geometry = objects.iter()
            .map(|o| {
                let id = o.id;
                let mut primitives = match o.geometry {
                    Geometry::Primitive(p) => vec![p],
                    Geometry::Mesh(ref m) => m.primitives(&model_library),
                };
                primitives.drain(..).map(move|p| (p, EntityID::Object(id))).collect()
            })
            .flat_map(|items: Vec<(Primitive, EntityID)>| { items.into_iter() })
            .collect();
        let bvh = construct_bvh_aac(primitive_geometry);
        Scene { skybox, objects, lights, bvh }
    }

    pub fn find_intersection(&self, ray: Ray) -> Option<(Collision, Material)> {
        self.bvh.find_intersection(ray).map(|(col, entity)| {
            match entity {
                EntityID::Object(id) => Some((col, self.objects[*id].material)),
                _ => None,
            }
        }).flatten()
    }

    pub fn random_light(&self) -> Option<&Light> {
        if self.lights.len() > 0 {
            let id = rand::thread_rng().gen_range(0, self.lights.len());
            Some(&self.lights[id])
        } else {
            None
        }
    }
}
