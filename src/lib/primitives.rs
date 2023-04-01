use crate::{object::Object, raycasting::Ray, vector::Vec3};

#[derive(Clone, Debug, Default)]
pub struct VertexIndices {
    // Vertex position index in related object
    pub position: usize,
    // Vertex normal index in related object
    pub normal: usize,
}

#[derive(Debug, Default)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
}

impl Vertex {
    pub fn from_indices<State>(object: &Object<State>, indices: VertexIndices) -> Vertex {
        Vertex {
            position: object.vertices[indices.position],
            normal: object.normals[indices.normal],
        }
    }
}

#[derive(Debug, Default)]
pub struct Triangle {
    pub a: Vertex,
    pub b: Vertex,
    pub c: Vertex,
}

impl Triangle {
    pub fn intersects(&self, ray: &Ray) -> Option<(Vec3, Vec3)> {
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
