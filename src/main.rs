mod shaderlib;

use eray::{
    engine::*,
    prelude::*,
    shader::{self, graph::SocketValue},
};

use std::path::Path;

fn main() {
    println!("Hello, world!");

    let mut cube = Object::load_obj(Path::new("./objects/cube.obj")).unwrap();

    cube.material = shaderlib::wave::material().unwrap();
    cube.material
        .set_input(&"width".into(), SocketValue::Number(Some(1024.)))
        .unwrap()
        .set_input(&"height".into(), SocketValue::Number(Some(1024.)))
        .unwrap()
        .set_input(&"x_fac".into(), SocketValue::Number(Some(1.)))
        .unwrap()
        .set_input(&"y_fac".into(), SocketValue::Number(Some(1.)))
        .unwrap();
    cube.material.update().unwrap();

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
            transform: Transform::default().apply_translation(Vector::new(0., 2., 0.)),
            variant: LightVariant::Ambient,
            color: Color::new(1., 1., 1.),
            brightness: 1.,
        })
        .add_light(Light {
            transform: Transform::default().apply_translation(Vector::new(0., 2., 0.)),
            variant: LightVariant::Point,
            color: Color::new(1., 1., 1.),
            brightness: 1.,
        })
        .add_object(cube.build().unwrap());

    // engine.render_to_path(Path::new("output.ppm")).unwrap();
}
