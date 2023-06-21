//! Mapper from three [Number](SocketValue::Number) values to a [Color] image.
//!
//! Mandatory inputs:
//! - width: Number, width of the output image
//! - height: Number, height of the output image
//! - red: Value
//! - green: Value
//! - blue: Value
//!
//! Output:
//! - color: Color

use crate::handle_missing_socket_values;

use super::{GraphResult, MaterialResult, NodeResult};

use eray::{
    get_sv, node,
    prelude::*,
    shader::{
        self,
        graph::{Graph, SocketType},
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
            "width": SocketType::Number.into(),
            "height": SocketType::Number.into(),

            "red": SocketType::Value.into(),
            "green": SocketType::Value.into(),
            "blue": SocketType::Value.into(),
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

            "red": (None, SocketType::Value),
            "green": (None, SocketType::Value),
            "blue": (None, SocketType::Value),
        outputs:
            "color": SocketType::Color.into();
        |inputs, outputs| {
            get_sv!( input | inputs  . "width": Number > width);
            get_sv!( input | inputs  . "height": Number > height);

            get_sv!( input | inputs  . "red": Value > red);
            get_sv!( input | inputs  . "green": Value > green);
            get_sv!( input | inputs  . "blue": Value > blue);

            get_sv!(output | outputs . "color": Color > out);

            handle_missing_socket_values![width, height, red, green, blue];

            let mut res = Image::new(*width as u32, *height as u32, Color::new(0., 0., 0.));

            for y in 0..res.height {
                for x in 0..res.width {
                    let index = (y * res.width + x) as usize;
                    let value = Color::new(red.pixels[index], green.pixels[index], blue.pixels[index]);
                    res.pixels[index] = value;
                }
            }
            out.replace(res);

            Ok(())
        }
    })
}
