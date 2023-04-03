pub mod color;
pub mod image;
pub mod matrix;
pub mod object;
pub mod primitives;
pub mod raycasting;
pub mod shader;
pub mod vector;

macro_rules! states {
    ($($state:tt),+ $(,)?) => {
        $(
            #[derive(Clone, Debug, Default)]
            pub struct $state;
        )+
    };
}

states!(Building, Built);
