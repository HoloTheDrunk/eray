//! Mapper from three [Number](SocketValue::Number) values to a [Color] image.
//!
//! Mandatory inputs:
//! - red: Number
//! - green: Number
//! - blue: Number
//!
//! Output:
//! - width: Number, width of the output image, defaults to 1
//! - height: Number, height of the output image, defaults to 1
//! - color: Color

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
            "red": SocketType::Number.into(),
            "green": SocketType::Number.into(),
            "blue": SocketType::Number.into(),

            // Optional
            "width": SocketValue::Number(Some(1.)),
            "height": SocketValue::Number(Some(1.)),
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
            "color": (ssref!(node "converter" "color"), SocketType::Color.into()),
    })
}

pub fn node() -> NodeResult {
    Ok(node! {
        inputs:
            "width": (None, SocketType::Number),
            "height": (None, SocketType::Number),

            "red": (None, SocketType::Number),
            "green": (None, SocketType::Number),
            "blue": (None, SocketType::Number),
        outputs:
            "color": SocketType::Color.into();
        |inputs, outputs| {
            get_sv!( input | inputs  . "width": Number > width);
            get_sv!( input | inputs  . "height": Number > height);

            get_sv!( input | inputs  . "red": Number > red);
            get_sv!( input | inputs  . "green": Number > green);
            get_sv!( input | inputs  . "blue": Number > blue);

            get_sv!(output | outputs . "color": Color > out);

            handle_missing_socket_values![width, height, red, green, blue];

            let mut res = Image::new(*width as u32, *height as u32, Color::new(*red, *green, *blue));

            out.replace(res);

            Ok(())
        }
    })
}
