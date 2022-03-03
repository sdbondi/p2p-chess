use std::{collections::HashMap, hash::Hash};

use crate::{
    color::Color,
    drawable::{Drawable, FrameBuffer},
    rect::Frame,
};

#[derive(Debug)]
pub struct SpriteSheet<K, I> {
    image: I,
    areas: HashMap<K, Frame>,
    ignore_color: Option<Color>,
}

impl<K: Hash + Eq, I: GetRgba> SpriteSheet<K, I> {
    pub fn new(image: I) -> Self {
        Self {
            image,
            areas: Default::default(),
            ignore_color: None,
        }
    }

    pub fn ignore_color(&mut self, colour: Color) -> &mut Self {
        self.ignore_color = Some(colour);
        self
    }

    pub fn add_area(&mut self, name: K, area: Frame) -> &mut Self {
        self.areas.insert(name, area);
        self
    }

    pub fn get_sprite_drawable(&self, name: &K, x: u32, y: u32) -> Option<DrawableSprite<'_, I>> {
        self.areas.get(name).map(|area| DrawableSprite::<'_, I> {
            image: &self.image,
            area,
            x,
            y,
            ignore_color: self.ignore_color,
            subst_color: None,
        })
    }
}

pub struct DrawableSprite<'a, I> {
    image: &'a I,
    area: &'a Frame,
    x: u32,
    y: u32,
    ignore_color: Option<Color>,
    subst_color: Option<(Color, Color)>,
}

impl<'a, I> DrawableSprite<'a, I> {
    pub fn with_substitute_color(&mut self, from: Color, to: Color) -> &mut Self {
        self.subst_color = Some((from, to));
        self
    }
}

impl<I: GetRgba> Drawable for DrawableSprite<'_, I> {
    fn draw(&mut self, buf: &mut FrameBuffer) {
        let mut offset_x = 0;
        let mut offset_y = 0;
        self.area.scan(|x, y| {
            let mut px = self.image.get_rgba(x as usize, y as usize);

            offset_x += 1;
            if offset_x % (self.area.w + 1) == 0 {
                offset_x = 0;
                offset_y += 1;
            }

            if let Some(c) = self.ignore_color {
                if px == c.to_rgba() {
                    // Dont draw any pixels if the mask colour is encountered
                    return true;
                }
            }
            if let Some((from, to)) = self.subst_color {
                if px == from.to_rgba() {
                    px = to.to_rgba();
                }
            }
            buf.put_pixel(offset_x + self.x, offset_y + self.y, px);

            true
        });
    }
}

pub trait GetRgba {
    fn get_rgba(&self, x: usize, y: usize) -> u32;
}
