//! Material shader definition.

use std::collections::HashMap;

use crate::{
    color::Color,
    shader::graph::{Graph, Name, SocketValue, Validated},
};

#[derive(Debug, Clone, Default)]
/// A material to be associated with an [Object] for rendering.
pub struct Material {
    selected_outputs: HashMap<StandardMaterialOutput, Name>,
    graph: Graph<Validated>,
    recompute: bool,
}

impl From<(Graph<Validated>, HashMap<StandardMaterialOutput, Name>)> for Material {
    fn from(
        (graph, selected_outputs): (Graph<Validated>, HashMap<StandardMaterialOutput, Name>),
    ) -> Self {
        Material {
            selected_outputs,
            graph,
            recompute: true,
        }
    }
}

impl Material {
    /// Retrieve all standard information about a pixel in the shader graph's result.
    pub fn get(&self, x: u32, y: u32) -> MaterialOutputBundle {
        let get_value = |output: StandardMaterialOutput| {
            self.selected_outputs
                .get(&output)
                .map(|name| self.graph.outputs.get(name))
                .flatten()
                .map(|(_ref, value)| match value {
                    SocketValue::Value(image) => image.as_ref().map(|image| image.mod_get(x, y)),
                    _ => None,
                })
                .flatten()
        };

        MaterialOutputBundle {
            color: self
                .selected_outputs
                .get(&StandardMaterialOutput::Color)
                .map(|name| self.graph.outputs.get(name))
                .flatten()
                .map(|(_ref, value)| match value {
                    SocketValue::Color(image) => image.as_ref().map(|image| image.mod_get(x, y)),
                    _ => None,
                })
                .flatten(),
            diffuse: get_value(StandardMaterialOutput::Diffuse),
            specular: get_value(StandardMaterialOutput::Specular),
            specular_power: get_value(StandardMaterialOutput::SpecularPower),
            reflection: get_value(StandardMaterialOutput::Reflection),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum StandardMaterialOutput {
    Color,
    Diffuse,
    Specular,
    SpecularPower,
    Reflection,
}

#[derive(Debug, Clone)]
pub struct MaterialOutputBundle {
    /// [Color] at point.
    pub color: Option<Color>,
    /// Diffuse value at point (k_d).
    pub diffuse: Option<f32>,
    /// Specular value at point (k_s).
    pub specular: Option<f32>,
    /// Specular power value at point.
    pub specular_power: Option<f32>,
    /// How much light is reflected.
    pub reflection: Option<f32>,
}
