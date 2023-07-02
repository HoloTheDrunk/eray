//! Basic image implementation with saving

use std::{
    fs::OpenOptions,
    io::{BufWriter, Write},
    path::Path,
};

use crate::vector::Vector;

use super::color::Color;

#[derive(Clone, Debug, PartialEq)]
/// Generic image struct. Usage of the word `pixel` in this documentation refers to an instance of
/// the stored data type.
pub struct Image<T> {
    /// Width of the image in pixels
    pub width: u32,
    /// Height of the image in pixels
    pub height: u32,
    /// Vector containing the image's pixels
    pub pixels: Vec<T>,
}

impl<T: Clone> Image<T> {
    /// Create an image from a pixel width and height and a default value
    pub fn new(width: u32, height: u32, value: T) -> Self {
        Self {
            width,
            height,
            pixels: vec![value; (width * height) as usize],
        }
    }

    /// Get a pixel at x/y coordinates, with width/height modulos applied to the respective coordinates for easy tiling.
    pub fn mod_get(&self, x: u32, y: u32) -> T {
        self.pixels[(((y % self.height) * self.width) + x % self.width) as usize].clone()
    }

    /// Set a pixel at x/y coordinates
    pub fn set(&mut self, x: u32, y: u32, value: T) {
        self.pixels[(y * self.width + x) as usize] = value;
    }
}

impl Image<Color> {
    /// Save current state as a .ppm according to the path given as argument
    pub fn save_as_ppm(&self, path: &Path) {
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)
            .expect("Error saving image");

        file.write_all(format!("P6 {} {} {}\n", self.width, self.height, u8::MAX).as_bytes())
            .expect("Error writing image headers");

        let mut writer = BufWriter::new(file);

        for row in self
            .pixels
            .windows(self.width as usize)
            .step_by(self.width as usize)
            .rev()
            .map(|row| row.iter().map(Color::as_bytes))
        {
            for rgb in row {
                writer.write_all(&rgb).expect("Error writing pixel value");
            }
        }

        writer.flush().unwrap();
    }
}

/// Allows for easy conversion between different image types.
pub trait Convertible<Target, Source: Into<Target>> {
    /// Convert image type if the underlying pixel type can be converted.
    fn convert_image(self, method: fn(Source) -> Target) -> Image<Target>;
}

impl<Target, Source: Into<Target>> Convertible<Target, Source> for Image<Source> {
    fn convert_image(self, method: fn(Source) -> Target) -> Image<Target> {
        let Self {
            width,
            height,
            pixels,
        } = self;

        Image::<Target> {
            width,
            height,
            pixels: pixels.into_iter().map(method).collect(),
        }
    }
}

// impl<const SDIM: usize, const DDIM: usize, TYPE> Convertible<Vector<DDIM, TYPE>>
//     for Image<Vector<SDIM, TYPE>>
// {
//     fn convert_image(self) -> Image<Vector<DDIM, TYPE>> {
//         let Self {
//             width,
//             height,
//             pixels,
//         } = self;
//
//         Image::<Target> {
//             width,
//             height,
//             pixels: pixels.into_iter().map(Vector::resize).collect(),
//         }
//     }
// }

impl<IC: Copy + Into<f32>> From<Image<Vector<3, IC>>> for Image<Color> {
    fn from(
        Image::<Vector<3, IC>> {
            width,
            height,
            pixels,
        }: Image<Vector<3, IC>>,
    ) -> Self {
        Self {
            width,
            height,
            pixels: pixels.into_iter().map(Color::from).collect(),
        }
    }
}

impl From<Image<f32>> for Image<Color> {
    fn from(value: Image<f32>) -> Self {
        Self {
            width: value.width,
            height: value.height,
            pixels: value.pixels.into_iter().map(Color::from).collect(),
        }
    }
}

impl<T: Default> Default for Image<T> {
    fn default() -> Self {
        Self {
            width: 1,
            height: 1,
            pixels: vec![T::default()],
        }
    }
}

#[cfg(test)]
mod test {
    use std::process::Stdio;

    use super::*;

    #[test]
    fn save_image_as_ppm() {
        let (width, height) = (1024, 1024);
        let mut image = Image {
            width,
            height,
            pixels: vec![Color::default(); (width * height) as usize],
        };

        for row in 0..height {
            for pixel in 0..width {
                image.pixels[((height - row - 1) * width + pixel) as usize] = Color {
                    r: pixel as f32 / width as f32,
                    g: 1. - row as f32 / height as f32,
                    b: 0.,
                };
            }
        }

        if let Err(err) = std::fs::create_dir("tests") {
            if err.kind() != std::io::ErrorKind::AlreadyExists {
                assert!(false, "Error creating output directory");
            }
        }

        image.save_as_ppm(Path::new("tests/test.ppm"));

        if which::which("ppmtojpeg").is_ok() {
            std::process::Command::new("ppmtojpeg")
                .arg("test.ppm")
                .stdout(Stdio::from(
                    OpenOptions::new()
                        .create(true)
                        .write(true)
                        .open("tests/test.jpeg")
                        .expect("Error creating jpeg"),
                ))
                .output()
                .expect("Error converting image to jpeg");
        }
    }

    #[test]
    fn mod_get() {
        let mut image = Image::new(10, 10, 0);

        for (index, pixel) in image.pixels.iter_mut().enumerate() {
            *pixel = index;
        }

        assert_eq!(
            image.mod_get(123, 12),
            image.pixels[(2 * image.width + 3) as usize]
        );
    }
}
