//! Sine wave.
//!
//! Mandatory inputs:
//! - width: Value, width of the output image
//! - height: Value, height of the output image
//!
//! Optional inputs:
//! - x_fac: Value, multiplier for x direction, default is 1.
//! - y_fac: Value, multiplier for y direction, default is 1.
//!
//! Output:
//! - value: Value

use crate::handle_missing_socket_values;

use super::{GraphResult, MaterialResult, NodeResult};

use eray::{
    get_sv, node,
    prelude::*,
    shader::{
        self,
        graph::{Graph, ImportedNode, SocketType, SocketValue},
        shader::Side,
    },
    ssref,
};

use map_macro::hash_map;

pub const DEFAULT_FACTOR: f32 = 1.;

pub fn material() -> MaterialResult {
    Ok(Material::from((
        shader::graph::graph! {
            inputs:
                // Mandatory
                "width": SocketType::Value.into(),
                "height": SocketType::Value.into(),

                // Optional
                "x_fac": SocketValue::Value(Some(DEFAULT_FACTOR)),
                "y_fac": SocketValue::Value(Some(DEFAULT_FACTOR)),
            nodes:
                "inner": {
                    let map = hash_map!{
                        String::from("inner") => ImportedNode::from(("inner", graph()?))
                    };

                    let mut node = node!(import "inner" from map);
                    node.set_input(&"width".into(), ssref!(graph "width"))?
                        .set_input(&"height".into(), ssref!(graph "height"))?
                        .set_input(&"x_fac".into(), ssref!(graph "x_fac"))?
                        .set_input(&"y_fac".into(), ssref!(graph "y_fac"))?;
                    node
                },
                "viewer": {
                    let mut node = node!(import graph "inner" super::rgb::graph()?);
                    node.set_input(&"width".into(), ssref!(graph "width"))?
                        .set_input(&"height".into(), ssref!(graph "height"))?
                        .set_input(&"red".into(), ssref!(node "inner" "value"))?
                        .set_input(&"green".into(), ssref!(node "inner" "value"))?
                        .set_input(&"blue".into(), ssref!(node "inner" "value"))?;
                    node
                },
            outputs:
                "color": (ssref!(node "viewer" "color"), SocketType::IColor.into()),
        }
        .validate()?,
        hash_map! {
            StandardMaterialOutput::Color => "color".into(),
        },
    )))
}

pub fn graph() -> GraphResult {
    Ok(shader::graph::graph! {
        inputs:
            // Mandatory
            "width": SocketType::Value.into(),
            "height": SocketType::Value.into(),

            // Optional
            "x_fac": SocketValue::Value(Some(DEFAULT_FACTOR)),
            "y_fac": SocketValue::Value(Some(DEFAULT_FACTOR)),
        nodes:
            "wave": {
                let mut node = node()?;
                node.set_input(&"width".into(), ssref!(graph "width"))?
                    .set_input(&"height".into(), ssref!(graph "height"))?
                    .set_input(&"x_fac".into(), ssref!(graph "x_fac"))?
                    .set_input(&"y_fac".into(), ssref!(graph "y_fac"))?;
                node
            },
        outputs:
            "value": (ssref!(node "wave" "value"), SocketType::Value.into()),
    })
}

pub fn node() -> NodeResult {
    Ok(node! {
        inputs:
            "width": (None, SocketType::Value),
            "height": (None, SocketType::Value),

            "x_fac": (None, SocketType::Value),
            "y_fac": (None, SocketType::Value),
        outputs:
            "value": SocketType::IValue.into();
        |inputs, outputs| {
            get_sv!( input | inputs  . "width": Value > width);
            get_sv!( input | inputs  . "height": Value > height);

            get_sv!( input | inputs  . "x_fac": Value > x_fac);
            get_sv!( input | inputs  . "y_fac": Value > y_fac);

            get_sv!(output | outputs . "value": IValue > out);

            handle_missing_socket_values![width, height];
            let x_fac = x_fac.unwrap_or(DEFAULT_FACTOR);
            let y_fac = y_fac.unwrap_or(DEFAULT_FACTOR);

            let mut res = Image::new(*width as u32, *height as u32, 0.);

            for y in 0..(res.height) {
                for x in 0..(res.width) {
                    let value = ((x as f32 * x_fac + y as f32 * y_fac) / 10.).cos().abs();
                    res.pixels[(y * res.width + x) as usize] = value;
                }
            }

            out.replace(res);

            Ok(())
        }
    })
}
