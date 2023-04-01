use std::{
    mem::{size_of, size_of_val},
    ops::Range,
    path::Path,
    str::SplitWhitespace,
};

use crate::{
    primitives::{Triangle, Vertex},
    raycasting::{Ray, RaycastHit},
};

use super::vector::Vec3;

#[derive(Debug)]
pub struct OpenGLObject {
    pub vertices_vbo: u32,
    pub normals_vbo: u32,
}

macro_rules! states {
    ($($state:tt),+ $(,)?) => {
        $(
            #[derive(Clone, Debug, Default)]
            pub struct $state;
        )+
    };
}

states!(Building, Built);

#[derive(Debug, Default)]
pub struct Object<State> {
    pub state: std::marker::PhantomData<State>,

    pub name: Option<String>,

    pub vertices: Vec<Vec3>,
    pub normals: Vec<Vec3>,

    pub faces: Vec<Triangle>,

    pub bounding_box: BoundingBox,
}

impl Object<Built> {
    fn intersects(&self, ray: &Ray) -> Option<RaycastHit> {
        for face in self.faces.iter() {
            if let Some((pos, normal)) = face.intersects(ray) {
                // TODO
                todo!()
            }
        }

        todo!()
    }
}

impl Object<Building> {
    pub fn new() -> Self {
        Self {
            state: std::marker::PhantomData::<Building>,
            ..Default::default()
        }
    }

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

        self.faces.push(Triangle {
            a: vertices.next().unwrap(),
            b: vertices.next().unwrap(),
            c: vertices.next().unwrap(),
        });
    }

    /// Set object name (optional)
    pub fn name(&mut self, name: &str) -> &mut Self {
        self.name = Some(name.to_owned());
        self
    }

    /// Set object vertices (mandatory)
    pub fn vertices(&mut self, vertices: impl Iterator<Item = Vec3>) -> &mut Self {
        self.vertices = vertices.collect();

        self.bounding_box = BoundingBox::default();
        self.vertices
            .iter()
            .for_each(|v| self.bounding_box.stretch_to(v));

        self
    }

    /// Set object normals (mandatory)
    pub fn normals(&mut self, normals: impl Iterator<Item = Vec3>) -> &mut Self {
        self.normals = normals.collect();
        self
    }

    /// Lock object's fields and allow for OpenGL conversion
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
                faces: self.faces,
                bounding_box: self.bounding_box,
            })
        }
    }
}

impl Object<Built> {
    /// Load an object from a Wavefront .obj file
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

    pub fn to_opengl(&self) {
        for (index, array, len) in [&self.vertices, &self.normals]
            .into_iter()
            .enumerate()
            .map(|(index, array)| (index as u32, array, array[0].len() as i32))
        {
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
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct BoundingBox {
    pub x: Range<f32>,
    pub y: Range<f32>,
}

impl BoundingBox {
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
