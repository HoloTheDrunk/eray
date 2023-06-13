use std::path::Path;

use eray::{
    object::Object,
    primitives::*,
    raycasting::{Ray, RaycastHit},
    vector::Vector,
};

fn main() {
    println!("Hello, world!");

    // let triangle = Triangle::new(
    //     Vertex {
    //         position: Vec3::new(-0.5, 0., -0.5),
    //         normal: Vec3::new(0., 1., 0.),
    //         uv: Vec3::new(0., 0., 0.),
    //     },
    //     Vertex {
    //         position: Vec3::new(0., 0., 0.5),
    //         normal: Vec3::new(0., 1., 0.),
    //         uv: Vec3::new(0.5, 1., 0.),
    //     },
    //     Vertex {
    //         position: Vec3::new(0.5, 0., -0.5),
    //         normal: Vec3::new(0., 1., 0.),
    //         uv: Vec3::new(1., 0., 0.),
    //     },
    // );
    //
    // let point = Vec3::new(0., 0., 0.);
    // let dir = Vec3::new(0., -1., 0.);
    //
    // let ray = Ray::new(point, dir);
    //
    // let (_pos, _normal, barycentric) = triangle.intersects(&ray).unwrap();
    //
    // let test = triangle.b.position * barycentric.x
    //     + triangle.a.position * barycentric.y
    //     + triangle.c.position * barycentric.z;
    //
    // dbg!(test);
    // dbg!(barycentric);

    let cube = Object::load_obj(Path::new("./objects/cube.obj")).unwrap();
}
