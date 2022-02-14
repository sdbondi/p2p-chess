use crate::drawable::{Drawable, FrameBuffer};

pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
    pub colour: Colour,
}

impl Drawable for Rect {
    fn draw(&self, buf: &mut FrameBuffer) {
        let x = self.x as usize;
        let y = self.y as usize;
        let colour = self.colour.to_rgb();
        let w = self.w as usize;
        let h = self.h as usize;
        for i in 0..h {
            buf.put_line(x, y + i, w, colour);
        }
    }
}

pub struct Colour {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Colour {
    pub fn white() -> Self {
        Self {
            r: 0xff,
            g: 0xff,
            b: 0xff,
            a: 0xff,
        }
    }

    pub fn black() -> Self {
        Self {
            r: 0x00,
            g: 0x00,
            b: 0x00,
            a: 0xff,
        }
    }

    pub fn green() -> Self {
        Self {
            r: 0x00,
            g: 0xff,
            b: 0x00,
            a: 0xff,
        }
    }

    pub fn to_rgb(&self) -> u32 {
        let r = self.r as u32;
        let g = self.g as u32;
        let b = self.b as u32;

        b + (g * 0x100) + (r * 0x10000)
    }
}
