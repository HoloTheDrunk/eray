#![allow(unused)]

mod utils;

pub mod flat_color;
pub mod mix_color;
pub mod rgb;
pub mod wave;

use eray::{
    prelude::Material,
    shader::graph::{Error, Graph, ImportedNode, Node, Unvalidated},
};

type GraphResult = Result<Graph<Unvalidated>, Error>;
type MaterialResult = Result<Material, Error>;
type NodeResult = Result<Node<Unvalidated>, Error>;

macro_rules! create_elib {
    ($($lib:ident),+ $(,)?) => {
        pub fn elib() -> Vec<ImportedNode<Unvalidated>> {
            vec![
                $(
                    ImportedNode::from((stringify!($lib), $lib::graph().unwrap()))
                ),+
            ]
        }
    };
}

create_elib! {
    // Generators
    flat_color,
    wave,

    // Converters
    rgb,

    // Mixers
    mix_color,
}

lazy_static::lazy_static! {
    pub static ref SHADERLIB: Vec<ImportedNode<Unvalidated>> = elib();
}
