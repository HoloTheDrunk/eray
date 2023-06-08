//! Basic primitives necessary for rendering

use crate::{raycasting::Ray, vector::Vec3};

#[derive(Debug, Default)]
/// A mesh vertex.
pub struct Vertex {
    /// XYZ position.
    pub position: Vec3,
    /// XYZ normal.
    pub normal: Vec3,
    /// UV(W) texture coordinates.
    pub uv: Vec3,
}

#[derive(Debug, Default)]
/// Group of 3 vertices.
///
/// The surface normal is always calculated as (b - a) x (c - a).
pub struct Triangle {
    #[allow(missing_docs)]
    pub a: Vertex,
    #[allow(missing_docs)]
    pub b: Vertex,
    #[allow(missing_docs)]
    pub c: Vertex,

    normal: Vec3,
}

impl Triangle {
    /// Create a new Triangle from three vertices. The surface normal is computed here.
    pub fn new(a: Vertex, b: Vertex, c: Vertex) -> Self {
        let normal = (b.position - a.position).cross_product(&(c.position - a.position));
        Self { a, b, c, normal }
    }

    /// Check for intersection with the provided [Ray] with backface culling.
    ///
    /// Returns a world-space position, world-space normalized surface normal vector and a barycentric position.
    pub fn intersects(&self, ray: &Ray) -> Option<(Vec3, Vec3, Vec3)> {
        // TODO: Check for ray / normal match to do backface culling
        let [a, b, c] = [self.a.position, self.b.position, self.c.position];

        let e1 = b - a;
        let e2 = c - a;
        let n = e1.cross_product(&e2);

        // Backface culling
        if n.dot_product(ray.dir()) > 0. {
            return None;
        }

        let det = -ray.dir().dot_product(&n);
        let invdet = 1. / det;

        let ao = *ray.start() - a;
        let dao = ao.cross_product(ray.dir());

        // Barycentric coordinates
        let u = e2.dot_product(&dao) * invdet;
        let v = -e1.dot_product(&dao) * invdet;
        let t = ao.dot_product(&n) * invdet;

        (det >= 1e-6 && t >= 0. && u >= 0. && v >= 0. && (u + v) <= 1.0).then(|| {
            (
                *ray.start() + *ray.dir() * t,
                (self.a.normal * u + self.b.normal * v + self.c.normal * t).normalize(),
                // TODO: This is invalid, figure out how the fuck barycentric coordinates work
                Vec3::new(u, v, 1. - u - v),
            )
        })
    }

    /// Returns the projected coordinates of the point on the triangle.
    pub fn project(&self, point: Vec3) -> Vec3 {
        let v = point - self.a.position;
        let dist = v.dot_product(&self.normal);
        let res = point - self.normal * dist;

        res
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn projection() {
        let triangle = Triangle::new(
            Vertex {
                position: Vec3::new(-0.5, 0., -0.5),
                normal: Vec3::new(0., 1., 0.),
                uv: Vec3::new(0., 0., 0.),
            },
            Vertex {
                position: Vec3::new(0., 0., 0.5),
                normal: Vec3::new(0., 1., 0.),
                uv: Vec3::new(0.5, 1., 0.),
            },
            Vertex {
                position: Vec3::new(0.5, 0., -0.5),
                normal: Vec3::new(0., 1., 0.),
                uv: Vec3::new(1., 0., 0.),
            },
        );

        let point = Vec3::new(0.2, 0.1, 0.);
        let proj = triangle.project(point);

        assert_eq!(Vec3::new(0.2, 0., 0.), proj);
    }
}
