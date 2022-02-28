use crate::color::Color;
use crate::sprite::GetRgba;
use bmp::Pixel;
use std::io::Read;

#[derive(Debug)]
pub struct Bitmap {
    image: bmp::Image,
}

impl Bitmap {
    pub fn from_reader<R: Read>(reader: &mut R) -> anyhow::Result<Self> {
        let image = bmp::from_reader(reader)?;
        Ok(Self { image })
    }
}

impl GetRgba for Bitmap {
    fn get_rgba(&self, x: usize, y: usize) -> u32 {
        to_rgb(self.image.get_pixel(x as u32, y as u32))
    }
}

fn to_rgb(pixel: Pixel) -> u32 {
    Color::new(pixel.r, pixel.g, pixel.b, 0xff).to_rgba()
}
