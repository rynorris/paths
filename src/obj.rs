use tobj;

use crate::colour::Colour;
use crate::material::{Material, MaterialColour};
use crate::model::Model;
use crate::vector::Vector3;

pub fn load_obj_file(filename: &str) -> Vec<Model> {
    let (obj_models, obj_materials) = tobj::load_obj(filename, true).expect("Failed to load obj file");

    let materials: Vec<Material> = obj_materials.iter()
        .map(|m| convert_material(m))
        .collect();

    println!("Loaded {} materials", materials.len());

    let models: Vec<Model> = obj_models.iter().map(|m| convert_model(m, &materials)).collect();

    println!("Loaded {} models", models.len());

    models
}

fn convert_material(obj_material: &tobj::Material) -> Material {
    // TODO: Flesh out.
    Material::lambertian(MaterialColour::Static(array_to_colour(obj_material.diffuse)), Colour::BLACK)
}

fn convert_model(obj_model: &tobj::Model, materials: &Vec<Material>) -> Model {
    let vertices: Vec<Vector3> = obj_model.mesh.positions
        .chunks_exact(3)
        .map(|coords| {
            Vector3::new(coords[0] as f64, coords[1] as f64, coords[2] as f64)
        }).collect();

    let faces: Vec<(usize, usize, usize)> = obj_model.mesh.indices
        .chunks_exact(3)
        .map(|indices| {
            (indices[0] as usize, indices[1] as usize, indices[2] as usize)
        }).collect();

    println!("Loaded model with {} vertices and {} faces", vertices.len(), faces.len());

    let mut model = Model::new(vertices, faces);

    let texcoords = &obj_model.mesh.texcoords;
    if texcoords.len() > 0 {
        println!("Model has texture coordinates");
        let texture_coords = texcoords
            .chunks_exact(2)
            .map(|coords| {
                (coords[0] as f64, coords[1] as f64)
            }).collect();

        model.attach_texture_coords(texture_coords);
    }

    match obj_model.mesh.material_id {
        Some(mat) => {
            println!("Model has associated material");
            model.attach_material(materials[mat]);
        },
        None => (),
    }

    model
}

fn array_to_colour(rgb: [f32; 3]) -> Colour {
    Colour::rgb(rgb[0] as f64, rgb[1] as f64, rgb[2] as f64)
}
