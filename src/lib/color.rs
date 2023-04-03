use std::{
    iter::Sum,
    ops::{Div, Mul},
};

use ::derive_more::{Add, AddAssign};

#[derive(Clone, Copy, Default, Debug, Add, AddAssign, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl Color {
    pub fn new(r: f32, g: f32, b: f32) -> Self {
        Color { r, g, b }
    }

    pub fn as_bytes(&self) -> [u8; 3] {
        [
            (self.r * 255.) as u8,
            (self.g * 255.) as u8,
            (self.b * 255.) as u8,
        ]
    }

    pub fn clamp(&self) -> Self {
        Self {
            r: self.r.clamp(0., 1.),
            g: self.g.clamp(0., 1.),
            b: self.b.clamp(0., 1.),
        }
    }
}

impl Mul<f64> for Color {
    type Output = Self;

    fn mul(mut self, rhs: f64) -> Self::Output {
        self.r = (self.r as f64 * rhs) as f32;
        self.g = (self.g as f64 * rhs) as f32;
        self.b = (self.b as f64 * rhs) as f32;

        self
    }
}

impl Mul for Color {
    type Output = Self;

    fn mul(mut self, rhs: Self) -> Self::Output {
        self.r *= rhs.r;
        self.g *= rhs.g;
        self.b *= rhs.b;

        self
    }
}

impl Sum<Color> for Color {
    fn sum<I: Iterator<Item = Color>>(iter: I) -> Self {
        iter.reduce(|acc, cur| acc + cur)
            .unwrap_or(Color::default())
    }
}

impl Div<f32> for Color {
    type Output = Self;
    fn div(mut self, rhs: f32) -> Self::Output {
        self.r /= rhs;
        self.g /= rhs;
        self.b /= rhs;

        self
    }
}
