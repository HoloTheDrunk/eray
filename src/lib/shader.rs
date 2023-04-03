use std::{cell::RefCell, collections::HashMap, fmt::Debug, rc::Rc};

use crate::{color::Color, image::Image, vector::Vec3};

#[derive(Debug)]
pub struct Graph {
    root: Rc<RefCell<Node>>,
}

impl Graph {
    pub fn build(&mut self) {
        self.root.borrow_mut().build();
    }

    pub fn parse(code: &str) {
        todo!()
    }
}

struct Node {
    name: String,

    inputs: Vec<InSocket>,
    outputs: Vec<OutSocket>,

    shader: Box<dyn Fn(&HashMap<&str, &SocketValue>, &mut HashMap<&str, &mut SocketValue>)>,
}

impl Debug for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Node")
            .field("inputs", &self.inputs)
            .field("outputs", &self.outputs)
            .finish_non_exhaustive()
    }
}

impl Node {
    pub fn new(
        name: &str,
        inputs: impl Iterator<Item = (String, SocketValue)>,
        outputs: impl Iterator<Item = (String, SocketValue)>,
        shader: Box<dyn Fn(&HashMap<&str, &SocketValue>, &mut HashMap<&str, &mut SocketValue>)>,
    ) -> Rc<RefCell<Self>> {
        let res = Rc::new(RefCell::new(Self {
            name: name.to_owned(),
            inputs: inputs
                .map(|(name, value)| InSocket {
                    prev: None,
                    name,
                    value,
                })
                .collect(),
            outputs: vec![],
            shader,
        }));

        res.borrow_mut().outputs.extend(
            outputs
                .map(|(name, value)| OutSocket {
                    node: res.clone(),
                    name,
                    value,
                })
                .collect::<Vec<_>>(),
        );

        res
    }

    pub fn build(&mut self) {
        self.inputs
            .iter_mut()
            .filter(|socket| socket.value.is_none())
            .for_each(|socket| {
                if let Some(out_socket) = socket.prev.as_ref().map(|refcell| refcell.borrow_mut()) {
                    out_socket.node.borrow_mut().build();
                } else {
                    socket.value = socket.value.get_default();
                }
            });

        (self.shader)(
            &self
                .inputs
                .iter()
                .map(|v| (v.name.as_str(), &v.value))
                .collect(),
            &mut self
                .outputs
                .iter_mut()
                .map(|v| (v.name.as_str(), &mut v.value))
                .collect(),
        );
    }
}

#[derive(Debug)]
struct InSocket {
    prev: Option<RefCell<OutSocket>>,

    name: String,
    value: SocketValue,
}

struct OutSocket {
    node: Rc<RefCell<Node>>,

    name: String,
    value: SocketValue,
}

impl Debug for OutSocket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OutSocket")
            .field("name", &self.name)
            .field("value", &self.value)
            .finish_non_exhaustive()
    }
}

impl OutSocket {
    pub fn compute_value(&mut self) -> &SocketValue {
        if self.value.is_none() {
            self.node.borrow_mut().build();
        };

        &self.value
    }
}

#[derive(Clone, Debug)]
enum SocketValue {
    Value(Option<Image<f32>>),
    Color(Option<Image<Color>>),
    Vec3(Option<Image<Vec3>>),
}

impl SocketValue {
    pub fn is_none(&self) -> bool {
        match self {
            SocketValue::Value(opt) => opt.is_none(),
            SocketValue::Color(opt) => opt.is_none(),
            SocketValue::Vec3(opt) => opt.is_none(),
        }
    }

    pub fn get_default(&self) -> Self {
        match self {
            SocketValue::Value(_) => SocketValue::Value(Some(Image::init(1, 1, 0.))),
            SocketValue::Color(_) => SocketValue::Color(Some(Image::init(1, 1, Color::default()))),
            SocketValue::Vec3(_) => SocketValue::Vec3(Some(Image::init(1, 1, Vec3::default()))),
        }
    }

    pub fn set_f32(&mut self, width: u32, height: u32, value: f32) {
        match self {
            SocketValue::Value(ref mut opt) => *opt = Some(Image::init(width, height, value)),
            SocketValue::Color(ref mut opt) => {
                *opt = Some(Image::init(width, height, Color::new(value, value, value)))
            }
            SocketValue::Vec3(ref mut opt) => {
                *opt = Some(Image::init(width, height, Vec3::new(value, value, value)))
            }
        }
    }

    pub fn set_color(&mut self, width: u32, height: u32, value: Color) {
        match self {
            SocketValue::Value(ref mut opt) => {
                *opt = Some(Image::init(
                    width,
                    height,
                    (value.r + value.g + value.b) / 3.,
                ))
            }
            SocketValue::Color(ref mut opt) => *opt = Some(Image::init(width, height, value)),
            SocketValue::Vec3(ref mut opt) => {
                *opt = Some(Image::init(
                    width,
                    height,
                    Vec3::new(value.r, value.g, value.b),
                ))
            }
        }
    }

    pub fn set_vec3(&mut self, width: u32, height: u32, value: Vec3) {
        match self {
            SocketValue::Value(ref mut opt) => {
                *opt = Some(Image::init(
                    width,
                    height,
                    (value.x + value.y + value.z) / 3.,
                ))
            }
            SocketValue::Color(ref mut opt) => {
                *opt = Some(Image::init(
                    width,
                    height,
                    Color::new(value.x, value.y, value.z),
                ))
            }
            SocketValue::Vec3(ref mut opt) => *opt = Some(Image::init(width, height, value)),
        }
    }
}

impl PartialEq for SocketValue {
    fn eq(&self, other: &Self) -> bool {
        macro_rules! compare {
            ($($variant:tt),+ $(,)?) => {
                match self {
                $(
                    SocketValue::$variant(lhs) => match other {
                        SocketValue::$variant(rhs) => match (lhs, rhs) {
                            (None, None) => true,
                            (Some(lhs), Some(rhs)) => lhs == rhs,
                            _ => false,
                        },
                        _ => false,
                    },
                )+
                }
            };
        }

        compare!(Value, Color, Vec3)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_valid_shader_compilation() {
        let shader = Graph {
            root: Node::new(
                "Test Shader",
                std::iter::empty(),
                std::iter::once(("Fac".to_owned(), SocketValue::Value(None))),
                Box::new(|_inputs, outputs| outputs.get_mut("Fac").unwrap().set_f32(50, 50, 1.)),
            ),
        };

        let prev = shader.root.borrow().outputs[0].value.clone();

        shader.root.borrow_mut().build();

        assert_ne!(prev, shader.root.borrow().outputs[0].value);
    }
}
