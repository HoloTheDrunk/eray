//! Light definition.

use crate::{color::Color, matrix::Transform};

#[derive(Clone, Debug)]
/// Light object that adds... light.
pub struct Light {
    /// 3D transform.
    pub transform: Transform,
    /// Type of light.
    pub variant: LightVariant,
    /// Light color.
    pub color: Color,
    /// Light brightness level.
    pub brightness: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
/// Different types of lights that behave differently.
pub enum LightVariant {
    /// Point light that shines in all directions.
    Point,
    /// Ambient light pointing in a certain direction.
    Ambient,
}
