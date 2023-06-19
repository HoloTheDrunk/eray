//! Collection of objects representing a scene to be rendered.

use crate::{camera::Camera, light::Light, object::Object, Building, Built};

use std::fmt::Debug;

#[derive(Default)]
/// Scene representation with objects, lights and a camera.
pub struct Scene<State> {
    state: std::marker::PhantomData<State>,
    /// Objects currently in the scene with a bool indicating visibility.
    pub objects: Vec<Object<Built>>,
    /// Lights currently in the scene.
    pub lights: Vec<Light>,
    /// Scene camera.
    pub camera: Camera,
}

impl<State> Debug for Scene<State> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Scene")
            .field("objects", &self.objects.len())
            .field("lights", &self.lights.len())
            .field("camera", &self.camera)
            .finish()
    }
}

impl Scene<Building> {
    pub fn new(camera: Camera) -> Self {
        Self {
            camera,
            ..Default::default()
        }
    }

    /// Adss an object to the scene.
    pub fn add_object(&mut self, object: Object<Built>) -> &mut Self {
        self.objects.push(object);
        self
    }

    /// Adss a light to the scene.
    pub fn add_light(&mut self, light: Light) -> &mut Self {
        self.lights.push(light);
        self
    }

    /// Sets the scene camera to the one passed as argument.
    pub fn set_camera(&mut self, camera: Camera) -> &mut Self {
        self.camera = camera;
        self
    }
}
