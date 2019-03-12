use std::collections::HashMap;

use crate::paths::camera::Camera;
use crate::paths::colour::Colour;
use crate::paths::sampling::CorrelatedMultiJitteredSampler;
use crate::paths::vector::Vector3;
use crate::paths::material;
use crate::paths::obj;
use crate::paths::scene;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VectorDescription {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl VectorDescription {
    pub fn to_vector(&self) -> Vector3 {
        Vector3{ x: self.x, y: self.y, z: self.z }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ColourDescription {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}

impl ColourDescription {
    pub fn to_colour(&self) -> Colour {
        Colour{ r: self.r, g: self.g, b: self.b }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SceneDescription {
    pub camera: CameraDescription,
    pub models: HashMap<String, ModelDescription>,
    pub objects: Vec<ObjectDescription>,
    pub skybox: SkyboxDescription,
}

impl SceneDescription {
    pub fn to_scene(&self) -> scene::Scene {
        let mut objects: Vec<scene::Object> = Vec::with_capacity(self.objects.len());
        let mut models: HashMap<String, Vec<Box<scene::Shape>>> = HashMap::with_capacity(self.models.len());

        self.models.iter().for_each(|(name, desc)| {
            let model = obj::load_obj_file(&desc.file);
            let triangles: Vec<Box<scene::Shape>> = model.resolve_triangles().iter()
                .map(|t| Box::new(*t) as Box<scene::Shape>)
                .collect();
            models.insert(name.clone(), triangles);
        });

        self.objects.iter().for_each(|o| {
            let material = o.material.to_material();
            let shapes: Vec<Box<scene::Shape>> = match o.shape {
                ShapeDescription::Sphere(ref shp) => vec![Box::new(scene::Sphere{
                    center: shp.center.to_vector(),
                    radius: shp.radius,
                })],
                ShapeDescription::Mesh(ref shp) => {
                    models.get(&shp.model).unwrap().clone()
                },
            };

            shapes.iter().for_each(|shape| {
                objects.push(scene::Object{ material: material.clone(), shape: shape.clone() });
            });
        });
        scene::Scene::new(self.camera.to_camera(), objects, self.skybox.to_skybox())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CameraDescription {
    pub image_width: u32,
    pub image_height: u32,

    pub location: VectorDescription,
    pub pitch: f64,
    pub yaw: f64,
    pub roll: f64,

    pub sensor_width: f64,
    pub sensor_height: f64,
    pub focal_length: f64,
    pub focus_distance: f64,
    pub aperture: f64,
}

impl CameraDescription {
    pub fn to_camera(&self) -> Camera {
        let mut camera = Camera::new(
            self.image_width,
            self.image_height,
            Box::new(CorrelatedMultiJitteredSampler::new(42, 16, 16)));

        camera.location = self.location.to_vector();
        camera.set_orientation(self.pitch, self.yaw, self.roll);

        camera.sensor_width = self.sensor_width;
        camera.sensor_height = self.sensor_height;
        camera.focal_length = self.focal_length;

        camera.distance_from_lens = (self.focal_length * self.focus_distance) / (self.focus_distance - self.focal_length);
        camera
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModelDescription {
    pub file: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ObjectDescription {
    pub shape: ShapeDescription,
    pub material: MaterialDescription,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ShapeDescription {
    Sphere(SphereDescription),
    Mesh(MeshDescription),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SphereDescription {
    pub center: VectorDescription,
    pub radius: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MeshDescription {
    pub model: String,
    pub center: VectorDescription,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MaterialDescription {
    Lambertian(LambertianMaterialDescription),
    Gloss(GlossMaterialDescription),
    Mirror(MirrorMaterialDescription),
}

impl MaterialDescription {
    pub fn to_material(&self) -> Box<material::Material> {
        match self {
            MaterialDescription::Lambertian(mat) => Box::new(material::Lambertian::new(mat.albedo.to_colour(), Colour::BLACK)),
            MaterialDescription::Gloss(mat) => Box::new(material::Gloss::new(mat.albedo.to_colour(), mat.reflectance)),
            MaterialDescription::Mirror(_) => Box::new(material::Mirror{}),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LambertianMaterialDescription {
    pub albedo: ColourDescription,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GlossMaterialDescription {
    pub albedo: ColourDescription,
    pub reflectance: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MirrorMaterialDescription {}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SkyboxDescription {
    Flat(FlatSkyboxDescription),
    Gradient(GradientSkyboxDescription),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FlatSkyboxDescription {
    pub colour: ColourDescription,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GradientSkyboxDescription {
    pub overhead_colour: ColourDescription,
    pub horizon_colour: ColourDescription,
}

impl SkyboxDescription {
    pub fn to_skybox(&self) -> Box<scene::Skybox> {
        match self {
            SkyboxDescription::Flat(sky) => Box::new(scene::FlatSky{ colour: sky.colour.to_colour() }),
            SkyboxDescription::Gradient(sky) => Box::new(scene::GradientSky{
                overhead_colour: sky.overhead_colour.to_colour(),
                horizon_colour: sky.horizon_colour.to_colour(),
            }),
        }
    }
}
