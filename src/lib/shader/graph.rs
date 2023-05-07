use super::{sockets::*, Signature, Type};

use crate::{color::Color, image::Image, vector::Vec3};

use std::{cell::RefCell, collections::HashMap, fmt::Debug, rc::Rc};

pub struct Graph {
    inputs: HashMap<String, GraphInput>,
    outputs: HashMap<String, InSocket>,
}
