use std::fs::File;

use ply_rs::parser;
use ply_rs::ply;
use ply_rs::ply::PropertyAccess;

use crate::geom::Primitive;
use crate::vector::Vector3;

pub struct Model {
    vertices: Vec<Vector3>,
    faces: Vec<(usize, usize, usize)>,
}

impl Model {
    pub fn resolve_triangles(&self) -> Vec<Primitive> {
        // Firstly, compute the face normals.
        let face_normals: Vec<Vector3> = self.faces.iter()
            .map(|&(a, b, c)| {
                let v1 = self.vertices[a];
                let v2 = self.vertices[b];
                let v3 = self.vertices[c];
                Model::face_normal(v1, v2, v3)
            })
            .collect();

        let mut vertex_normal_sums: Vec<Vector3> = vec![Vector3::new(0.0, 0.0, 0.0); self.vertices.len()];
        let mut vertex_normal_counts: Vec<usize> = vec![0; self.vertices.len()];

        self.faces.iter()
            .enumerate()
            .for_each(|(ix, &(a, b, c))| {
                let n = face_normals[ix];
                vertex_normal_sums[a] += n;
                vertex_normal_sums[b] += n;
                vertex_normal_sums[c] += n;
                vertex_normal_counts[a] += 1;
                vertex_normal_counts[b] += 1;
                vertex_normal_counts[c] += 1;
            });

        let vertex_normals: Vec<Vector3> = vertex_normal_sums.iter()
            .enumerate()
            .map(|(ix, &v)| v / (vertex_normal_counts[ix]) as f64)
            .collect();

        self.faces.iter()
            .enumerate()
            .map(|(ix, &(a, b, c))| {
                let v1 = self.vertices[a];
                let v2 = self.vertices[b];
                let v3 = self.vertices[c];

                let vn1 = vertex_normals[a];
                let vn2 = vertex_normals[b];
                let vn3 = vertex_normals[c];

                let vertices = [v1, v2, v3];
                let surface_normal = face_normals[ix];
                let vertex_normals = [vn1, vn2, vn3];

                Primitive::triangle(vertices, surface_normal, vertex_normals)
            })
            .collect()
    }

    fn face_normal(v1: Vector3, v2: Vector3, v3: Vector3) -> Vector3 {
        let side_1 = v2 - v1;
        let side_2 = v3 - v1;
        return side_1.cross(side_2).normed();
    }
}

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

    let vertices: Vec<Vector3> = ply_vertices.iter().map(|v| {
        Vector3{
            x: v.get_float(&vertex.properties["x"].name).unwrap() as f64,
            y: v.get_float(&vertex.properties["y"].name).unwrap() as f64,
            z: v.get_float(&vertex.properties["z"].name).unwrap() as f64,
        }
    }).collect();

    let faces: Vec<(usize, usize, usize)> = ply_faces.iter().map(|f| {
        let vertex_indices = f.get_list_int(&face.properties["vertex_indices"].name).unwrap();
        (vertex_indices[0] as usize, vertex_indices[1] as usize, vertex_indices[2] as usize)
    }).collect();

    println!("Loaded {} vertices and {} faces.", vertices.len(), faces.len());

    Model{ vertices, faces }
}
