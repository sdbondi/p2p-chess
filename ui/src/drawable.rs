use crate::color::Color;
use std::cmp;
use std::collections::Bound;
use std::ops::RangeBounds;

pub trait Drawable {
    fn draw(&mut self, buf: &mut FrameBuffer);
}

pub struct FrameBuffer {
    buf: Vec<u32>,
    width: u32,
    height: u32,
}

impl FrameBuffer {
    pub fn new(width: u32, height: u32, background_color: Color) -> Self {
        Self {
            buf: vec![background_color.to_rgba(); width as usize * height as usize],
            width,
            height,
        }
    }

    pub fn put_pixel(&mut self, x: u32, y: u32, val: u32) -> &mut Self {
        let pos = (x + y * self.width) as usize;
        if pos < self.buf.len() {
            self.buf[pos] = val;
        }
        self
    }

    pub fn put_line(&mut self, x: u32, y: u32, width: u32, val: u32) -> &mut Self {
        let offset_y = self.width * y;
        let start = (x + offset_y) as usize;
        let end = (x + offset_y + width) as usize;
        self.as_slice_mut(start..=end).fill(val);
        self
    }

    pub fn clear(&mut self, colour: Color) -> &mut Self {
        self.buf[..].fill(colour.to_rgba());
        self
    }

    fn as_slice_mut<R: RangeBounds<usize>>(&mut self, range: R) -> &mut [u32] {
        let start = match range.start_bound() {
            Bound::Included(x) => cmp::min(*x + 1, self.buf.len() - 1),
            Bound::Excluded(x) => cmp::min(*x, self.buf.len() - 1),
            Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            Bound::Included(x) => cmp::min(*x + 1, self.buf.len() - 1),
            Bound::Excluded(x) => cmp::min(*x, self.buf.len() - 1),
            Bound::Unbounded => self.buf.len() - 1,
        };
        &mut self.buf[start..end]
    }

    pub fn as_slice(&self) -> &[u32] {
        &self.buf
    }
}
