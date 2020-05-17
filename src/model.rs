use std::collections::HashMap;

use crate::colour::Colour;
use crate::geom::Primitive;
use crate::material::Material;
use crate::obj;
use crate::ply;
use crate::vector::Vector3;

pub struct ModelLibrary {
    declarations: HashMap<String, ModelDeclaration>,
    models: Vec<Model>,
}

#[derive(Clone, Debug)]
pub struct ModelDeclaration {
    filepath: String,
    is_loaded: bool,
    loaded_models: Vec<usize>,
}

impl ModelDeclaration {
    pub fn new(filepath: String) -> ModelDeclaration {
        ModelDeclaration {
            filepath,
            is_loaded: false,
            loaded_models: Vec::new(),
        }
    }

    pub fn mark_loaded(&mut self, models: Vec<usize>) {
        self.loaded_models = models;
        self.is_loaded = true;
    }
}

impl ModelLibrary {
    pub fn new() -> ModelLibrary {
        ModelLibrary {
            declarations: HashMap::new(),
            models: Vec::new(),
        }
    }

    pub fn declare(&mut self, name: String, filepath: String) {
        self.declarations.insert(name, ModelDeclaration::new(filepath));
    }

    pub fn load(&mut self, name: &String) -> Vec<usize> {
        let mut decl = match self.declarations.get(name) {
            Some(decl) => decl.clone(),
            None => panic!("Attempt to load model '{}' before declaration", name),
        };

        if decl.is_loaded {
            println!("Model '{}' already loaded.", name);
            return decl.loaded_models.clone();
        }

        let filepath = &decl.filepath;

        println!("Loading model '{}' from '{}'", name, filepath);
        let path = std::path::Path::new(&filepath);
        let extension = path.extension().map(|osstr| osstr.to_str()).flatten();

        let base_ix = self.models.len();
        let model_indices = match extension {
            Some("obj") => {
                let model_indices = obj::load_obj_file(&filepath)
                    .drain(..)
                    .enumerate()
                    .map(|(ix, m)| {
                        self.models.push(m);
                        base_ix + ix
                    })
                    .collect();

                model_indices
            },
            Some("ply") => {
                let model = ply::load_ply_file(&filepath);
                let ix = self.models.len();
                self.models.push(model);
                vec![ix]
            },
            Some(ext) => panic!("Unknown file extension: {}", ext),
            None => panic!("Could not identify filetype for path because it has no extension: {:?}", path),
        };

        decl.mark_loaded(model_indices.clone());
        self.declarations.insert(name.clone(), decl);

        model_indices
    }

    pub fn get(&self, ix: usize) -> &Model {
        &self.models[ix]
    }

    pub fn get_mut(&mut self, ix: usize) -> &mut Model {
        &mut self.models[ix]
    }
}

pub struct Model {
    pub vertices: Vec<Vector3>,
    pub faces: Vec<(usize, usize, usize)>,
    pub face_normals: Vec<Vector3>,
    pub vertex_normals: Option<Vec<Vector3>>,
    pub vertex_colours: Option<Vec<Colour>>,
    pub texture_coords: Option<Vec<(f64, f64)>>,
    pub material: Option<Material>,
}

impl Model {
    pub fn new(vertices: Vec<Vector3>, faces: Vec<(usize, usize, usize)>) -> Model {
        let face_normals = Model::compute_face_normals(&vertices, &faces);

        Model{ 
            vertices,
            faces,
            face_normals,
            vertex_normals: None,
            vertex_colours: None,
            texture_coords: None,
            material: None,
        }
    }

    pub fn attach_vertex_colours(&mut self, vertex_colours: Vec<Colour>) {
        self.vertex_colours = Some(vertex_colours);
    }

    pub fn attach_texture_coords(&mut self, texture_coords: Vec<(f64, f64)>) {
        self.texture_coords = Some(texture_coords);
    }

    pub fn attach_material(&mut self, material: Material) {
        self.material = Some(material);
    }

    pub fn smooth_normal(&self, face_ix: usize, bx: f64, by: f64, bz: f64) -> Vector3 {
        match self.vertex_normals {
            Some(ref vertex_normals) => {
                let (a, b, c) = self.faces[face_ix];
                let an = vertex_normals[a];
                let bn = vertex_normals[b];
                let cn = vertex_normals[c];

                let smooth_normal = an * bx + bn * by + cn * bz;

                smooth_normal
            },
            None => panic!("Vertex normals not pre-computed"),
        }
    }

    pub fn smooth_colour(&self, face_ix: usize, bx: f64, by: f64, bz: f64) -> Colour {
        match self.vertex_colours {
            Some(ref vertex_colours) => {
                let (a, b, c) = self.faces[face_ix];
                let ac = vertex_colours[a];
                let bc = vertex_colours[b];
                let cc = vertex_colours[c];

                let smooth_colour = ac * bx + bc * by + cc * bz;

                smooth_colour
            },
            None => panic!("Model does not have vertex colours"),
        }
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

                if surface_normal.is_nan() {
                    None
                } else {
                    Some(Primitive::triangle(ix, vertices, surface_normal))
                }
            })
            .collect()
    }

    pub fn compute_vertex_normals(&mut self) {
        if self.vertex_normals.is_some() {
            return;
        }

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
