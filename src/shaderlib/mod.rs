pub mod rgb;
pub mod wave;

use eray::{
    prelude::Material,
    shader::graph::{Error, Graph, Node, Unvalidated},
};

pub(self) type GraphResult = Result<Graph<Unvalidated>, Error>;
pub(self) type MaterialResult = Result<Material, Error>;
pub(self) type NodeResult = Result<Node<Unvalidated>, Error>;
