mod utils;

pub mod mix_color;
pub mod rgb;
pub mod wave;

use eray::{
    prelude::Material,
    shader::graph::{Error, Graph, Node, Unvalidated},
};

type GraphResult = Result<Graph<Unvalidated>, Error>;
type MaterialResult = Result<Material, Error>;
type NodeResult = Result<Node<Unvalidated>, Error>;
