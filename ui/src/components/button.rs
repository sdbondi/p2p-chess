use crate::color::Color;
use crate::drawable::{Drawable, FrameBuffer};
use crate::letters::Letters;
use crate::rect::Rect;
use minifb::{MouseButton, MouseMode, Window};

#[derive(Debug)]
pub struct Button {
    rect: Rect,
    text: String,
    letters: Letters,
    is_disabled: bool,
    click: Option<()>,
}

impl Button {
    pub fn new(rect: Rect, letters: Letters) -> Self {
        Self {
            rect,
            text: "Button".to_string(),
            is_disabled: false,
            letters,
            click: None,
        }
    }

    pub fn set_text<T: Into<String>>(&mut self, text: T) -> &mut Self {
        self.text = text.into();
        self
    }

    pub fn set_disabled(&mut self, disabled: bool) -> &mut Self {
        self.is_disabled = disabled;
        self
    }

    pub fn was_clicked(&mut self) -> bool {
        self.click.take().is_some()
    }

    pub fn update(&mut self, window: &Window) {
        if self.is_disabled {
            return;
        }

        if window.get_mouse_down(MouseButton::Left) {
            if let Some((x, y)) = window.get_mouse_pos(MouseMode::Discard) {
                if self.rect.is_in_boundary(x.round() as u32, y.round() as u32) {
                    self.click = Some(());
                }
            }
        }
    }

    fn draw_text(&self, buf: &mut FrameBuffer) {
        let half_text_w = self.text.len() as u32 * 11 / 2;
        let x = self.rect.x() + (self.rect.w() / 2) - half_text_w;
        let y = self.rect.y() + (self.rect.h() / 2) - 8;
        self.letters
            .draw_string(&self.text, x, y, Color::light_grey(), buf);
    }
}

impl Drawable for Button {
    fn draw(&mut self, buf: &mut FrameBuffer) {
        self.rect.draw(buf);
        self.draw_text(buf);
    }
}
