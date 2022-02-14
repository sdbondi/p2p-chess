use std::cmp;
use std::collections::Bound;
use std::ops::RangeBounds;

pub trait Drawable {
    fn draw(&self, buf: &mut FrameBuffer);
}

pub struct FrameBuffer {
    buf: Vec<u32>,
    width: usize,
    height: usize,
}

impl FrameBuffer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            buf: vec![0; width * height],
            width,
            height,
        }
    }

    pub fn put_pixel(&mut self, x: usize, y: usize, val: u32) -> &mut Self {
        self.buf[x + y * self.width] = val;
        self
    }

    pub fn put_line(&mut self, x: usize, y: usize, width: usize, val: u32) -> &mut Self {
        let offset_y = self.width * y;
        self.get_slice_mut((x + offset_y)..offset_y + x + width)
            .fill(val);
        self
    }

    pub fn clear(&mut self, val: u32) -> &mut Self {
        self.buf[..].fill(val);
        self
    }

    fn get_slice_mut<R: RangeBounds<usize>>(&mut self, range: R) -> &mut [u32] {
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
