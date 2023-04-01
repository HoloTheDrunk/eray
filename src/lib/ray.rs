use super::vector::Vec3;

#[derive(Clone, Debug, Default)]
pub struct Ray {
    start: Vec3,
    dir: Vec3,
}

impl Ray {
    pub fn new(start: Vec3, dir: Vec3) -> Self {
        Self {
            start,
            dir: dir.normalize(),
        }
    }

    pub fn calc(&self, t: f32) -> Vec3 {
        self.start + self.dir * t
    }

    #[inline]
    pub fn start(&self) -> &Vec3 {
        &self.start
    }

    #[inline]
    pub fn dir(&self) -> &Vec3 {
        &self.dir
    }
}
