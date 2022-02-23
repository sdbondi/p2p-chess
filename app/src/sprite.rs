use crate::bitmap::Bitmap;
use crate::{Colour, Drawable, Frame, FrameBuffer};
use std::collections::HashMap;
use std::hash::Hash;

pub struct SpriteSheet<K, I> {
    image: I,
    areas: HashMap<K, Frame>,
    ignore_colour: Option<Colour>,
}

impl<K: Hash + Eq, I: GetRgba> SpriteSheet<K, I> {
    pub fn new(image: I) -> Self {
        Self {
            image,
            areas: Default::default(),
            ignore_colour: None,
        }
    }

    pub fn ignore_colour(&mut self, colour: Colour) -> &mut Self {
        self.ignore_colour = Some(colour);
        self
    }

    pub fn add_area(&mut self, name: K, area: Frame) -> &mut Self {
        self.areas.insert(name, area);
        self
    }

    pub fn get_sprite(&self, name: &K, x: u32, y: u32) -> Option<DrawableSprite<'_, I>> {
        self.areas.get(name).map(|area| DrawableSprite::<'_, I> {
            image: &self.image,
            area,
            x,
            y,
            ignore_colour: self.ignore_colour,
        })
    }
}

pub struct DrawableSprite<'a, I> {
    image: &'a I,
    area: &'a Frame,
    x: u32,
    y: u32,
    ignore_colour: Option<Colour>,
}

impl<I: GetRgba> Drawable for DrawableSprite<'_, I> {
    fn draw(&self, buf: &mut FrameBuffer) {
        let mut offset_x = 0;
        let mut offset_y = 0;
        self.area.scan(|x, y| {
            let px = self.image.get_rgba(x as usize, y as usize); // as u32;

            offset_x += 1;
            if offset_x % (self.area.w as usize + 1) == 0 {
                offset_x = 0;
                offset_y += 1;
            }

            if let Some(c) = self.ignore_colour {
                if px == c.to_rgba() {
                    // Dont draw any pixels if the mask colour is encountered
                    return true;
                }
            }
            buf.put_pixel(
                offset_x + self.x as usize,
                offset_y + self.y as usize,
                px as u32,
            );
            true
        })
    }
}

pub trait GetRgba {
    fn get_rgba(&self, x: usize, y: usize) -> u32;
}
