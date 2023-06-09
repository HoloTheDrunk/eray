//! Material shader definition.

use std::collections::HashMap;

use crate::{
    color::Color,
    shader::{
        graph::{Error, Graph, Name, SocketValue, Validated},
        shader::Side,
    },
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
    /// Recomputes the inner graph if needed.
    pub fn update(&mut self) -> Result<(), Error> {
        if self.recompute {
            self.graph.run()?;
            self.recompute = false;
        }

        #[cfg(debug_assertions)]
        self.selected_outputs
            .get(&StandardMaterialOutput::Color)
            .and_then(|name| self.graph.outputs.get(name))
            .map(|(_ref, value)| match value {
                SocketValue::IColor(image) => image
                    .as_ref()
                    .map(|image| image.save_as_ppm(std::path::Path::new("color.ppm"))),
                _ => panic!(),
            });

        Ok(())
    }

    /// Retrieve all standard information about a pixel in the shader graph's result.
    pub fn get(&self, x: f32, y: f32) -> MaterialOutputBundle {
        let get_value = |output: StandardMaterialOutput| {
            self.selected_outputs
                .get(&output)
                .and_then(|name| self.graph.outputs.get(name))
                .and_then(|(_ref, value)| match value {
                    SocketValue::IValue(image) => image.as_ref().map(|image| {
                        image.mod_get(
                            (x * image.width as f32) as u32,
                            (y * image.height as f32) as u32,
                        )
                    }),
                    _ => None,
                })
        };

        MaterialOutputBundle {
            color: self
                .selected_outputs
                .get(&StandardMaterialOutput::Color)
                .and_then(|name| {
                    let res = self.graph.outputs.get(name);
                    res
                })
                .and_then(|(_ref, value)| match value {
                    SocketValue::IColor(image) => image.as_ref().map(|image| {
                        image.mod_get(
                            (x * image.width as f32) as u32,
                            (y * image.height as f32) as u32,
                        )
                    }),
                    _ => None,
                }),
            diffuse: get_value(StandardMaterialOutput::Diffuse),
            specular: get_value(StandardMaterialOutput::Specular),
            specular_power: get_value(StandardMaterialOutput::SpecularPower),
            reflection: get_value(StandardMaterialOutput::Reflection),
        }
    }

    /// Set the value of a graph input.
    pub fn set_input(&mut self, name: &Name, value: SocketValue) -> Result<&mut Self, Error> {
        self.graph
            .inputs
            .get_mut(name)
            .ok_or_else(|| Error::Missing(Side::Input, name.clone()))
            .map(|old| *old = value)
            .map(|_| self)
    }
}

#[allow(missing_docs)]
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
/// Standardized shade graph output types.
pub enum StandardMaterialOutput {
    Color,
    Diffuse,
    Specular,
    SpecularPower,
    Reflection,
}

#[derive(Debug, Clone)]
/// Standardized shader graph outputs at a given position.
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
