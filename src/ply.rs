use std::fs::File;

use ply_rs::parser;
use ply_rs::ply;
use ply_rs::ply::PropertyAccess;

use crate::model::Model;
use crate::vector::Vector3;

pub fn load_ply_file(filename: &str) -> Model {
    let mut f = File::open(filename).unwrap();
    let p = parser::Parser::<ply::DefaultElement>::new();
    let ply = p.read_ply(&mut f).unwrap();

    println!("Read PLY file with header: {:?}", ply.header);

    // Ignoring any nuances of the file format for now.
    // Just assume the format we expect.
    let vertex = &ply.header.elements["vertex"];
    let face = &ply.header.elements["face"];
    let ply_vertices = &ply.payload["vertex"];
    let ply_faces = &ply.payload["face"];

    let mut min_x = f64::MAX;
    let mut min_y = f64::MAX;
    let mut min_z = f64::MAX;
    let mut max_x = f64::MIN;
    let mut max_y = f64::MIN;
    let mut max_z = f64::MIN;

    let vertices: Vec<Vector3> = ply_vertices.iter().map(|v| {
        let v = Vector3{
            x: v.get_float(&vertex.properties["x"].name).unwrap() as f64,
            y: v.get_float(&vertex.properties["y"].name).unwrap() as f64,
            z: v.get_float(&vertex.properties["z"].name).unwrap() as f64,
        };

        min_x = f64::min(min_x, v.x);
        min_y = f64::min(min_y, v.y);
        min_z = f64::min(min_z, v.z);
        max_x = f64::max(max_x, v.x);
        max_y = f64::max(max_y, v.y);
        max_z = f64::max(max_z, v.z);

        v
    }).collect();

    let faces: Vec<(usize, usize, usize)> = ply_faces.iter().map(|f| {
        let vertex_indices = f.get_list_int(&face.properties["vertex_indices"].name).unwrap();
        (vertex_indices[0] as usize, vertex_indices[1] as usize, vertex_indices[2] as usize)
    }).collect();

    println!("Loaded {} vertices and {} faces.", vertices.len(), faces.len());
    println!("Model bounds: X: {}-{}, Y: {}-{}, Z: {}-{}", min_x, max_x, min_y, max_y, min_z, max_z);

    Model::new(vertices, faces)
}
