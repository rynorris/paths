use std::fs::File;
use std::io::{Read};
use std::str::FromStr;

use nom::{digit, double};
use nom::types::CompleteStr;

use crate::model::Model;
use crate::vector::Vector3;

pub fn load_obj_file(filename: &str) -> Model {
    let f = File::open(filename).unwrap();
    parse_obj(f)
}

// This is a MASSIVE over-simplification of the .obj file format and won't work
// for any but the simplest possible files.
pub fn parse_obj<R>(mut reader: R) -> Model where R : Read {
    let mut contents = Vec::new();
    reader.read_to_end(&mut contents).unwrap();

    let contents_str = std::str::from_utf8(contents.as_slice()).unwrap();
    let input = CompleteStr(contents_str);
    object(input).unwrap().1
}

named!(newline(CompleteStr) -> (),
    do_parse!(opt!(char!('\r')) >> char!('\n') >> ())
);

named!(index(CompleteStr) -> usize, map_res!(recognize!(digit), |c: CompleteStr| usize::from_str(*c)));

named!(float(CompleteStr) -> f64, call!(double));

named!(vertex(CompleteStr) -> Vector3,
    do_parse!(
           char!('v') >>
           char!(' ') >>
        x: float      >>
           char!(' ') >>
        y: float      >>
           char!(' ') >>
        z: float      >>
        (Vector3::new(x, y, z))
    )
);

named!(face(CompleteStr) -> (usize, usize, usize),
    do_parse!(
           char!('f') >>
           char!(' ') >>
        a: index      >>
           char!(' ') >>
        b: index      >>
           char!(' ') >>
        c: index      >>
        (a - 1, b - 1, c - 1)  // These are 1-indexed in the file.
    )
);

named!(vertices(CompleteStr) -> Vec<Vector3>, many1!(terminated!(vertex, opt!(newline))));

named!(faces(CompleteStr) -> Vec<(usize, usize, usize)>, many1!(terminated!(face, opt!(newline))));

named!(object(CompleteStr) -> Model,
    do_parse!(
        vertices: vertices            >>
                  many0!(char!('\n')) >>
        faces:    faces               >>
        (Model::new(vertices, faces))
    )
);

#[cfg(test)]
mod test {
    use std::fs::File;
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn test_vertex() {
        assert_eq!(
            vertex(CompleteStr("v 0.1 1.5 27")),
            Ok((CompleteStr(""), Vector3::new(0.1, 1.5, 27.0))));
    }

    #[test]
    fn test_face() {
        assert_eq!(
            face(CompleteStr("f 43 1 562")),
            Ok((CompleteStr(""), (42, 0, 561))));
    }

    #[test]
    fn test_vertices() {
        assert_eq!(
            vertices(CompleteStr("v 1 2 3\nv 4 5 6\nv 7 8 9\n")),
            Ok((CompleteStr(""), vec![
               Vector3::new(1.0, 2.0, 3.0),
               Vector3::new(4.0, 5.0, 6.0),
               Vector3::new(7.0, 8.0, 9.0),
            ])));
    }

    #[test]
    fn test_faces() {
        assert_eq!(
            faces(CompleteStr("f 1 2 3\nf 4 5 6\nf 7 8 9\n")),
            Ok((CompleteStr(""), vec![
               (0, 1, 2),
               (3, 4, 5),
               (6, 7, 8),
            ])));
    }

    #[test]
    fn test_teapot() {
        let teapot = parse_obj_file("scenes/objects/teapot.obj");
        assert_eq!(teapot.vertices.len(), 3644);
        assert_eq!(teapot.vertices[0], Vector3::new(-3.0, 1.8, 0.0));
        assert_eq!(teapot.vertices[3643], Vector3::new(3.434, 2.4729, 0.0));

        assert_eq!(teapot.faces.len(), 6320);
        assert_eq!(teapot.faces[0], (2908, 2920, 2938));
        assert_eq!(teapot.faces[6319], (3000, 3003, 3021));
    }

    fn parse_obj_file(name: &str) -> Model {
        let mut f = File::open(test_resource_path(name)).unwrap();
        let mut contents = Vec::new();
        f.read_to_end(&mut contents).unwrap();

        let contents_str = std::str::from_utf8(contents.as_slice()).unwrap();
        let input = CompleteStr(contents_str);
        let res = object(input).unwrap();
        assert_eq!(res.0, CompleteStr(""));
        res.1
    }

    fn test_resource_path(name: &str) -> PathBuf {
        let mut buf = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        buf.push(name);
        buf
    }
}
