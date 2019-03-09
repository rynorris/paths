use crate::paths::colour::Colour;
use crate::paths::vector::Vector3;
use crate::paths::material;
use crate::paths::scene;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VectorDescription {
    x: f64,
    y: f64,
    z: f64,
}

impl VectorDescription {
    pub fn to_vector(&self) -> Vector3 {
        Vector3{ x: self.x, y: self.y, z: self.z }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ColourDescription {
    r: f64,
    g: f64,
    b: f64,
}

impl ColourDescription {
    pub fn to_colour(&self) -> Colour {
        Colour{ r: self.r, g: self.g, b: self.b }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SceneDescription {
    pub objects: Vec<ObjectDescription>,
    pub skybox: SkyboxDescription,
}

impl SceneDescription {
    pub fn to_scene(&self) -> scene::Scene {
        let mut objects: Vec<scene::Object> = Vec::with_capacity(self.objects.len());
        self.objects.iter().for_each(|o| objects.push(o.to_object()));
        scene::Scene {
            objects,
            skybox: self.skybox.to_skybox(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ObjectDescription {
    pub shape: ShapeDescription,
    pub material: MaterialDescription,
}

impl ObjectDescription {
    pub fn to_object(&self) -> scene::Object {
        scene::Object {
            material: self.material.to_material(),
            shape: self.shape.to_shape(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ShapeDescription {
    Sphere(SphereDescription),
}

impl ShapeDescription {
    pub fn to_shape(&self) -> Box<scene::Shape> {
        match self {
            ShapeDescription::Sphere(shp) => Box::new(scene::Sphere{ center: shp.center.to_vector(), radius: shp.radius }),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SphereDescription {
    pub center: VectorDescription,
    pub radius: f64,
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
