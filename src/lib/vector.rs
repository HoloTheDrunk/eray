//! 3D vector definition

use super::{color::Color, DefaultType, DEFAULT_DIM};

use std::ops::{Add, AddAssign, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Sub, SubAssign};

use paste::paste;

#[derive(PartialEq, Clone, Copy, Debug)]
/// DIM-dimensional vector of TYPE values.
pub struct Vector<const DIM: usize = DEFAULT_DIM, TYPE = DefaultType> {
    /// Coordinate vector.
    pub inner: [TYPE; DIM],
}

impl<const DIM: usize, TYPE: Default + Copy> Default for Vector<DIM, TYPE> {
    fn default() -> Self {
        Self {
            inner: [TYPE::default(); DIM],
        }
    }
}

impl<const DIM: usize, TYPE> Index<usize> for Vector<DIM, TYPE> {
    type Output = TYPE;

    fn index(&self, index: usize) -> &Self::Output {
        &self.inner[index]
    }
}

impl<const DIM: usize, TYPE> IndexMut<usize> for Vector<DIM, TYPE> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.inner[index]
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
            impl<const DIM: usize, TYPE: Copy + [<$trait Assign>]<TYPE>> [<$trait Assign>]<Self> for Vector<DIM, TYPE> {
                fn [<$function _assign>](&mut self, rhs: Self) {
                    for (l, r) in self.inner.iter_mut().zip(rhs.inner.iter()) {
                        *l $($op)+ *r;
                    }
                }
            }

            impl<const DIM: usize, TYPE: Copy + [<$trait Assign>]<TYPE>> $trait<Self> for Vector<DIM, TYPE> {
                type Output = Self;

                fn $function(mut self, rhs: Self) -> Self::Output {
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
            impl<const DIM: usize, TYPE: Copy + [<$trait Assign>]<TYPE>> [<$trait Assign>]<TYPE> for Vector<DIM, TYPE> {
                fn [<$function _assign>](&mut self, rhs: TYPE) {
                    for v in self.inner.as_mut_slice() {
                        *v $($op)+ rhs;
                    }
                }
            }

            impl<const DIM: usize, TYPE: Copy + [<$trait Assign>]<TYPE>> $trait<TYPE> for Vector<DIM, TYPE> {
                type Output = Self;

                fn $function(mut self, rhs: TYPE) -> Self::Output {
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
        TYPE: Default
            + Copy
            + Add<Output = TYPE>
            + Mul<Output = TYPE>
            + Div<Output = TYPE>
            + DivAssign<TYPE>
            + From<f32>
            + Into<f32>,
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

    #[inline]
    /// Performs `above / self` element-wise.
    pub fn div_under(mut self, above: TYPE) -> Self {
        for v in self.inner.iter_mut() {
            *v = above / *v;
        }

        self
    }
}

impl<const DIM: usize, TYPE: Copy + Default + Add<Output = TYPE> + Mul<Output = TYPE>>
    Vector<DIM, TYPE>
{
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

impl<TYPE: Copy + Mul<Output = TYPE> + Sub<TYPE, Output = TYPE>> Vector<3, TYPE> {
    /// Perform cross product with `other`.
    pub fn cross_product(&self, other: &Self) -> Self {
        Vector {
            inner: [
                other.inner[2] * self.inner[1] - self.inner[2] * other.inner[1],
                other.inner[0] * self.inner[2] - self.inner[0] * other.inner[2],
                other.inner[1] * self.inner[0] - self.inner[1] * other.inner[0],
            ],
        }
    }
}

impl From<Color> for Vector<3, f32> {
    fn from(Color { r, g, b }: Color) -> Self {
        Self { inner: [r, g, b] }
    }
}

impl<const DIM: usize, TYPE: Copy + Into<f32>> From<TYPE> for Vector<DIM, TYPE> {
    fn from(value: TYPE) -> Self {
        Self {
            inner: [value; DIM],
        }
    }
}

impl<const DIM: usize, TYPE: Into<f32>> From<Vector<DIM, TYPE>> for f32 {
    fn from(val: Vector<DIM, TYPE>) -> Self {
        let len = val.inner.len();
        val.inner.into_iter().map(Into::into).sum::<f32>() / len as f32
    }
}

impl<const DIM: usize, TYPE: Default + Copy> Vector<DIM, TYPE> {
    /// Change a vector's dimensionality, filling the missing values with the default one if needed.
    pub fn resize<const NEW_DIM: usize>(value: Vector<DIM, TYPE>) -> Vector<NEW_DIM, TYPE> {
        let mut v = Vector::<NEW_DIM, TYPE>::default();

        for i in 0..DIM.min(NEW_DIM) {
            v[i] = value[i];
        }

        v
    }
}

#[cfg(test)]
mod test {
    use float_eq::assert_float_eq;

    use super::*;

    fn get_vecs() -> (Vector, Vector) {
        (Vector::new(1., 2., -3.), Vector::new(-1.5, 2.3, 0.1))
    }

    #[test]
    /// This will break if generics are very poorly defined.
    fn generics() {
        Vector::<3, f32>::new(1., 2., 3.).normalize();
    }

    #[test]
    fn dot_product() {
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
    fn cross_product() {
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
    fn angle() {
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
