//! Actual renderer leveraging the constructs defined in the eray library.

use crate::Building;

use super::prelude::*;

use std::path::Path;

use rand::prelude::*;

/// Render engine.
pub struct Engine<State> {
    image: Image<Color>,
    scene: Scene<State>,
    bounces: usize,
    anti_aliasing: usize,
}

impl Engine<Building> {
    /// Create an Engine with a [default](Default) [Scene] from the given parameters.
    pub fn new((width, height): (u32, u32), bounces: usize, anti_aliasing: usize) -> Self {
        Self {
            image: Image {
                width,
                height,
                pixels: vec![Color::new(0., 0., 0.); (width * height) as usize],
            },
            // scene: Scene::new(Camera {
            //     center: Vector::new(0., 0., 1.),
            //     fov: Fov(60., 60. * (width as f32 / height as f32)),
            //     width,
            //     ..Default::default()
            // }),
            scene: Default::default(),
            bounces,
            anti_aliasing,
        }
    }

    /// Get the [Scene] to add entities to it.
    pub fn scene(&mut self) -> &mut Scene<Building> {
        &mut self.scene
    }

    /// Render a frame to the inner 1-frame buffer.
    pub fn render(&mut self) -> &Image<Color> {
        let (width, height) = self.scene.camera.size();

        let mut rng = rand::thread_rng();

        let mut step = 0;
        for y in 0..height {
            let new_step = ((y as f32 / height as f32) * 100.) as u32 / 10;
            if new_step > step {
                step = new_step;
                println!("{}%", step * 10);
            }

            for x in 0..width {
                let mut average: Color = self.cast_ray_from_camera(x as f32, y as f32).sum();

                for _ in 0..self.anti_aliasing {
                    average += self
                        .cast_ray_from_camera(
                            x as f32 + rng.gen_range((-1.)..1.),
                            y as f32 + rng.gen_range((-1.)..1.),
                        )
                        .sum();
                }

                if self.anti_aliasing > 0 {
                    self.image
                        .set(x, y, (average / self.anti_aliasing as f32).clamp());
                } else {
                    self.image.set(x, y, average);
                }
            }
        }

        &self.image
    }

    /// Use [render](Self::render) to render a frame and save the result as a file to a given path,
    /// creating any missing directories on the way.
    pub fn render_to_path(&mut self, path: &Path) -> std::io::Result<&Image<Color>> {
        self.render();

        if let Some((false, parent)) = path
            .parent()
            .map(|parent| (!parent.exists() || parent.is_dir(), parent))
        {
            std::fs::create_dir_all(parent)?;
        }

        self.image.save_as_ppm(path);

        Ok(&self.image)
    }

    fn cast_ray_from_camera(&self, x: f32, y: f32) -> impl Iterator<Item = Color> {
        let (width, height) = self.scene.camera.size();

        let ray = self
            .scene
            .camera
            .pixel_to_ray(x / width as f32, y / height as f32);

        self.cast_ray(&ray, 0)
    }

    // fn cast_ray(&self, x: f32, y: f32, bounce_depth: usize) -> impl Iterator<Item = Color> {
    fn cast_ray(&self, ray: &Ray, bounce_depth: usize) -> impl Iterator<Item = Color> {
        let mut lighting: Vec<Color> = Vec::new();
        let mut closest: Option<f32> = None;

        for object in self.scene.objects.iter() {
            let Some(RaycastHit { face_index: _, position, normal, material }) = object.intersects(ray) else {continue;};

            // Ignore if further than closest encountered
            let dist_sq = (position - self.scene.camera.center).len_sq();
            if closest.is_none() || dist_sq < closest.unwrap() {
                closest = Some(dist_sq);
                lighting.clear();
            } else {
                continue;
            }

            let color: Color = material.color.unwrap_or_default();

            for light in self
                .scene
                .lights
                .iter()
                .filter(|light| light.variant != LightVariant::Ambient)
            {
                if self.reaches_light(
                    &Ray::new(
                        position + normal * 0.1,
                        light.transform.translation() - position,
                    ),
                    light,
                ) {
                    let mut prod = normal
                        .dot_product(&(light.transform.translation() - position))
                        .clamp(0., 1.);

                    if prod.is_nan() {
                        prod = 0.;
                    }

                    let falloff = 1. / (light.transform.translation() - position).len();

                    let diffusion = color
                        * light.color
                        * material.diffuse.unwrap_or(0.5)
                        * prod
                        * light.brightness
                        * falloff;

                    let specular_power = material.specular_power.unwrap_or(1.);
                    let specular = {
                        // w = v - 2 * (v x n) * n
                        let reflected = *ray.dir() - normal * 2. * (ray.dir().dot_product(&normal));
                        let res = (material.specular.unwrap_or(0.5)
                            * light.brightness
                            * reflected
                                .normalize()
                                .dot_product(
                                    &(light.transform.translation() - position).normalize(),
                                )
                                .powf(specular_power))
                        .clamp(0., 1.);
                        Color::new(res, res, res)
                    } * falloff.powf(specular_power).clamp(0., 1.);

                    let result = diffusion + specular;

                    lighting.push(result);
                }

                let reflection = material.reflection.unwrap_or(0.);
                if bounce_depth < self.bounces && reflection != 0. {
                    let start = position + normal * 0.1;
                    let dir = *ray.dir() - normal * 2. * (ray.dir().dot_product(&normal));
                    let ray = Ray::new(start, dir);

                    lighting.extend(
                        self.cast_ray(&ray, bounce_depth + 1)
                            .map(|color| color * reflection),
                    );
                }
            }

            // if let Some(ref ambient) = self.scene.ambient {
            //     lighting.push(ambient.color * props.diffusion * ambient.brightness);
            // }
            for ambient in self
                .scene
                .lights
                .iter()
                .filter(|light| light.variant == LightVariant::Ambient)
            {
                lighting.push(
                    ambient.color.min(&color)
                        * material.diffuse.unwrap_or(0.5)
                        * ambient.brightness,
                );
            }
        }

        if closest.is_none() {
            lighting.push(Color::new(0.1, 0.1, 0.2));
        }

        lighting.into_iter()
    }

    fn reaches_light(&self, ray: &Ray, light: &Light) -> bool {
        let dist = (light.transform.translation() - *ray.start()).len();

        for object in self.scene.objects.iter() {
            if let Some(intersection) = object.intersects(ray) {
                return (intersection.position - *ray.start()).len() > dist;
            }
        }

        true
    }
}
