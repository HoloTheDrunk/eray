//! Mix two colors together by a factor.
//!
//! Mandatory inputs:
//! - width: [Number](SocketValue::Number), width of the output image
//! - height: [Number](SocketValue::Number), height of the output image
//! - left: [Color](SocketValue::Color)
//! - right: [Color](SocketValue::Color)
//!
//! Optional inputs:
//! - factor: [Number](SocketValue::Number), mixing factor
//!   - used in `left * (1 - factor) + factor * right`
//!   - default: `DEFAULT_FACTOR`
//!
//! Output:
//! - color: [Color](SocketValue::Color)

use super::{utils::handle_missing_socket_values, GraphResult, MaterialResult, NodeResult};

use eray::{
    get_sv, node,
    prelude::*,
    shader::{self, graph::*},
    ssref,
};

use map_macro::hash_map;

const DEFAULT_FACTOR: f32 = 0.5;

/// Get a ready-to-use [Material](Material).
pub fn material() -> MaterialResult {
    Ok(Material::from((
        graph()?.validate()?,
        hash_map! {
            StandardMaterialOutput::Color => "color".into(),
        },
    )))
}

/// Get a wrapping [Graph](Graph) containing the node.
pub fn graph() -> GraphResult {
    Ok(shader::graph::graph! {
        inputs:
            // Mandatory
            "width": SocketType::Number.into(),
            "height": SocketType::Number.into(),

            "left": SocketType::Color.into(),
            "right": SocketType::Color.into(),

            // Optional
            "factor": SocketValue::Number(Some(DEFAULT_FACTOR)),
        nodes:
            "mix": {
                let mut node = node()?;
                node.set_input(&"width".into(), ssref!(graph "width"))?
                    .set_input(&"height".into(), ssref!(graph "height"))?
                    .set_input(&"left".into(), ssref!(graph "left"))?
                    .set_input(&"right".into(), ssref!(graph "right"))?
                    .set_input(&"factor".into(), ssref!(graph "factor"))?;
                node
            },
        outputs:
            "color": (ssref!(node "mix" "color"), SocketType::Color.into()),
    })
}

/// Get the [node](Node::Graph) by itself.
pub fn node() -> NodeResult {
    Ok(node! {
        inputs:
            "width": (None, SocketType::Number),
            "height": (None, SocketType::Number),

            "left": (None, SocketType::Color),
            "right": (None, SocketType::Color),

            "factor": (None, SocketType::Number),
        outputs:
            "color": SocketType::Color.into();
        |inputs, outputs| {
            get_sv!( input | inputs  . "width": Number > width);
            get_sv!( input | inputs  . "height": Number > height);

            get_sv!( input | inputs  . "left": Color > left);
            get_sv!( input | inputs  . "right": Color > right);

            get_sv!( input | inputs  . "factor": Number > factor);

            get_sv!(output | outputs . "color": Color > out);

            handle_missing_socket_values![width, height, left, right];
            let factor = factor.unwrap_or(DEFAULT_FACTOR);

            let mut res = Image::new(*width as u32, *height as u32, Color::default());

            for y in 0..(res.height) {
                for x in 0..(res.width) {
                    let index = (y * res.width + x) as usize;

                    let interp = |l, r| l * (1. - factor) + r * factor;
                    let (lpx, rpx) = (left.mod_get(x, y), right.mod_get(x, y));
                    let value = Color::new(interp(lpx.r, rpx.r), interp(lpx.g, rpx.g), interp(lpx.b, rpx.b));

                    res.pixels[index] = value;
                }
            }

            out.replace(res);

            Ok(())
        }
    })
}
