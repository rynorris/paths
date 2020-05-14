use rand;
use rand::Rng;

use crate::bvh::{construct_bvh_aac, BVH};
use crate::colour::Colour;
use crate::geom::{Collision, CollisionMetadata, Geometry, Primitive, Ray};
use crate::material::Material;
use crate::model::ModelLibrary;
use crate::vector::Vector3;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntityID {
    Object(usize),
    Light(usize),
}

#[derive(Clone)]
pub enum Entity {
    Object(Object),
    Light(Light),
}

impl Entity {
    pub fn id(self) -> EntityID {
        match self {
            Entity::Object(o) => EntityID::Object(o.id),
            Entity::Light(l) => EntityID::Light(l.id),
        }
    }
}

#[derive(Clone)]
pub struct Object {
    pub id: usize,
    pub geometry: Geometry,
    pub material: Material,
}

#[derive(Clone, Debug)]
pub struct Light {
    pub id: usize,
    pub geometry: LightGeometry,
    pub colour: Colour,
    pub intensity: f64,
}

impl Light {
    pub fn entity_id(&self) -> EntityID {
        EntityID::Light(self.id)
    }

    pub fn sample(&self, from: Vector3) -> (Vector3, f64) {
        match self.geometry {
            LightGeometry::Point(v) => (v, 1.0),
            LightGeometry::Area(p) => p.sample(from),
        }
    }
}

#[derive(Clone, Debug)]
pub enum LightGeometry {
    Point(Vector3),
    Area(Primitive),
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
    pub skybox: Skybox,
    pub models: ModelLibrary,
    objects: Vec<Object>,
    lights: Vec<Light>,
    bvh: BVH<EntityID>,
}

impl Scene {
    pub fn new(mut models: ModelLibrary, objects: Vec<Object>, lights: Vec<Light>, skybox: Skybox) -> Scene {
        let object_primitives = objects.iter()
            .map(|o| {
                let id = o.id;
                let primitives = match o.geometry {
                    Geometry::Primitive(p) => vec![p],
                    Geometry::Mesh(ref m) => m.primitives(&mut models),
                };
                primitives.into_iter().map(move|p| (p, EntityID::Object(id))).collect()
            })
            .flat_map(|items: Vec<(Primitive, EntityID)>| { items.into_iter() });

        let light_primitives = lights.iter()
            .map(|l| {
                let id = l.id;
                let primitives = match l.geometry {
                    LightGeometry::Point(_) => vec![],
                    LightGeometry::Area(primitive) => std::iter::once(primitive).collect(),
                };
                primitives.into_iter().map(move|p| (p, EntityID::Light(id))).collect()
            })
            .flat_map(|items: Vec<(Primitive, EntityID)>| { items.into_iter() });

        let primitive_geometry = object_primitives.chain(light_primitives).collect();

        let bvh = construct_bvh_aac(primitive_geometry);
        Scene { skybox, models, objects, lights, bvh }
    }

    pub fn find_intersection(&self, ray: Ray) -> Option<(Collision, Entity)> {
        self.bvh.find_intersection(ray).map(|(mut col, entity)| {
            match entity {
                EntityID::Object(id) => {
                    let obj = &self.objects[*id];
                    match &obj.geometry {
                        Geometry::Mesh(mesh) => {
                            match col.metadata {
                                CollisionMetadata::Mesh(face_ix, bx, by, bz) => {
                                    if mesh.smooth_normals {
                                        let model = self.models.get(&mesh.model);
                                        let smooth_normal = model.smooth_normal(face_ix, bx, by, bz);
                                        col.normal = mesh.rotate(smooth_normal);
                                    }
                                    Some((col, Entity::Object(obj.clone())))
                                },
                                CollisionMetadata::None => panic!("Mesh collision should include metadata"),
                            }
                        },
                        _ => Some((col, Entity::Object(self.objects[*id].clone()))),
                    }
                },
                EntityID::Light(id) => Some((col, Entity::Light(self.lights[*id].clone()))),
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
