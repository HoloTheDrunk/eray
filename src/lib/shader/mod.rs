pub mod graph;
pub mod parsing;
pub mod shader;

use graph::*;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub struct Signature {
    input: HashMap<Name, SocketType>,
    output: HashMap<Name, SocketType>,
}
