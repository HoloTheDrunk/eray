use std::{collections::HashMap, path::Path};

use eray::{
    engine::*,
    get_sv, node,
    prelude::*,
    shader::{
        self,
        graph::{Graph, SocketType, SocketValue},
        shader::Side,
    },
    ssref,
};

use map_macro::hash_map;

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

    let mut cube = Object::load_obj(Path::new("./objects/cube.obj")).unwrap();
    cube.material = Material::from((
        shader::graph::graph! {
            inputs:
                "color": SocketValue::Color(Some(Image::new(1024, 1024, Color::new(1., 1., 1.)))),
            nodes:
                "wave": node! {
                    inputs:
                        "in": (ssref!(graph "color"), SocketType::Color.into()),
                    outputs:
                        "color": SocketType::Color.into();
                    |inputs, outputs| {
                        get_sv!( input | inputs  . "in": Color > color);
                        get_sv!(output | outputs . "color": Color > out);

                        if let Some(color) = color {
                            let out = out.get_or_insert(Image::new(color.width, color.height, Color::default()));

                            for y in 0..(out.height) {
                                for x in 0..(out.width) {
                                    out.pixels[(y * out.width + x) as usize] = color.mod_get(x, y) * ((x + y) as f32 / 10.).cos().abs();
                                }
                            }
                        } else {
                            return Err(crate::shader::shader::Error::Missing(Side::Input, "color".into()));
                        }

                        Ok(())
                    }
                },
            outputs:
                "color": (ssref!(node "wave" "color"), SocketType::Color.into()),
        }
        .validate()
        .unwrap(),
        hash_map! {
            StandardMaterialOutput::Color => "color".into()
        },
    ));

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
    engine.render_to_path(Path::new("output.ppm")).unwrap();
}
