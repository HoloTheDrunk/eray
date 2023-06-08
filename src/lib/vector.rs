//! 3D vector definition

use paste::paste;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

#[derive(PartialEq, Clone, Copy, Debug)]
pub struct Vector<const DIM: usize = 3, TYPE = f32> {
    inner: [TYPE; DIM],
}

impl<const DIM: usize, TYPE: Default + Copy> Default for Vector<DIM, TYPE> {
    fn default() -> Self {
        Self {
            inner: [TYPE::default(); DIM],
        }
    }
}

macro_rules! into_primitive_array {
    ($($target:ty),+ $(,)?) => {
        $(
            impl<const DIM: usize> From<Vector<DIM, $target>> for [$target; DIM] {
                fn from(value: Vector<DIM, $target>) -> Self {
                    value.inner
                }
            }

            impl<const DIM: usize> From<[$target; DIM]> for Vector<DIM, $target> {
                fn from(value: [$target; DIM]) -> Self {
                    Self {
                        inner: value
                    }
                }
            }

            impl<const DIM: usize> From<&[$target]> for Vector<DIM, $target> {
                fn from(value: &[$target]) -> Self {
                    let mut res = [<$target as Default>::default(); DIM];
                    for i in 0..DIM {
                        res[i] = value[i];
                    }

                    Self {
                        inner: res
                    }
                }
            }
        )+
    };
}

into_primitive_array!(i32, i64, f32, f64);

macro_rules! impl_vec_vec_op {
    ($trait:ident, $function:ident, $($op:tt)+) => {
        paste! {
            impl<const DIM: usize, TYPE: [<$trait Assign>]<TYPE>> [<$trait Assign>]<Self> for Vector<DIM, TYPE> {
                fn [<$function _assign>](&mut self, rhs: Self) {
                    for (l, r) in self.inner.iter_mut().zip(rhs.inner.iter()) {
                        *l $($op)+ *r;
                    }
                }
            }

            impl<const DIM: usize, TYPE: [<$trait Assign>]<TYPE>> $trait<Self> for Vector<DIM, TYPE> {
                type Output = Self;

                fn $function(self, rhs: Self) -> Self::Output {
                    self $($op)+ rhs;
                    self
                }
            }
        }
    }
}

impl_vec_vec_op! (Add, add, +=);
impl_vec_vec_op! (Sub, sub, -=);

macro_rules! impl_vec_type_op {
    ($trait:ident, $function:ident, $($op:tt)+) => {
        paste! {
            impl<const DIM: usize, TYPE: [<$trait Assign>]<TYPE>> [<$trait Assign>]<TYPE> for Vector<DIM, TYPE> {
                fn [<$function _assign>](&mut self, rhs: TYPE) {
                    for v in self.inner.as_mut_slice() {
                        *v $($op)+ rhs;
                    }
                }
            }

            impl<const DIM: usize, TYPE: [<$trait Assign>]<TYPE>> $trait<TYPE> for Vector<DIM, TYPE> {
                type Output = Self;

                fn $function(self, rhs: TYPE) -> Self::Output {
                    self $($op)+ rhs;
                    self
                }
            }
        }
    }
}

impl_vec_type_op! (Add, add, +=);
impl_vec_type_op! (Sub, sub, -=);
impl_vec_type_op! (Mul, mul, *=);
impl_vec_type_op! (Div, div, /=);

impl<TYPE> Vector<3, TYPE> {
    /// Create a new 3D vector from values.
    pub fn new<T: Into<TYPE>>(x: T, y: T, z: T) -> Self {
        Self {
            inner: [x.into(), y.into(), z.into()],
        }
    }
}

impl<
        const DIM: usize,
        TYPE: Default + Add<Output = TYPE> + Mul<Output = TYPE> + DivAssign<TYPE> + From<f32> + Into<f32>,
    > Vector<DIM, TYPE>
{
    #[inline]
    /// Get length of the vector.
    pub fn len(&self) -> f32 {
        self.len_sq().into().sqrt()
    }

    #[inline]
    /// Get normalized vector pointing in the same direction.
    pub fn normalize(&self) -> Self {
        *self / self.len().into()
    }

    /// Get angle to `other` vector.
    pub fn angle_to(&self, other: &Self) -> f32 {
        let dot = self.dot_product(other).into();
        let res = dot / (self.len() * other.len());
        res.acos()
    }
}

impl<const DIM: usize, TYPE: Default + Add<Output = TYPE> + Mul<Output = TYPE>> Vector<DIM, TYPE> {
    #[inline]
    /// Get squared length of the vector, slightly faster than [Vec3::len].
    pub fn len_sq(&self) -> TYPE {
        self.dot_product(self)
    }

    /// Perform dot product with `other`.
    pub fn dot_product(&self, other: &Self) -> TYPE {
        self.inner
            .iter()
            .zip(other.inner.iter())
            .fold(TYPE::default(), |acc, cur| acc + *cur.0 * *cur.1)
    }
}

impl<TYPE: Mul<Output = TYPE> + Sub<TYPE, Output = TYPE>> Vector<3, TYPE> {
    /// Perform cross product with `other`.
    pub fn cross_product(&self, other: &Self) -> Self {
        Vector {
            inner: [
                other.inner[2] * self.inner[1] - self.inner[0] * other.inner[1],
                other.inner[0] * self.inner[2] - self.inner[0] * other.inner[2],
                other.inner[1] * self.inner[0] - self.inner[1] * other.inner[0],
            ],
        }
    }
}

impl<const DIM: usize, TYPE: Default> Vector<DIM, TYPE> {}

#[cfg(test)]
mod test {
    use float_eq::assert_float_eq;

    use super::*;

    fn get_vecs() -> (Vector, Vector) {
        (Vector::new(1., 2., -3.), Vector::new(-1.5, 2.3, 0.1))
    }

    #[test]
    fn test_dot_product() {
        let (first, second) = get_vecs();
        let got = first.dot_product(&second);
        let expected = 2.8;
        assert_float_eq!(
            expected,
            got,
            abs <= 0.000_1,
            "Invalid dot product result {got}, expected {expected}"
        );
    }

    #[test]
    fn test_cross_product() {
        let (first, second) = get_vecs();

        let got: Vector<3, f32> = first.cross_product(&second);
        let expected: Vector<3, f32> = Vector::new(7.1, 4.4, 5.3);
        let comp = (got - expected).len_sq();

        assert!(
            comp < 0.000_1,
            "Invalid cross product result {got:?}, expected {expected:?}"
        );
    }

    #[test]
    fn test_angle() {
        let (first, second) = get_vecs();

        let got = first.angle_to(&second);
        let expected = 1.2949;

        assert_float_eq!(
            expected,
            got,
            abs <= 0.000_1,
            "Invalid angle {got}, expected {expected}"
        );
    }
}
