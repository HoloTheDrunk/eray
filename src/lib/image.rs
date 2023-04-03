use std::{
    fs::OpenOptions,
    io::{BufWriter, Write},
    path::Path,
};

use super::color::Color;

#[derive(Clone, Debug, PartialEq)]
pub struct Image<T> {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<T>,
}

impl<T: Clone> Image<T> {
    pub fn init(width: u32, height: u32, value: T) -> Self {
        Self {
            width,
            height,
            pixels: vec![value; (width * height) as usize],
        }
    }

    pub fn mod_get(&self, x: u32, y: u32) -> T {
        self.pixels[((y * self.width) % self.height + x % self.width) as usize].clone()
    }

    pub fn set(&mut self, x: u32, y: u32, value: T) {
        self.pixels[(y * self.width + x) as usize] = value;
    }
}

impl Image<Color> {
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

#[cfg(test)]
mod test {
    use std::process::Stdio;

    use super::*;

    #[test]
    fn test_save() {
        let (width, height) = (1024, 1024);
        let mut image = Image {
            width,
            height,
            pixels: vec![Color::default(); (width * height) as usize],
        };

        for row in 0..height {
            for pixel in 0..width {
                image.pixels[((height - row - 1) * width + pixel) as usize] = Color {
                    r: (255 * pixel / width) as f32,
                    g: (255 * row / height) as f32,
                    b: {
                        let y = 0.1 * row as f32;
                        let x = 0.1 * pixel as f32;
                        let v = ((y.sin() + 1.) / 2. + (x.cos() + 1.) / 2.) / 2. * 255.;

                        let mask = v > 50.;

                        v.powi(50) * mask as u32 as f32
                    },
                };
            }
        }

        image.save_as_ppm(Path::new("test.ppm"));

        if let Err(err) = std::fs::create_dir("output") {
            if err.kind() != std::io::ErrorKind::AlreadyExists {
                assert!(false, "Error creating output directory");
            }
        }

        std::process::Command::new("ppmtojpeg")
            .arg("test.ppm")
            .stdout(Stdio::from(
                OpenOptions::new()
                    .create(true)
                    .write(true)
                    .open("output/test.jpeg")
                    .expect("Error creating jpeg"),
            ))
            .output()
            .expect("Error converting image to jpeg");
    }
}
