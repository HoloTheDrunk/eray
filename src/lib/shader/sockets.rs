use super::{shader::Node, Type};

use crate::{color::Color, image::Image, vector::Vec3};

use std::fmt::Debug;
use std::{cell::RefCell, rc::Rc};

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
    pub node: Rc<RefCell<Node>>,

    pub name: String,
    pub value: SocketValue,
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
