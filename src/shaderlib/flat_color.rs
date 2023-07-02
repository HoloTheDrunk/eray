//! Mapper from three [Value](SocketValue::Value) values to a [IColor] image.
//!
//! Mandatory inputs:
//! - red: Value
//! - green: Value
//! - blue: Value
//!
//! Output:
//! - width: Value, width of the output image, defaults to 1
//! - height: Value, height of the output image, defaults to 1
//! - color: IColor

use crate::handle_missing_socket_values;

use super::{GraphResult, MaterialResult, NodeResult};

use eray::{
    get_sv, node,
    prelude::*,
    shader::{
        self,
        graph::{Graph, SocketType, SocketValue},
        shader::Side,
    },
    ssref,
};

use map_macro::hash_map;

pub fn material() -> MaterialResult {
    Ok(Material::from((
        graph()?.validate()?,
        hash_map! {
            StandardMaterialOutput::Color => "color".into()
        },
    )))
}

pub fn graph() -> GraphResult {
    Ok(shader::graph::graph! {
        inputs:
            // Mandatory
            "red": SocketType::Value.into(),
            "green": SocketType::Value.into(),
            "blue": SocketType::Value.into(),

            // Optional
            "width": SocketValue::Value(Some(1.)),
            "height": SocketValue::Value(Some(1.)),
        nodes:
            "converter": {
                let mut node = node()?;
                node.set_input(&"width".into(), ssref!(graph "width"))?
                    .set_input(&"height".into(), ssref!(graph "height"))?
                    .set_input(&"red".into(), ssref!(graph "red"))?
                    .set_input(&"green".into(), ssref!(graph "green"))?
                    .set_input(&"blue".into(), ssref!(graph "blue"))?;
                node
            },
        outputs:
            "color": (ssref!(node "converter" "color"), SocketType::IColor.into()),
    })
}

pub fn node() -> NodeResult {
    Ok(node! {
        inputs:
            "width": (None, SocketType::Value),
            "height": (None, SocketType::Value),

            "red": (None, SocketType::Value),
            "green": (None, SocketType::Value),
            "blue": (None, SocketType::Value),
        outputs:
            "color": SocketType::IColor.into();
        |inputs, outputs| {
            get_sv!( input | inputs  . "width": Value > width);
            get_sv!( input | inputs  . "height": Value > height);

            get_sv!( input | inputs  . "red": Value > red);
            get_sv!( input | inputs  . "green": Value > green);
            get_sv!( input | inputs  . "blue": Value > blue);

            get_sv!(output | outputs . "color": IColor > out);

            handle_missing_socket_values![width, height, red, green, blue];

            let mut res = Image::new(*width as u32, *height as u32, Color::new(*red, *green, *blue));

            out.replace(res);

            Ok(())
        }
    })
}
