use std::collections::HashMap;

use crate::geom::Primitive;
use crate::obj;
use crate::ply;
use crate::vector::Vector3;

pub struct ModelLibrary {
    declarations: HashMap<String, ModelDeclaration>,
    models: HashMap<String, Model>,
}

pub struct ModelDeclaration {
    filepath: String,
}

impl ModelLibrary {
    pub fn new() -> ModelLibrary {
        ModelLibrary {
            declarations: HashMap::new(),
            models: HashMap::new(),
        }
    }

    pub fn declare(&mut self, name: String, filepath: String) {
        self.declarations.insert(name, ModelDeclaration{ filepath });
    }

    pub fn load(&mut self, name: &String) {
        if self.models.contains_key(name) {
            println!("Model '{}' already loaded.", name);
            return;
        }

        let filepath = match self.declarations.get(name) {
            Some(decl) => &decl.filepath,
            None => panic!("Attempt to load model '{}' before declaration", name),
        };

        println!("Loading model '{}' from '{}'", name, filepath);
        let path = std::path::Path::new(&filepath);
        let extension = path.extension().map(|osstr| osstr.to_str()).flatten();
        let model = match extension {
            Some("obj") => {
                obj::load_obj_file(&filepath)
            },
            Some("ply") => {
                ply::load_ply_file(&filepath)
            },
            Some(ext) => panic!("Unknown file extension: {}", ext),
            None => panic!("Could not identify filetype for path because it has no extension: {:?}", path),
        };

        self.models.insert(name.clone(), model);
    }

    pub fn get(&self, name: &String) -> &Model {
        match self.models.get(name) {
            Some(m) => m,
            None => panic!("Model '{}' has not been loaded", name),
        }
    }
}

pub struct Model {
    vertices: Vec<Vector3>,
    faces: Vec<(usize, usize, usize)>,
    face_normals: Vec<Vector3>,
    vertex_normals: Option<Vec<Vector3>>,
}

impl Model {
    pub fn new(vertices: Vec<Vector3>, faces: Vec<(usize, usize, usize)>) -> Model {
        let face_normals = Model::compute_face_normals(&vertices, &faces);

        Model { vertices, faces, face_normals, vertex_normals: None }
    }

    pub fn resolve_primitives(&self) -> Vec<Primitive> {
        self.faces.iter()
            .enumerate()
            .filter_map(|(ix, &(a, b, c))| {
                let v1 = self.vertices[a];
                let v2 = self.vertices[b];
                let v3 = self.vertices[c];

                let vertices = [v1, v2, v3];
                let surface_normal = self.face_normals[ix];

                let vertex_normals = match self.vertex_normals {
                    Some(ref vertex_normals) => {
                        let vn1 = vertex_normals[a];
                        let vn2 = vertex_normals[b];
                        let vn3 = vertex_normals[c];
                        [vn1, vn2, vn3]
                    },
                    None => [Vector3::zero(), Vector3::zero(), Vector3::zero()],
                };

                if surface_normal.is_nan() {
                    None
                } else {
                    Some(Primitive::triangle(vertices, surface_normal, vertex_normals))
                }
            })
            .collect()
    }

    pub fn compute_vertex_normals(&mut self) {
        let mut vertex_normal_sums: Vec<Vector3> = vec![Vector3::new(0.0, 0.0, 0.0); self.vertices.len()];
        let mut vertex_normal_counts: Vec<usize> = vec![0; self.vertices.len()];

        self.faces.iter()
            .enumerate()
            .for_each(|(ix, &(a, b, c))| {
                let n = self.face_normals[ix];
                if n.is_nan() {
                    return;
                }

                vertex_normal_sums[a] += n;
                vertex_normal_sums[b] += n;
                vertex_normal_sums[c] += n;
                vertex_normal_counts[a] += 1;
                vertex_normal_counts[b] += 1;
                vertex_normal_counts[c] += 1;
            });

        self.vertex_normals = Some(
            vertex_normal_sums.iter()
            .enumerate()
            .map(|(ix, &v)| v / (vertex_normal_counts[ix]) as f64)
            .collect()
        );
    }

    fn compute_face_normals(vertices: &Vec<Vector3>, faces: &Vec<(usize, usize, usize)>) -> Vec<Vector3> {
        faces.iter()
            .map(|&(a, b, c)| {
                let v1 = vertices[a];
                let v2 = vertices[b];
                let v3 = vertices[c];
                Model::face_normal(v1, v2, v3)
            })
            .collect()
    }

    fn face_normal(v1: Vector3, v2: Vector3, v3: Vector3) -> Vector3 {
        let side_1 = v2 - v1;
        let side_2 = v3 - v1;
        let side_3 = v3 - v2;
        let mut n = side_1.cross(side_2).normed();

        if n.x.is_nan() || n.y.is_nan() || n.z.is_nan() {
            // Try again with a different pair of sides.
            n = side_1.cross(side_3).normed();
        }

        n
    }
}
