use std::ops::Mul;

use crate::vector::Vec3;

#[derive(Debug, Default)]
pub struct Mat4 {
    pub inner: [[f32; 4]; 4],
}

#[derive(Debug, Default)]
pub struct Transform {
    inner: Mat4,

    translation: Vec3,
    scale: Vec3,
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
    fn new_translation(delta: Vec3) -> Mat4 {
        let mut res = Mat4::default();

        res.inner[0][3] = delta.x;
        res.inner[1][3] = delta.y;
        res.inner[2][3] = delta.z;

        res
    }

    fn new_scaling(delta: Vec3) -> Mat4 {
        let mut res = Mat4::default();

        res.inner[0][0] = delta.x;
        res.inner[1][1] = delta.y;
        res.inner[2][2] = delta.z;

        res
    }

    fn new_rotation(axis: Vec3, angle: f32) -> Mat4 {
        let mut res = Mat4::default();

        let asin = angle.sin();
        let acos = angle.cos();
        let ncos = 1. - acos;

        res.inner[0][0] = acos + axis.x.powi(2) * ncos;
        res.inner[0][1] = axis.x * axis.y * ncos - axis.z * asin;
        res.inner[0][2] = axis.x * axis.z * ncos + axis.y * asin;

        res.inner[1][0] = axis.y * axis.x * ncos + axis.z * asin;
        res.inner[1][1] = acos + axis.y.powi(2) * ncos;
        res.inner[1][2] = axis.y * axis.z * ncos - axis.x * asin;

        res.inner[2][0] = axis.z * axis.x * ncos - axis.y * asin;
        res.inner[2][1] = axis.z * axis.y * ncos - axis.x * asin;
        res.inner[2][2] = acos + axis.z.powi(2) * ncos;

        res
    }

    pub fn translate(mut self, delta: Vec3) -> Self {
        self.translation += delta;
        self.inner = self.inner * Transform::new_translation(delta);
        self
    }

    pub fn scale(mut self, delta: Vec3) -> Self {
        self.scale += delta;
        self.inner = self.inner * Transform::new_scaling(delta);
        self
    }

    pub fn rotate(mut self, axis: Vec3, angle: f32) -> Self {
        self.inner = self.inner * Transform::new_rotation(axis.normalize(), angle);
        self
    }
}
