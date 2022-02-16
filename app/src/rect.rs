use crate::drawable::{Drawable, FrameBuffer};

pub struct Rect {
    frame: Frame,
    colour: Colour,
}
impl Rect {
    pub fn new(x: u32, y: u32, w: u32, h: u32, colour: Colour) -> Self {
        Self {
            frame: Frame { x, y, w, h },
            colour,
        }
    }
}

impl Drawable for Rect {
    fn draw(&self, buf: &mut FrameBuffer) {
        self.frame.fill(self.colour.to_rgb(), buf);
    }
}

pub struct Frame {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

impl Frame {
    pub fn fill(&self, val: u32, buf: &mut FrameBuffer) {
        let x = self.x as usize;
        let y = self.y as usize;
        let w = self.w as usize;
        let h = self.h as usize;
        for i in 0..h {
            buf.put_line(x, y + i, w, val);
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Colour {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Colour {
    pub const fn white() -> Self {
        Self {
            r: 0xff,
            g: 0xff,
            b: 0xff,
            a: 0xff,
        }
    }

    pub const fn black() -> Self {
        Self {
            r: 0x00,
            g: 0x00,
            b: 0x00,
            a: 0xff,
        }
    }

    pub const fn green() -> Self {
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
