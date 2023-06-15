use std::path::Path;

use eray::{engine::*, prelude::*};

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

    // let cube = Object::load_obj(Path::new("./objects/cube.obj")).unwrap();
    let mut engine = Engine::new((1024, 1024), 0, 0);
    engine
        .scene()
        .set_camera(Camera {
            center: Vector::new(0., 0., 5.),
            fov: Fov(60., 60.),
            width: 1024,
            ..Default::default()
        })
        .add_light(Light {
            transform: Transform::default(),
            variant: LightVariant::Point,
            color: Color::new(1., 1., 1.),
            brightness: 1.,
        })
        .add_object(Object::load_obj(Path::new("./objects/cube.obj")).unwrap());
    engine.render_to_path(Path::new("output.ppm")).unwrap();
}
