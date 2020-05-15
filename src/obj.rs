use tobj;

use crate::model::Model;
use crate::vector::Vector3;

pub fn load_obj_file(filename: &str) -> Model {
    let (models, _materials) = tobj::load_obj(filename, true).expect("Failed to load obj file");
    if models.len() > 1 {
        panic!("Obj file '{}' contains more than 1 model.  This is unsupported.", filename);
    }

    let model = &models[0];

    convert_model(model)
}

pub fn convert_model(obj_model: &tobj::Model) -> Model {
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

    let mut model = Model::new(vertices, faces);

    let texcoords = &obj_model.mesh.texcoords;
    if texcoords.len() > 0 {
        let texture_coords = texcoords
            .chunks_exact(2)
            .map(|coords| {
                (coords[0] as f64, coords[1] as f64)
            }).collect();

        model.attach_texture_coords(texture_coords);
    }

    model
}
