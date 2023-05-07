use super::{sockets::*, Signature, Type};

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
    pub fn new_wrapped(
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

    // pub fn duplicate(&self) -> Rc<RefCell<Self>> {
    //     let shader = &self.to_owned().shader;
    //     let res = Rc::new(RefCell::new(Self {
    //         outputs: vec![],
    //         shader: shader.clone(),
    //         ..*self.clone()
    //     }));
    //
    //     // res.borrow_mut().outputs.extend(outputs.map)
    //     todo!()
    // }
    //
    fn add_outputs(
        wrapped: &mut Rc<RefCell<Node>>,
        outputs: impl Iterator<Item = Box<dyn OutSocket>>,
    ) {
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_valid_shader_compilation() {
        let shader = Graph {
            inputs: Default::default(),
            outputs: Default::default(),
            root: Node::new_wrapped(
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
