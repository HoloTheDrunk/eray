//! Mapper from three [Value image](SocketType::IValue)s to a [Color image](SocketType::IColor).
//!
//! Mandatory inputs:
//! - width: Value, width of the output image
//! - height: Value, height of the output image
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
            "width": SocketType::Value.into(),
            "height": SocketType::Value.into(),

            "red": SocketType::IValue.into(),
            "green": SocketType::IValue.into(),
            "blue": SocketType::IValue.into(),
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

            "red": (None, SocketType::IValue),
            "green": (None, SocketType::IValue),
            "blue": (None, SocketType::IValue),
        outputs:
            "color": SocketType::IColor.into();
        |inputs, outputs| {
            get_sv!( input | inputs  . "width": Value > width);
            get_sv!( input | inputs  . "height": Value > height);

            get_sv!( input | inputs  . "red": IValue > red);
            get_sv!( input | inputs  . "green": IValue > green);
            get_sv!( input | inputs  . "blue": IValue > blue);

            get_sv!(output | outputs . "color": IColor > out);

            handle_missing_socket_values![width, height, red, green, blue];

            let mut res = Image::new(*width as u32, *height as u32, Color::new(0., 0., 0.));

            for y in 0..res.height {
                for x in 0..res.width {
                    let index = (y * res.width + x) as usize;
                    let value = Color::new(red.pixels[index], green.pixels[index], blue.pixels[index]);
                    res.pixels[index] = value;
                }
            }
            res.save_as_ppm(std::path::Path::new("rgb.ppm"));
            out.replace(res);


            Ok(())
        }
    })
}
