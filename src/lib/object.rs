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
    vector::Vec3,
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
    pub vertices: Vec<Vec3>,
    /// Normal directions.
    pub normals: Vec<Vec3>,
    /// Texture UV positions.
    pub uvs: Vec<Vec3>,

    /// All faces are 3-gons (i.e. [Triangle] instances).
    pub faces: Vec<Triangle>,

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
                        let uv = face.a.uv * barycentric.z
                            + face.b.uv * barycentric.x
                            + face.c.uv * barycentric.y;

                        self.material.get(uv.x as u32, uv.y as u32)
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
        self.vertices.push(coords[0..2].into());
    }

    fn push_normal(&mut self, line: usize, tokens: SplitWhitespace) {
        let coords = parse_coords(tokens, Some(line));
        self.normals.push(coords[0..2].into());
    }

    fn push_face(&mut self, line: usize, tokens: SplitWhitespace) {
        let vertices = tokens
            .map(|token| {
                let indices = parse_indices(token);
                Vertex {
                    position: self.vertices[indices[0].unwrap() - 1],
                    normal: self.normals[indices[1].unwrap() - 1],
                    uv: self.uvs[indices[2].unwrap() - 1],
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
    pub fn vertices(&mut self, vertices: impl Iterator<Item = Vec3>) -> &mut Self {
        self.vertices = vertices.collect();

        self.bounding_box = BoundingBox::default();
        self.vertices
            .iter()
            .for_each(|v| self.bounding_box.stretch_to(v));

        self
    }

    /// Set object normals (mandatory).
    pub fn normals(&mut self, normals: impl Iterator<Item = Vec3>) -> &mut Self {
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
                    println!("Parsing object `{name}`");
                    object.name(name);
                }
                "g" => println!("Parsing group `{}`", tokens.next().unwrap()),
                "s" => println!(
                    "Smooth shading would now be {}",
                    match tokens.next().unwrap() {
                        "1" | "on" => "on",
                        "0" | "off" => "off",
                        v => panic!("Unhandled smooth shading setting `{v}`"),
                    }
                ),
                "v" => object.push_vertex(line, tokens),
                "vn" => object.push_normal(line, tokens),
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
                        size_of::<Vec3>().try_into().unwrap(),
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
    pub fn bounds(&self) -> [Vec3; 2] {
        [
            Vec3::new(self.x.start, self.y.start, self.z.start),
            Vec3::new(self.x.end, self.y.end, self.z.end),
        ]
    }

    /// Checks if the [Ray] intersects with the [BoundingBox].
    pub fn intersects(&self, ray: &Ray) -> bool {
        let start = ray.start();

        let invdir = 1. / *ray.dir();
        let signs = Vec3::from(
            Into::<[f32; 3]>::into(invdir)
                .iter()
                .map(|&v| (v < 0.) as u32 as f32)
                .collect::<Vec<f32>>()
                .as_slice(),
        );
        let bounds = self.bounds();

        let mut txmin = (bounds[signs.x as usize].x - start.x) * invdir.x;
        let mut txmax = (bounds[1 - signs.x as usize].x - start.x) * invdir.x;

        let tymin = (bounds[signs.y as usize].y - start.y) * invdir.y;
        let tymax = (bounds[1 - signs.y as usize].y - start.y) * invdir.y;

        if (txmin > tymax) || (tymin > txmax) {
            return false;
        }

        if tymin > txmin {
            txmin = tymin;
        }

        if tymax < tymax {
            txmax = tymax;
        }

        let tzmin = (bounds[signs.z as usize].z - start.z) * invdir.z;
        let tzmax = (bounds[1 - signs.z as usize].z - start.z) * invdir.z;

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

    fn stretch_to(&mut self, pos: &Vec3) {
        if pos.x < self.x.start {
            self.x.start = pos.x;
        } else if pos.x > self.x.end {
            self.x.end = pos.x;
        }

        if pos.y < self.y.start {
            self.y.start = pos.y;
        } else if pos.y > self.y.end {
            self.y.end = pos.y;
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

    if !(3..4).contains(&coords.len()) {
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
