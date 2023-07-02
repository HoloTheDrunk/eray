//! Mix two colors together by a factor.
//!
//! Mandatory inputs:
//! - width: [Value](SocketValue::IValue), width of the output image
//! - height: [Value](SocketValue::IValue), height of the output image
//! - left: [Color](SocketValue::IColor)
//! - right: [Color](SocketValue::IColor)
//!
//! Optional inputs:
//! - factor: [Value](SocketValue::IValue), mixing factor
//!   - used in `left * (1 - factor) + factor * right`
//!   - default: `DEFAULT_FACTOR`
//!
//! Output:
//! - color: [Color](SocketValue::IColor)

use super::{utils::handle_missing_socket_values, GraphResult, NodeResult};

use eray::{
    get_sv, node,
    prelude::*,
    shader::{self, graph::*},
    ssref,
};

const DEFAULT_FACTOR: f32 = 0.5;

/// Get a wrapping [Graph](Graph) containing the node.
pub fn graph() -> GraphResult {
    Ok(shader::graph::graph! {
        inputs:
            // Mandatory
            "width": SocketType::IValue.into(),
            "height": SocketType::IValue.into(),

            "left": SocketType::IColor.into(),
            "right": SocketType::IColor.into(),

            // Optional
            "factor": SocketValue::Value(Some(DEFAULT_FACTOR)),
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
            "color": (ssref!(node "mix" "color"), SocketType::IColor.into()),
    })
}

/// Get the [node](Node::Graph) by itself.
pub fn node() -> NodeResult {
    Ok(node! {
        inputs:
            "width": (None, SocketType::IValue),
            "height": (None, SocketType::IValue),

            "left": (None, SocketType::IColor),
            "right": (None, SocketType::IColor),

            "factor": (None, SocketType::IValue),
        outputs:
            "color": SocketType::IColor.into();
        |inputs, outputs| {
            get_sv!( input | inputs  . "width": Value > width);
            get_sv!( input | inputs  . "height": Value > height);

            get_sv!( input | inputs  . "left": IColor > left);
            get_sv!( input | inputs  . "right": IColor > right);

            get_sv!( input | inputs  . "factor": Value > factor);

            get_sv!(output | outputs . "color": IColor > out);

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
