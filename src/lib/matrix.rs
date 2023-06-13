//! Basic 4x4 matrix implementation

use std::ops::Mul;

use crate::vector::Vector;

#[derive(Clone, Debug, Default)]
/// 4x4 matrix
pub struct Mat4 {
    /// Arrays storing the matrix data
    pub inner: [[f32; 4]; 4],
}

#[derive(Clone, Debug, Default)]
/// 3D transformation representation
pub struct Transform {
    inner: Mat4,

    translation: Vector<3, f32>,
    scale: Vector<3, f32>,
    rotation: Vector<3, f32>,
}

impl Mul<Mat4> for Mat4 {
    type Output = Mat4;

    fn mul(self, rhs: Mat4) -> Self::Output {
        let mut res = Mat4::default();

        for i in 0..4 {
            for j in 0..4 {
                for k in 0..4 {
                    res.inner[i][j] += self.inner[i][k] * rhs.inner[k][j];
                }
            }
        }

        res
    }
}

impl Transform {
    fn new_translation(delta: Vector<3, f32>) -> Mat4 {
        let mut res = Mat4::default();

        res.inner[0][3] = delta[0];
        res.inner[1][3] = delta[1];
        res.inner[2][3] = delta[2];

        res
    }

    fn new_scaling(delta: Vector<3, f32>) -> Mat4 {
        let mut res = Mat4::default();

        res.inner[0][0] = delta[0];
        res.inner[1][1] = delta[1];
        res.inner[2][2] = delta[2];

        res
    }

    fn new_rotation(axis: Vector<3, f32>, angle: f32) -> Mat4 {
        let mut res = Mat4::default();

        let asin = angle.sin();
        let acos = angle.cos();
        let ncos = 1. - acos;

        res.inner[0][0] = acos + axis[0].powi(2) * ncos;
        res.inner[0][1] = axis[0] * axis[1] * ncos - axis[2] * asin;
        res.inner[0][2] = axis[0] * axis[2] * ncos + axis[1] * asin;

        res.inner[1][0] = axis[1] * axis[0] * ncos + axis[2] * asin;
        res.inner[1][1] = acos + axis[1].powi(2) * ncos;
        res.inner[1][2] = axis[1] * axis[2] * ncos - axis[0] * asin;

        res.inner[2][0] = axis[2] * axis[0] * ncos - axis[1] * asin;
        res.inner[2][1] = axis[2] * axis[1] * ncos - axis[0] * asin;
        res.inner[2][2] = acos + axis[2].powi(2) * ncos;

        res
    }

    /// Add a translation of `delta` to the [Transform]
    pub fn translate(mut self, delta: Vector<3, f32>) -> Self {
        self.translation += delta;
        self.inner = self.inner * Transform::new_translation(delta);
        self
    }

    /// Scale by `delta`
    pub fn scale(mut self, delta: Vector<3, f32>) -> Self {
        self.scale += delta;
        self.inner = self.inner * Transform::new_scaling(delta);
        self
    }

    /// Rotate by `angle` around `axis`
    pub fn rotate(mut self, axis: Vector<3, f32>, angle: f32) -> Self {
        self.rotation += axis * angle;
        self.inner = self.inner * Transform::new_rotation(axis.normalize(), angle);
        self
    }
}
