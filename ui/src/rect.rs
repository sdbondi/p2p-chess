use crate::{
    color::Color,
    drawable::{Drawable, FrameBuffer},
};

#[derive(Debug)]
pub struct Rect {
    frame: Frame,
    bg_color: Color,
    border_size: u32,
    border_color: Color,
}

impl Rect {
    pub fn new(frame: Frame, bg_color: Color) -> Self {
        Self {
            frame,
            bg_color,
            border_size: 0,
            border_color: Color::black(),
        }
    }

    pub fn set_bg_colour(&mut self, color: Color) -> &mut Self {
        self.bg_color = color;
        self
    }

    pub fn set_border(&mut self, size: u32, color: Color) -> &mut Self {
        self.border_size = size;
        self.border_color = color;
        self
    }

    pub fn is_in_boundary(&self, x: u32, y: u32) -> bool {
        self.frame.is_in_boundary(x, y)
    }

    pub fn x(&self) -> u32 {
        self.frame.x
    }

    pub fn y(&self) -> u32 {
        self.frame.y
    }

    pub fn w(&self) -> u32 {
        self.frame.w
    }

    pub fn h(&self) -> u32 {
        self.frame.h
    }
}

impl Drawable for Rect {
    fn draw(&mut self, buf: &mut FrameBuffer) {
        self.frame.fill(self.bg_color.to_rgba(), buf);
        if self.border_size > 0 {
            self.frame.draw_border(self.border_size, self.border_color, buf);
        }
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
    pub fn new(x: u32, y: u32, w: u32, h: u32) -> Self {
        Self { x, y, w, h }
    }

    pub fn is_in_boundary(&self, x: u32, y: u32) -> bool {
        x >= self.x && x <= self.x + self.w && y >= self.y && y <= self.y + self.h
    }
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
        let h = self.h as usize;
        for i in 0..=h {
            buf.put_line(self.x, self.y + i as u32, self.w, val);
        }
    }

    pub fn draw_border(&self, size: u32, color: Color, buf: &mut FrameBuffer) {
        let bottom = self.h.saturating_sub(size);
        let color = color.to_rgba();
        for i in 0..=self.h {
            if i < size || i > bottom {
                buf.put_line(self.x, self.y + i, self.w, color);
            } else {
                buf.put_line(self.x, self.y + i, size, color);
                buf.put_line(self.x + self.w - size, self.y + i, size, color);
            }
        }
    }

    pub fn scan<F>(&self, mut callback: F)
    where F: FnMut(u32, u32) -> bool {
        for y in self.y..=self.y + self.h {
            for x in self.x..=self.x + self.w {
                if !callback(x, y) {
                    return;
                }
            }
        }
    }
}
