//! Structs required for raycasting

use super::vector::Vector;

use crate::material::MaterialOutputBundle;

#[derive(Clone, Debug, Default)]
/// A ray defined by its start position and direction.
pub struct Ray {
    start: Vector<3, f32>,
    dir: Vector<3, f32>,
}

impl Ray {
    /// Create a new [Ray] from a position and direction.
    pub fn new(start: Vector<3, f32>, dir: Vector<3, f32>) -> Self {
        Self {
            start,
            dir: dir.normalize(),
        }
    }

    /// Get position at `t` along ray.
    pub fn calc(&self, t: f32) -> Vector<3, f32> {
        self.start + self.dir * t
    }

    #[inline]
    /// Get starting position
    pub fn start(&self) -> &Vector<3, f32> {
        &self.start
    }

    #[inline]
    /// Get direction
    pub fn dir(&self) -> &Vector<3, f32> {
        &self.dir
    }
}

#[derive(Debug)]
/// Information about the hit and hit object.
pub struct RaycastHit {
    /// ID of the hit face of the hit object.
    pub face_index: usize,

    /// World-space position of the hit.
    pub position: Vector<3, f32>,
    /// World-space direction of the normal at the hit's position.
    pub normal: Vector<3, f32>,

    /// Material properties at the hit point
    pub material: MaterialOutputBundle,
}
