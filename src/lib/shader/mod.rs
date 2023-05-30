//! Shader graph implementation

pub mod graph;
pub mod parsing;
pub mod shader;

use graph::*;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
/// Type signature of a [Graph] or [Node]
pub struct Signature {
    input: HashMap<Name, SocketType>,
    output: HashMap<Name, SocketType>,
}
