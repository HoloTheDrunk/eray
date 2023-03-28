use ::derive_more::{Add, AddAssign, Neg, Sub, SubAssign};
use std::ops::{Div, Mul, MulAssign};

#[derive(Add, AddAssign, Sub, SubAssign, Neg, PartialEq, Clone, Copy, Debug, Default)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

macro_rules! into_primitive_array {
    ($($target:ty),+ $(,)?) => {
        $(
            impl From<Vec3> for [$target; 3] {
                fn from(value: Vec3) -> Self {
                    [value.x as $target, value.y as $target, value.z as $target]
                }
            }
        )+
    };
}

into_primitive_array!(i32, i64, f32, f64);

impl Vec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Vec3 { x, y, z }
    }

    #[inline]
    pub fn len_sq(&self) -> f32 {
        self.dot_product(self)
    }

    #[inline]
    pub fn len(&self) -> f32 {
        self.len_sq().sqrt()
    }

    #[inline]
    pub fn normalize(&self) -> Vec3 {
        *self / self.len()
    }

    pub fn dot_product(&self, other: &Vec3) -> f32 {
        macro_rules! dot_product {
            ($l:ident, $r:ident | $($field:ident),*) => {
                0. $( + $l.$field * $r.$field )*
            };
        }

        dot_product!(self, other | x, y, z)
        // self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn cross_product(&self, other: &Vec3) -> Vec3 {
        Vec3 {
            x: other.z * self.y - self.z * other.y,
            y: other.x * self.z - self.x * other.z,
            z: other.y * self.x - self.y * other.x,
        }
    }

    pub fn angle_to(&self, other: &Vec3) -> f32 {
        let dot = self.dot_product(other);
        let res = dot / (self.len() * other.len());
        res.acos()
    }
}

impl Mul<f32> for Vec3 {
    type Output = Self;

    fn mul(mut self, rhs: f32) -> Self::Output {
        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;

        self
    }
}

impl MulAssign<f32> for Vec3 {
    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
    }
}

impl Div<f32> for Vec3 {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        self * (1. / rhs)
    }
}

impl Mul for Vec3 {
    type Output = f32;

    fn mul(self, rhs: Self) -> Self::Output {
        self.dot_product(&rhs)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn get_vecs() -> (Vec3, Vec3) {
        (Vec3::new(1., 2., -3.), Vec3::new(-1.5, 2.3, 0.1))
    }

    #[test]
    fn test_dot_product() {
        let (first, second) = get_vecs();
        assert_eq!(first * second, 2.8);
    }

    #[test]
    fn test_cross_product() {
        let (first, second) = get_vecs();
        assert_eq!(first.cross_product(&second), Vec3::new(7.1, 4.4, 5.3));
    }

    #[test]
    fn test_angle() {
        let (first, second) = get_vecs();

        let got = first.angle_to(&second);
        let expected = 1.294;
        let comp = (got - expected).abs();

        assert!(comp < 0.001, "Invalid angle {got}, expected {expected}");
    }
}
