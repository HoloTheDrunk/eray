mod shaderlib;

use eray::{
    engine::Engine,
    node,
    prelude::*,
    shader::{
        self,
        graph::{SocketType, SocketValue},
    },
    ssref,
};
use map_macro::hash_map;

use std::path::Path;

fn main() -> std::io::Result<()> {
    let mut cube = Object::load_obj(Path::new("./objects/cube.obj")).unwrap();

    // cube.material = shaderlib::wave::material().unwrap();
    cube.material = material().unwrap();
    cube.material
        .set_input(&"width".into(), SocketValue::Value(Some(1024.)))
        .unwrap()
        .set_input(&"height".into(), SocketValue::Value(Some(1024.)))
        .unwrap()
        // Wave
        .set_input(&"x_fac".into(), SocketValue::Value(Some(1.)))
        .unwrap()
        .set_input(&"y_fac".into(), SocketValue::Value(Some(1.)))
        .unwrap()
        // Color
        .set_input(&"red".into(), SocketValue::Value(Some(1.)))
        .unwrap()
        .set_input(&"green".into(), SocketValue::Value(Some(0.)))
        .unwrap()
        .set_input(&"blue".into(), SocketValue::Value(Some(0.)))
        .unwrap()
        // Mixing
        .set_input(&"factor".into(), SocketValue::Value(Some(0.5)))
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
            brightness: 0.2,
        })
        .add_light(Light {
            transform: Transform::default().apply_translation(Vector::new(1., 1., 2.)),
            variant: LightVariant::Point,
            color: Color::new(1., 1., 1.),
            brightness: 1.,
        })
        .add_object(cube.build().unwrap());

    #[cfg(not(debug_assertions))]
    {
        engine.render_to_path(Path::new("output.ppm")).unwrap();
    }

    // shader::parsing::parse_shader("nodes/rgb_wave.eray", &mut HashMap::new()).unwrap();

    let slib: &Vec<_> = shaderlib::SHADERLIB.as_ref();
    dbg!(slib.len());

    Ok(())
}

fn material() -> Result<Material, eray::shader::graph::Error> {
    Ok(Material::from((
        eray::shader::graph::graph! {
            inputs:
                // Mandatory
                "width": SocketType::Value.into(),
                "height": SocketType::Value.into(),

                // Optional
                "x_fac": SocketValue::Value(Some(shaderlib::wave::DEFAULT_FACTOR)),
                "y_fac": SocketValue::Value(Some(shaderlib::wave::DEFAULT_FACTOR)),

                "red": SocketValue::Value(Some(1.)),
                "green": SocketValue::Value(Some(1.)),
                "blue": SocketValue::Value(Some(1.)),

                "factor": SocketValue::Value(Some(0.5)),
            nodes:
                "wave": {
                    let mut node = node!(import graph "wave" shaderlib::wave::graph()?);
                    node.set_input(&"width".into(), ssref!(graph "width"))?
                        .set_input(&"height".into(), ssref!(graph "height"))?
                        .set_input(&"x_fac".into(), ssref!(graph "x_fac"))?
                        .set_input(&"y_fac".into(), ssref!(graph "y_fac"))?;
                    node
                },
                "wave_to_color": {
                    let mut node = node!(import graph "rgb" shaderlib::rgb::graph()?);
                    node.set_input(&"width".into(), ssref!(graph "width"))?
                        .set_input(&"height".into(), ssref!(graph "height"))?
                        .set_input(&"red".into(), ssref!(node "wave" "value"))?
                        .set_input(&"green".into(), ssref!(node "wave" "value"))?
                        .set_input(&"blue".into(), ssref!(node "wave" "value"))?;
                    node
                },
                "flat_color": {
                    let mut node = node!(import graph "flat_color" shaderlib::flat_color::graph()?);
                    node.set_input(&"width".into(), ssref!(graph "width"))?
                        .set_input(&"height".into(), ssref!(graph "height"))?
                        .set_input(&"red".into(), ssref!(graph "red"))?
                        .set_input(&"green".into(), ssref!(graph "green"))?
                        .set_input(&"blue".into(), ssref!(graph "blue"))?;
                    node
                },
                "mixer": {
                    let mut node = node!(import graph "mixer" shaderlib::mix_color::graph()?);
                    node.set_input(&"width".into(), ssref!(graph "width"))?
                        .set_input(&"height".into(), ssref!(graph "height"))?
                        .set_input(&"left".into(), ssref!(node "wave_to_color" "color"))?
                        .set_input(&"right".into(), ssref!(node "flat_color" "color"))?
                        .set_input(&"factor".into(), ssref!(graph "factor"))?;
                    node
                },
            outputs:
                "color": (ssref!(node "mixer" "color"), SocketType::IColor.into()),
                "diffuse": (ssref!(node "wave" "value"), SocketType::IValue.into()),
        }
        .validate()
        .unwrap(),
        hash_map! {
            StandardMaterialOutput::Color => "color".into(),
            StandardMaterialOutput::Diffuse => "diffuse".into(),
        },
    )))
}
