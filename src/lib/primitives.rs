//! Basic primitives necessary for rendering

use crate::{raycasting::Ray, vector::Vec3};

#[derive(Debug, Default)]
/// A mesh vertex
pub struct Vertex {
    /// XYZ position
    pub position: Vec3,
    /// XYZ normal
    pub normal: Vec3,
}

#[derive(Debug, Default)]
/// Group of 3 vertices
pub struct Triangle {
    #[allow(missing_docs)]
    pub a: Vertex,
    #[allow(missing_docs)]
    pub b: Vertex,
    #[allow(missing_docs)]
    pub c: Vertex,
}

impl Triangle {
    /// Check for intersection with the provided [Ray] with backface culling.
    pub fn intersects(&self, ray: &Ray) -> Option<(Vec3, Vec3)> {
        // TODO: Check for ray / normal match to do backface culling
        let [a, b, c] = [self.a.position, self.b.position, self.c.position];

        let e1 = b - a;
        let e2 = c - a;
        let n = e1.cross_product(&e2);

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
            )
        })
    }
}
