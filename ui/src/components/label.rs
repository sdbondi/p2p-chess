use crate::{
    color::Color,
    drawable::{Drawable, FrameBuffer},
    letters::LETTERS,
    rect::{Frame, Rect},
};

#[derive(Debug)]
pub struct Label {
    text: String,
    text_color: Color,
    bg_color: Option<Color>,
    dims: Frame,
}

impl Label {
    pub fn new(dims: Frame) -> Self {
        Self {
            text: String::new(),
            text_color: Color::white(),
            bg_color: None,
            dims,
        }
    }

    pub fn set_text<T: Into<String>>(&mut self, text: T) -> &mut Self {
        self.text = text.into();
        self
    }

    pub fn set_text_color(&mut self, color: Color) -> &mut Self {
        self.text_color = color;
        self
    }

    pub fn set_bg_color(&mut self, color: Color) -> &mut Self {
        self.bg_color = Some(color);
        self
    }

    fn draw_text(&self, buf: &mut FrameBuffer) {
        let mid = (self.dims.h / 2) - 10;
        LETTERS.draw_string(&self.text, self.dims.x + 2, self.dims.y + mid, self.text_color, buf);
    }
}

impl Drawable for Label {
    fn draw(&mut self, buf: &mut FrameBuffer) {
        if let Some(c) = self.bg_color {
            Rect::from_frame(self.dims, c).draw(buf);
        }
        self.draw_text(buf);
    }
}
