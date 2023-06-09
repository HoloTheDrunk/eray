//! Structs required for raycasting

use crate::color::Color;

use super::vector::Vec3;

#[derive(Clone, Debug, Default)]
/// A ray defined by its start position and direction.
pub struct Ray {
    start: Vec3,
    dir: Vec3,
}

impl Ray {
    /// Create a new [Ray] from a position and direction.
    pub fn new(start: Vec3, dir: Vec3) -> Self {
        Self {
            start,
            dir: dir.normalize(),
        }
    }

    /// Get position at `t` along ray.
    pub fn calc(&self, t: f32) -> Vec3 {
        self.start + self.dir * t
    }

    #[inline]
    /// Get starting position
    pub fn start(&self) -> &Vec3 {
        &self.start
    }

    #[inline]
    /// Get direction
    pub fn dir(&self) -> &Vec3 {
        &self.dir
    }
}

#[derive(Debug)]
/// Information about the hit and hit object.
pub struct RaycastHit {
    /// ID of the hit face of the hit object.
    pub face_index: usize,
    /// World-space position of the hit.
    pub position: Vec3,
    /// World-space direction of the normal at the hit's position.
    pub normal: Vec3,

    /// Color of the hit face.
    pub color: Color,

    /// Diffuse property of the hit face (k_d).
    pub diffuse: f32,
    /// Specular property of the hit face (k_s).
    pub specular: f32,
    /// Specular power property of the hit face.
    pub specular_power: f32,
}
