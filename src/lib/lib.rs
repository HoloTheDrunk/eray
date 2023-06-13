#![warn(missing_docs)]

//! Library used by the eray software. Provides a simple shader node graph and basic functionality
//! useful for writing rendering applications.

// TODO: package all of those into their own sub-modules because jesus christ.
pub mod camera;
pub mod color;
pub mod image;
pub mod light;
pub mod material;
pub mod matrix;
pub mod object;
pub mod primitives;
pub mod raycasting;
pub mod scene;
pub mod shader;
pub mod vector;

pub mod engine;

const DEFAULT_DIM: usize = 3;
type DefaultType = f32;

macro_rules! states {
    {$($(#[$attr:meta])* $state:ident),+ $(,)?} => {
        $(
            #[derive(Clone, Debug, Default)]
            $(#[$attr])*
            pub struct $state;
        )+
    };
}

states! {
    /// Building state where vertices, normals and other properties are filled.
    Building,
    /// Object's properties have been fixed and it can safely be used for rendering.
    Built,
    /// Object has been converted to an [OpenGLObject] and cannot be converted again.
    GLConsumed
}

/// Everything in the eray library.
pub mod prelude {
    pub use super::{
        camera::*, color::*, image::*, light::*, material::*, matrix::*, object::*, primitives::*,
        raycasting::*, scene::*, shader::*, vector::*,
    };
}
