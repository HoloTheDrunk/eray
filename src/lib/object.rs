//! [Object] and [OpenGLObject] definitions along with auxilliary / helper functions and data
//! structures.

use std::{
    mem::{size_of, size_of_val},
    ops::Range,
    path::Path,
    str::SplitWhitespace,
};

use crate::{
    material::Material,
    primitives::{Triangle, Vertex},
    raycasting::{Ray, RaycastHit},
    vector::Vector,
    Building, Built, GLConsumed,
};

#[derive(Debug)]
/// OpenGL-ready helper struct.
pub struct OpenGLObject {
    /// OpenGL index of this object's vertices VBO.
    pub vertices_vbo: u32,
    /// OpenGL index of this object's normals VBO.
    pub normals_vbo: Option<u32>,
}

#[derive(Debug)]
/// Full object with metadata and optimization info.
pub struct Object<State> {
    /// Current state ([Building]/[Built]).
    pub state: std::marker::PhantomData<State>,

    /// Name tag.
    pub name: Option<String>,

    /// Vertex positions.
    pub vertices: Vec<Vector<3, f32>>,
    /// Normal directions.
    pub normals: Vec<Vector<3, f32>>,
    /// Texture UV positions.
    pub uvs: Vec<Vector<2, f32>>,

    /// All faces are 3-gons (i.e. [Triangle] instances).
    pub faces: Vec<Triangle<3, f32>>,

    /// Min and max coordinates of the object in x, y and z.
    pub bounding_box: BoundingBox,

    /// Object material.
    pub material: Material,
}

impl Object<Built> {
    /// Check if a ray intersects the object and return intersection information.
    ///
    /// Uses the contained [BoundingBox] to ignore objects.
    pub fn intersects(&self, ray: &Ray) -> Option<RaycastHit> {
        if !self.bounding_box.intersects(ray) {
            return None;
        }

        for (index, face) in self.faces.iter().enumerate() {
            if let Some((position, normal, barycentric)) = face.intersects(ray) {
                return Some(RaycastHit {
                    face_index: index,
                    position,
                    normal,
                    material: {
                        let uv = face.a.uv * barycentric[2]
                            + face.b.uv * barycentric[0]
                            + face.c.uv * barycentric[1];

                        self.material.get(uv[0] as u32, uv[1] as u32)
                    },
                });
            }
        }

        None
    }
}

impl Default for Object<Building> {
    fn default() -> Self {
        Self {
            state: std::marker::PhantomData::<Building>,
            name: Some(String::default()),
            vertices: vec![],
            normals: vec![],
            uvs: vec![],
            faces: vec![],
            bounding_box: BoundingBox::default(),
            material: Material::default(),
        }
    }
}

impl Object<Building> {
    fn push_vertex(&mut self, line: usize, tokens: SplitWhitespace) {
        let coords = parse_coords(tokens, Some(line));
        self.vertices.push(coords[0..=2].into());
    }

    fn push_normal(&mut self, line: usize, tokens: SplitWhitespace) {
        let coords = parse_coords(tokens, Some(line));
        self.normals.push(coords[0..=2].into());
    }

    fn push_uv(&mut self, line: usize, tokens: SplitWhitespace) {
        let coords = parse_coords(tokens, Some(line));
        self.uvs.push(coords[0..=1].into());
    }

    fn push_face(&mut self, line: usize, tokens: SplitWhitespace) {
        let vertices = tokens
            .map(|token| {
                let indices = parse_indices(token);
                Vertex {
                    position: self.vertices[indices[0].unwrap() - 1],
                    uv: self.uvs[indices[1].unwrap() - 1],
                    normal: self.normals[indices[2].unwrap() - 1],
                }
            })
            .collect::<Vec<_>>();

        assert_eq!(
            3,
            vertices.len(),
            "Invalid vertex count for face at line {line} (should be 3, is {})",
            vertices.len()
        );

        let mut vertices = vertices.into_iter();

        self.faces.push(Triangle::new(
            vertices.next().unwrap(),
            vertices.next().unwrap(),
            vertices.next().unwrap(),
        ));
    }

    /// Set object name (optional).
    pub fn name(&mut self, name: &str) -> &mut Self {
        self.name = Some(name.to_owned());
        self
    }

    /// Set object vertices (mandatory).
    pub fn vertices(&mut self, vertices: impl Iterator<Item = Vector<3, f32>>) -> &mut Self {
        self.vertices = vertices.collect();

        self.bounding_box = BoundingBox::default();
        self.vertices
            .iter()
            .for_each(|v| self.bounding_box.stretch_to(v));

        self
    }

    /// Set object normals (mandatory).
    pub fn normals(&mut self, normals: impl Iterator<Item = Vector<3, f32>>) -> &mut Self {
        self.normals = normals.collect();
        self
    }

    /// Lock object's fields and allow for OpenGL conversion.
    pub fn build(self) -> Result<Object<Built>, &'static str> {
        if self.vertices.is_empty() {
            Err("Missing vertices")
        } else if self.normals.is_empty() {
            Err("Missing normals")
        } else {
            Ok(Object::<Built> {
                state: std::marker::PhantomData,
                name: self.name,
                vertices: self.vertices,
                normals: self.normals,
                uvs: self.uvs,
                faces: self.faces,
                bounding_box: self.bounding_box,
                material: self.material,
            })
        }
    }
}

impl Object<Built> {
    /// Load an object from a Wavefront .obj file.
    pub fn load_obj(path: &Path) -> std::io::Result<Self> {
        let content = std::fs::read_to_string(path)?;

        // let mut object = Self::default();
        let mut object = Object::<Building>::default();

        for (line, line_content) in content.lines().enumerate() {
            if line_content.is_empty() || line_content.chars().next().unwrap_or('#') == '#' {
                continue;
            }

            let mut tokens = line_content.split_whitespace();
            let marker = tokens.next().unwrap();

            match marker {
                "o" => {
                    let name = tokens.next().unwrap();
                    dbg!("Parsing object `{name}`");
                    object.name(name);
                }
                "g" => {
                    dbg!("Parsing group `{}`", tokens.next().unwrap());
                }
                "s" => {
                    dbg!(
                        "Smooth shading would now be {}",
                        match tokens.next().unwrap() {
                            "1" | "on" => "on",
                            "0" | "off" => "off",
                            v => panic!("Unhandled smooth shading setting `{v}`"),
                        }
                    );
                }
                "v" => object.push_vertex(line, tokens),
                "vn" => object.push_normal(line, tokens),
                "vt" => object.push_uv(line, tokens),
                "f" => object.push_face(line, tokens),
                _ => panic!("Unhandled marker {marker}"),
            }
        }

        Ok(object.build().unwrap())
    }

    /// Convert into an [OpenGLObject] and mark as consumed.
    pub fn to_opengl(self) -> (Object<GLConsumed>, OpenGLObject) {
        let vbos = [&self.vertices, &self.normals]
            .into_iter()
            .enumerate()
            .map(|(index, array)| (index as u32, array, array[0].len() as i32))
            .map(|(index, array, len)| {
                if len == 0 {
                    return None;
                }

                unsafe {
                    gl::VertexAttribPointer(
                        // Index
                        index,
                        // Component count
                        len,
                        // Component type
                        gl::FLOAT,
                        // Normalized?
                        gl::FALSE,
                        // Stride (could also be 0 here)
                        size_of::<Vector<3, f32>>().try_into().unwrap(),
                        // Pointer in VBO
                        0 as *const _,
                    );

                    gl::EnableVertexAttribArray(index);

                    // Generate a Vertex Buffer Object
                    let mut vbo = 0;
                    {
                        gl::GenBuffers(1, &mut vbo);
                        assert_ne!(vbo, 0);

                        // Bind it
                        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);

                        // Pass data to it
                        gl::BufferData(
                            gl::ARRAY_BUFFER,
                            (array.len() * size_of_val(&array[0])) as isize,
                            array.as_ptr().cast(),
                            gl::STATIC_DRAW,
                        );
                    }

                    Some(vbo)
                }
            })
            .collect::<Vec<_>>();

        (
            Object::<GLConsumed> {
                state: std::marker::PhantomData,
                name: self.name,
                vertices: self.vertices,
                normals: self.normals,
                uvs: self.uvs,
                faces: self.faces,
                bounding_box: self.bounding_box,
                material: self.material,
            },
            OpenGLObject {
                vertices_vbo: vbos[0].unwrap(),
                normals_vbo: vbos[1],
            },
        )
    }
}

// TODO: Make N-dimensional..?
#[derive(Debug, Default)]
/// Spatial limits of the object's vertices relative to its origin.
pub struct BoundingBox {
    /// X-axis limits (left -> right).
    pub x: Range<f32>,
    /// Y-axis limits (down -> up).
    pub y: Range<f32>,
    /// Z-axis limits (backwards -> forwards).
    pub z: Range<f32>,
}

impl BoundingBox {
    /// Get the start and end opposite corners of the [BoundingBox].
    pub fn bounds(&self) -> [Vector<3, f32>; 2] {
        [
            Vector::new(self.x.start, self.y.start, self.z.start),
            Vector::new(self.x.end, self.y.end, self.z.end),
        ]
    }

    /// Checks if the [Ray] intersects with the [BoundingBox].
    pub fn intersects(&self, ray: &Ray) -> bool {
        let start = ray.start();

        let invdir = ray.dir().div_under(1.);
        let signs = Vector::<3, f32>::from(
            Into::<[f32; 3]>::into(invdir)
                .iter()
                .map(|&v| (v < 0.) as u32 as f32)
                .collect::<Vec<f32>>()
                .as_slice(),
        );
        let bounds = self.bounds();

        let mut txmin = (bounds[signs[0] as usize][0] - start[0]) * invdir[0];
        let mut txmax = (bounds[1 - signs[0] as usize][0] - start[0]) * invdir[0];

        let tymin = (bounds[signs[1] as usize][1] - start[1]) * invdir[1];
        let tymax = (bounds[1 - signs[1] as usize][1] - start[1]) * invdir[1];

        if (txmin > tymax) || (tymin > txmax) {
            return false;
        }

        if tymin > txmin {
            txmin = tymin;
        }

        if tymax < tymax {
            txmax = tymax;
        }

        let tzmin = (bounds[signs[2] as usize][2] - start[2]) * invdir[2];
        let tzmax = (bounds[1 - signs[2] as usize][2] - start[2]) * invdir[2];

        if tzmin > txmin {
            txmin = tzmin;
        }

        if tzmax < txmax {
            txmax = tzmax;
        }

        let mut t = txmin;

        if t < 0. {
            t = txmax;
            if t < 0. {
                return false;
            }
        }

        return true;
    }

    fn stretch_to(&mut self, pos: &Vector<3, f32>) {
        if pos[0] < self.x.start {
            self.x.start = pos[0];
        } else if pos[0] > self.x.end {
            self.x.end = pos[0];
        }

        if pos[1] < self.y.start {
            self.y.start = pos[1];
        } else if pos[1] > self.y.end {
            self.y.end = pos[1];
        }
    }
}

fn parse_coords(tokens: SplitWhitespace, line: Option<usize>) -> Vec<f32> {
    let coords = tokens
        .map(|token| {
            token
                .parse::<f32>()
                .expect(format!("Failed to parse coords, should be an f32: {token}").as_str())
        })
        .collect::<Vec<_>>();

    if !(2..4).contains(&coords.len()) {
        panic!(
            "Invalid coordinate count at line {}: {coords:?}",
            line.map(|line| line.to_string())
                .unwrap_or("Unknown".to_owned())
        );
    }

    coords
}

fn parse_indices(string: &str) -> Vec<Option<usize>> {
    string
        .split('/')
        .map(|index| index.parse::<usize>().ok())
        .collect()
}
