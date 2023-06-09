#![warn(missing_docs)]

//! Library used by the eray software. Provides a simple shader node graph and basic functionality
//! useful for writing rendering applications.

pub mod color;
pub mod image;
pub mod matrix;
pub mod object;
pub mod primitives;
pub mod raycasting;
pub mod shader;
pub mod vector;

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
