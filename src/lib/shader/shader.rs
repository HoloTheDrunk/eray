use super::{Signature, Type};

use crate::{color::Color, image::Image, vector::Vec3};

use std::{cell::RefCell, collections::HashMap, fmt::Debug, rc::Rc};

#[derive(Debug)]
pub struct Graph<'names> {
    inputs: HashMap<&'names str, GraphInput>,
    outputs: HashMap<&'names str, InSocket>,

    root: Rc<RefCell<Node>>,
}

impl Graph<'_> {
    pub fn build(&mut self) {
        self.root.borrow_mut().build();
    }

    pub fn parse(code: &str) {
        todo!()
    }
}

pub struct Node {
    name: String,

    inputs: Vec<InSocket>,
    outputs: Vec<Box<dyn OutSocket>>,

    shader: Box<dyn Fn(&HashMap<&str, &SocketValue>, &mut HashMap<String, &mut SocketValue>)>,
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
        shader: Box<dyn Fn(&HashMap<&str, &SocketValue>, &mut HashMap<String, &mut SocketValue>)>,
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

        res.borrow_mut()
            .outputs
            .extend(outputs.map(|(name, value)| -> Box<dyn OutSocket> {
                Box::new(NodeOutSocket {
                    node: res.clone(),
                    name,
                    value,
                })
            }));

        res
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn signature(&self) -> Signature {
        Signature {
            input: self
                .inputs
                .iter()
                .map(|in_socket| (in_socket.name.clone(), in_socket.value.r#type()))
                .collect(),
            output: self
                .outputs
                .iter()
                .map(|out_socket| (out_socket.name().to_owned(), out_socket.value().r#type()))
                .collect(),
        }
    }

    pub fn build(&mut self) {
        self.inputs
            .iter_mut()
            .filter(|socket| socket.value.is_none())
            .for_each(|socket| {
                if let Some(mut out_socket) =
                    socket.prev.as_ref().map(|refcell| refcell.borrow_mut())
                {
                    out_socket.compute_value();
                } else {
                    socket.value.set_default();
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
                .map(|v| (v.name().to_string(), v.value_mut()))
                .collect(),
        );
    }
}

#[derive(Debug)]
pub struct InSocket {
    pub prev: Option<RefCell<Box<dyn OutSocket>>>,

    pub name: String,
    pub value: SocketValue,
}

impl InSocket {
    pub fn new(name: String, value: SocketValue) -> Self {
        Self {
            prev: None,
            name,
            value,
        }
    }
}

pub trait OutSocket: Debug {
    fn compute_value(&mut self) -> &SocketValue;
    fn name(&self) -> &str;
    fn value(&self) -> &SocketValue;
    fn value_mut(&mut self) -> &mut SocketValue;
}

pub struct NodeOutSocket {
    node: Rc<RefCell<Node>>,

    name: String,
    value: SocketValue,
}

impl Debug for NodeOutSocket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OutSocket")
            .field("name", &self.name)
            .field("value", &self.value)
            .finish_non_exhaustive()
    }
}

impl OutSocket for NodeOutSocket {
    fn compute_value(&mut self) -> &SocketValue {
        if self.value.is_none() {
            self.node.borrow_mut().build();
        };

        &self.value
    }

    fn name(&self) -> &str {
        self.name.as_str()
    }

    fn value(&self) -> &SocketValue {
        &self.value
    }

    fn value_mut(&mut self) -> &mut SocketValue {
        &mut self.value
    }
}

#[derive(Debug)]
pub struct GraphInput {
    name: String,
    value: SocketValue,
}

impl GraphInput {
    pub fn new(name: String, value: SocketValue) -> Self {
        GraphInput { name, value }
    }
}

impl OutSocket for GraphInput {
    fn compute_value(&mut self) -> &SocketValue {
        if self.value.is_none() {
            self.value.set_default();
        }

        &self.value
    }

    fn name(&self) -> &str {
        self.name.as_str()
    }

    fn value(&self) -> &SocketValue {
        &self.value
    }

    fn value_mut(&mut self) -> &mut SocketValue {
        &mut self.value
    }
}

#[derive(Clone, Debug)]
pub enum SocketValue {
    Value(Option<Image<f32>>),
    Color(Option<Image<Color>>),
    Vec3(Option<Image<Vec3>>),
}

impl From<Type> for SocketValue {
    fn from(r#type: Type) -> Self {
        match r#type {
            Type::Value => SocketValue::Value(None),
            Type::Vec3 => SocketValue::Vec3(None),
            Type::Color => SocketValue::Color(None),
        }
    }
}

impl SocketValue {
    pub fn is_none(&self) -> bool {
        match self {
            SocketValue::Value(opt) => opt.is_none(),
            SocketValue::Color(opt) => opt.is_none(),
            SocketValue::Vec3(opt) => opt.is_none(),
        }
    }

    pub(super) fn r#type(&self) -> Type {
        match self {
            SocketValue::Value(_) => Type::Value,
            SocketValue::Color(_) => Type::Color,
            SocketValue::Vec3(_) => Type::Vec3,
        }
    }

    pub fn set_default(&mut self) {
        match self {
            SocketValue::Value(ref mut value) => *value = Some(Image::init(1, 1, 0.)),
            SocketValue::Color(ref mut color) => *color = Some(Image::init(1, 1, Color::default())),
            SocketValue::Vec3(ref mut vec3) => *vec3 = Some(Image::init(1, 1, Vec3::default())),
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
            inputs: Default::default(),
            outputs: Default::default(),
            root: Node::new(
                "Test Shader",
                std::iter::empty(),
                std::iter::once(("Fac".to_owned(), SocketValue::Value(None))),
                Box::new(|_inputs, outputs| outputs.get_mut("Fac").unwrap().set_f32(50, 50, 1.)),
            ),
        };

        let prev = shader.root.borrow().outputs[0].value().clone();

        shader.root.borrow_mut().build();

        assert_ne!(prev, *shader.root.borrow().outputs[0].value());
    }
}
