use std::collections::HashMap;

use crate::camera::Camera;
use crate::colour::Colour;
use crate::matrix::Matrix3;
use crate::vector::Vector3;
use crate::geom;
use crate::material::{BasicMaterial, Material, MaterialColour};
use crate::model;
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
#[serde(tag = "type")]
pub enum MaterialColourDescription {
    Rgb { r: f64, g: f64, b: f64 },
    Vertex,
}

impl MaterialColourDescription {
    pub fn to_material_colour(&self) -> MaterialColour {
        match self {
            MaterialColourDescription::Rgb { r, g, b } => MaterialColour::Static(Colour::rgb(*r, *g, *b)),
            MaterialColourDescription::Vertex => MaterialColour::Vertex,
        }
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
    pub lights: Vec<LightDescription>,
    pub skybox: SkyboxDescription,

    #[serde(default)]
    pub models: HashMap<String, ModelDescription>,
}

impl SceneDescription {
    pub fn camera(&self) -> Camera {
        self.camera.to_camera()
    }

    pub fn scene(&self) -> scene::Scene {
        let mut model_library = model::ModelLibrary::new();
        let mut objects: Vec<scene::Object> = Vec::with_capacity(self.objects.len());
        let mut lights: Vec<scene::Light> = Vec::with_capacity(self.lights.len());

        self.models.iter().for_each(|(name, desc)| {
            println!("Declaring model '{}' from '{}'", name, desc.file);
            model_library.declare(name.clone(), desc.file.clone());
        });

        self.objects.iter().for_each(|o| {
            match o.shape {
                ShapeDescription::Sphere(ref shp) => {
                    let obj_ix = objects.len();
                    let geometry = geom::Geometry::Primitive(geom::Primitive::sphere(shp.center.to_vector(), shp.radius));
                    let material: Material = (&o.material).into();

                    objects.push(scene::Object{
                        id: obj_ix,
                        geometry,
                        material,
                    });
                },
                ShapeDescription::Mesh(ref shp) => {
                    println!("Constructing object using model '{}'", shp.model);
                    let translation = shp.translation.to_vector();
                    let rotation = Matrix3::rotation(shp.rotation.pitch, shp.rotation.yaw, shp.rotation.roll);

                    // Ensure model is loaded.
                    let model_indices = model_library.load(&shp.model);

                    model_indices.iter().for_each(|ix| {
                        let obj_ix = objects.len();

                        if shp.smooth_normals {
                            // Ensure vertex normals are pre-calculated if we want smooth normals.
                            model_library
                                .get_mut(*ix)
                                .compute_vertex_normals();
                        }

                        let geometry = geom::Geometry::Mesh(
                            geom::Mesh::new(*ix, translation, rotation, shp.scale, shp.smooth_normals)
                        );

                        let material: Material = match o.material {
                            MaterialDescription::Auto => model_library.get(*ix).material.unwrap_or(
                                Material::lambertian(MaterialColour::Static(Colour::WHITE), Colour::BLACK)
                           ),
                            _ => (&o.material).into(),
                        };

                        objects.push(scene::Object{
                            id: obj_ix,
                            geometry,
                            material,
                        });
                    });
                },
            };

        });

        self.lights.iter().enumerate().for_each(|(ix, l)| {
            lights.push(scene::Light{
                id: ix,
                geometry: l.geometry.to_light_geometry(),
                colour: l.colour.to_colour(),
                intensity: l.intensity,
            });
        });

        scene::Scene::new(model_library, objects, lights, self.skybox.to_skybox())
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
pub struct LightDescription {
    pub geometry: LightGeometryDescription,
    pub colour: ColourDescription,
    pub intensity: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum LightGeometryDescription {
    Point(VectorDescription),
    Sphere(SphereDescription),
}

impl LightGeometryDescription {
    pub fn to_light_geometry(&self) -> scene::LightGeometry {
        match self {
            LightGeometryDescription::Point(v) => scene::LightGeometry::Point(v.to_vector()),
            LightGeometryDescription::Sphere(s) => scene::LightGeometry::Area(
                geom::Primitive::sphere(s.center.to_vector(), s.radius)
            ),
        }
    }
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

    #[serde(default = "default_smooth_normals")]
    pub smooth_normals: bool,
    pub translation: VectorDescription,
    pub rotation: RotationDescription,
    pub scale: f64,
}

fn default_smooth_normals() -> bool {
    true
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MaterialDescription {
    Auto,
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
            MaterialDescription::Auto => panic!("Cannot directly convert Auto material description into material."),
            MaterialDescription::Lambertian(mat) => Material::lambertian(
                mat.albedo.to_material_colour(), Colour::BLACK
            ),
            MaterialDescription::Gloss(mat) => Material::gloss(mat.albedo.to_material_colour(), mat.reflectance, mat.metalness),
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
            BasicMaterialDescription::Lambertian(mat) => Material::lambertian(
                mat.albedo.to_material_colour(), Colour::BLACK
            ).to_basic(),
            BasicMaterialDescription::Gloss(mat) => Material::gloss(mat.albedo.to_material_colour(), mat.reflectance, mat.metalness).to_basic(),
            BasicMaterialDescription::Mirror(_mat) => Material::mirror().to_basic(),
            BasicMaterialDescription::CookTorrance(mat) => Material::cook_torrance(mat.albedo.to_colour(), mat.roughness).to_basic(),
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct LambertianMaterialDescription {
    pub albedo: MaterialColourDescription,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct GlossMaterialDescription {
    pub albedo: MaterialColourDescription,
    pub reflectance: f64,
    pub metalness: f64,
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
