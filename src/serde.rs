use std::collections::HashMap;

use crate::camera::Camera;
use crate::colour::Colour;
use crate::matrix::Matrix3;
use crate::vector::Vector3;
use crate::geom;
use crate::material::{BasicMaterial, Material};
use crate::obj;
use crate::scene;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
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

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
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

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct RotationDescription {
    pub pitch: f64,
    pub yaw: f64,
    pub roll: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SceneDescription {
    pub camera: CameraDescription,
    pub objects: Vec<ObjectDescription>,
    pub skybox: SkyboxDescription,

    #[serde(default)]
    pub models: HashMap<String, ModelDescription>,
}

impl SceneDescription {
    pub fn camera(&self) -> Camera {
        self.camera.to_camera()
    }

    pub fn scene(&self) -> scene::Scene {
        let mut model_library: scene::ModelLibrary = HashMap::with_capacity(self.models.len());
        let mut objects: Vec<scene::Object> = Vec::with_capacity(self.objects.len());

        self.models.iter().for_each(|(name, desc)| {
            println!("Loading model '{}' from '{}'", name, desc.file);
            let model = obj::load_obj_file(&desc.file);
            let triangles: Vec<geom::Primitive> = model.resolve_triangles().iter()
                .map(|v| *v)
                .collect();
            model_library.insert(name.clone(), triangles);
        });

        self.objects.iter().enumerate().for_each(|(ix, o)| {
            let material: Material = (&o.material).into();
            let geometry: geom::Geometry = match o.shape {
                ShapeDescription::Sphere(ref shp) => geom::Geometry::Primitive(
                    geom::Primitive::sphere(shp.center.to_vector(), shp.radius)
                ),
                ShapeDescription::Mesh(ref shp) => {
                    println!("Constructing object using model '{}'", shp.model);
                    let translation = shp.translation.to_vector();
                    let rotation = Matrix3::rotation(shp.rotation.pitch, shp.rotation.yaw, shp.rotation.roll);
                    geom::Geometry::Mesh(
                        geom::Mesh::new(shp.model.clone(), translation, rotation, shp.scale)
                    )
                },
            };

            objects.push(scene::Object{
                id: ix,
                geometry,
                material,
            });
        });

        scene::Scene::new(model_library, objects, self.skybox.to_skybox())
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct CameraDescription {
    pub image_width: u32,
    pub image_height: u32,

    pub location: VectorDescription,
    pub orientation: RotationDescription,

    pub sensor_width: f64,
    pub sensor_height: f64,
    pub focal_length: f64,
    pub focus_distance: f64,
    pub aperture: f64,
}

impl CameraDescription {
    pub fn to_camera(&self) -> Camera {
        let mut camera = Camera::new(self.image_width, self.image_height);

        camera.location = self.location.to_vector();
        let orientation = Matrix3::rotation(self.orientation.yaw, self.orientation.pitch, self.orientation.roll);
        camera.set_orientation(orientation);

        camera.sensor_width = self.sensor_width;
        camera.sensor_height = self.sensor_height;
        camera.focal_length = self.focal_length;
        camera.aperture = self.aperture;

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

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct SphereDescription {
    pub center: VectorDescription,
    pub radius: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MeshDescription {
    pub model: String,
    pub translation: VectorDescription,
    pub rotation: RotationDescription,
    pub scale: f64,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MaterialDescription {
    Lambertian(LambertianMaterialDescription),
    Gloss(GlossMaterialDescription),
    Mirror(MirrorMaterialDescription),
    CookTorrance(CookTorranceMaterialDescription),
    Fresnel(FresnelMaterialDescription),
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum BasicMaterialDescription {
    Lambertian(LambertianMaterialDescription),
    Gloss(GlossMaterialDescription),
    Mirror(MirrorMaterialDescription),
    CookTorrance(CookTorranceMaterialDescription),
}

impl From<&MaterialDescription> for Material {
    fn from(desc: &MaterialDescription) -> Material {
        match desc {
            MaterialDescription::Lambertian(mat) => Material::lambertian(mat.albedo.to_colour(), Colour::BLACK),
            MaterialDescription::Gloss(mat) => Material::gloss(mat.albedo.to_colour(), mat.reflectance),
            MaterialDescription::Mirror(_mat) => Material::mirror(),
            MaterialDescription::CookTorrance(mat) => Material::cook_torrance(mat.albedo.to_colour(), mat.roughness),
            MaterialDescription::Fresnel(mat) => 
                Material::fresnel_combination(
                    mat.diffuse.into(),
                    mat.specular.into(),
                    mat.refractive_index
                ),
        }
    }
}

impl From<BasicMaterialDescription> for BasicMaterial {
    fn from(desc: BasicMaterialDescription) -> BasicMaterial {
        match desc {
            BasicMaterialDescription::Lambertian(mat) => Material::lambertian(mat.albedo.to_colour(), Colour::BLACK).to_basic(),
            BasicMaterialDescription::Gloss(mat) => Material::gloss(mat.albedo.to_colour(), mat.reflectance).to_basic(),
            BasicMaterialDescription::Mirror(_mat) => Material::mirror().to_basic(),
            BasicMaterialDescription::CookTorrance(mat) => Material::cook_torrance(mat.albedo.to_colour(), mat.roughness).to_basic(),
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct LambertianMaterialDescription {
    pub albedo: ColourDescription,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct GlossMaterialDescription {
    pub albedo: ColourDescription,
    pub reflectance: f64,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct MirrorMaterialDescription {}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct CookTorranceMaterialDescription {
    pub albedo: ColourDescription,
    pub roughness: f64,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct FresnelMaterialDescription {
    pub refractive_index: f64,
    pub diffuse: BasicMaterialDescription,
    pub specular: BasicMaterialDescription,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SkyboxDescription {
    Flat(FlatSkyboxDescription),
    Gradient(GradientSkyboxDescription),
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct FlatSkyboxDescription {
    pub colour: ColourDescription,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct GradientSkyboxDescription {
    pub overhead_colour: ColourDescription,
    pub horizon_colour: ColourDescription,
}

impl SkyboxDescription {
    pub fn to_skybox(&self) -> scene::Skybox {
        match self {
            SkyboxDescription::Flat(sky) => scene::Skybox::flat(sky.colour.to_colour()),
            SkyboxDescription::Gradient(sky) => scene::Skybox::gradient(
                sky.overhead_colour.to_colour(),
                sky.horizon_colour.to_colour(),
            ),
        }
    }
}
