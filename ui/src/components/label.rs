use crate::{
    color::Color,
    drawable::{Drawable, FrameBuffer},
    letters::Letters,
    rect::Frame,
};

#[derive(Debug)]
pub struct Label {
    text: String,
    text_color: Color,
    dims: Frame,
    letters: Letters,
}

impl Label {
    pub fn new(dims: Frame, letters: Letters) -> Self {
        Self {
            text: String::new(),
            text_color: Color::white(),
            dims,
            letters,
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

    fn draw_text(&self, buf: &mut FrameBuffer) {
        let mid = (self.dims.h / 2) - 10;
        self.letters
            .draw_string(&self.text, self.dims.x + 2, self.dims.y + mid, self.text_color, buf);
    }
}

impl Drawable for Label {
    fn draw(&mut self, buf: &mut FrameBuffer) {
        self.draw_text(buf);
    }
}
