pub mod graph;
pub mod parsing;
pub mod shader;
pub mod sockets;

use sockets::{GraphInput, InSocket, SocketValue};

use std::{collections::HashMap, str::FromStr};

#[derive(Debug)]
struct GraphSignature {
    input: HashMap<String, GraphInput>,
    output: HashMap<String, InSocket>,
}

impl From<Signature> for GraphSignature {
    fn from(signature: Signature) -> Self {
        GraphSignature {
            input: signature
                .input
                .into_iter()
                .map(|(name, r#type)| {
                    (
                        name.clone(),
                        GraphInput::new(name, SocketValue::from(r#type)),
                    )
                })
                .collect(),
            output: signature
                .output
                .into_iter()
                .map(|(name, r#type)| {
                    (name.clone(), InSocket::new(name, SocketValue::from(r#type)))
                })
                .collect(),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Signature {
    input: HashMap<String, Type>,
    output: HashMap<String, Type>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Type {
    Value,
    Vec3,
    Color,
}

impl FromStr for Type {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "Value" => Type::Value,
            "Color" => Type::Color,
            "Vec3" => Type::Vec3,
            other => Err(format!("Unrecognized type `{other}`."))?,
        })
    }
}
