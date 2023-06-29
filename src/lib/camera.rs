//! Definition of the camera and its auxilliary data structures.

use crate::{raycasting::Ray, vector::Vector};

#[derive(Clone, Debug)]
/// Field of view as angles in radians.
pub struct Fov(pub f32, pub f32);

impl Fov {
    fn ratio(&self) -> f32 {
        self.0 / self.1
    }
}

#[derive(Clone, Debug)]
/// A 3D camera.
pub struct Camera {
    /// Position.
    pub center: Vector,
    /// Look (-Z) direction.
    pub target: Vector,
    /// Local up (+Y) direction.
    pub up: Vector,

    /// Field of view.
    pub fov: Fov,
    /// Number of pixels making width-wise.
    pub width: u32,

    /// Clipping plane.
    pub z_dist: f32,
}

impl Camera {
    /// Get viewport size in pixels.
    pub fn size(&self) -> (u32, u32) {
        (self.width, (self.width as f32 / self.fov.ratio()) as u32)
    }
}

impl Default for Camera {
    fn default() -> Self {
        Camera {
            center: Default::default(),
            target: Vector::new(0., 0., -1.),
            up: Vector::new(0., 1., 0.),
            fov: Fov(60., 60.),
            width: 1024,
            z_dist: 1.,
        }
    }
}

impl Camera {
    /// Convert discrete 2D pixel coordinates to a ray from the camera position toward the center
    /// of the desired pixel.
    pub fn pixel_to_ray(&self, x: f32, y: f32) -> Ray {
        // Image
        let aspect_ratio = self.fov.ratio();

        // Camera
        let viewport_height = 2.;
        let viewport_width = aspect_ratio * viewport_height;
        let focal_length = self.z_dist;

        let horizontal = Vector::new(viewport_width, 0., 0.);
        let vertical = Vector::new(0., viewport_height, 0.);

        let botleft =
            self.center - horizontal / 2. - vertical / 2. - Vector::new(0., 0., focal_length);

        Ray::new(
            self.center,
            botleft + horizontal * x + vertical * y - self.center,
        )
    }
}
