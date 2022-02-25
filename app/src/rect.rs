use crate::drawable::{Drawable, FrameBuffer};
use crate::Colour;

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
        self.frame.fill(self.colour.to_rgba(), buf);
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Frame {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

impl Frame {
    pub fn offset_xy(&self, x: u32, y: u32) -> Self {
        Self {
            x: self.x + x,
            y: self.y + y,
            ..*self
        }
    }
}

impl Frame {
    pub fn fill(&self, val: u32, buf: &mut FrameBuffer) {
        let x = self.x as usize;
        let y = self.y as usize;
        let w = self.w as usize;
        let h = self.h as usize;
        for i in 0..=h {
            buf.put_line(x, y + i, w, val);
        }
    }

    pub fn scan<F>(&self, mut callback: F)
    where
        F: FnMut(u32, u32) -> bool,
    {
        for y in self.y..=self.y + self.h {
            for x in self.x..=self.x + self.w {
                if !callback(x, y) {
                    return;
                }
            }
        }
    }
}
