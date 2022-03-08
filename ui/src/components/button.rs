use std::fmt::{Debug, Formatter};

use minifb::{MouseButton, MouseMode, Window};

use crate::{
    color::Color,
    components::handler::ClickHandler,
    drawable::{Drawable, FrameBuffer},
    letters::LETTERS,
    rect::Rect,
};

pub struct Button {
    rect: Rect,
    text: String,
    is_disabled: bool,
    click: Option<()>,
    on_click: Option<Box<dyn ClickHandler>>,
}

impl Button {
    pub fn new(rect: Rect) -> Self {
        Self {
            rect,
            text: "Button".to_string(),
            is_disabled: false,
            click: None,
            on_click: None,
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

    pub fn on_click<F: ClickHandler + 'static>(&mut self, handler: F) -> &mut Self {
        self.on_click = Some(Box::new(handler));
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
                    if let Some(ref mut handler) = self.on_click {
                        handler.handle_click();
                    }
                }
            }
        }
    }

    fn draw_text(&self, buf: &mut FrameBuffer) {
        let half_text_w = self.text.len() as u32 * 11 / 2;
        let x = self.rect.x() + (self.rect.w() / 2) - half_text_w;
        let y = self.rect.y() + (self.rect.h() / 2) - 8;
        LETTERS.draw_string(&self.text, x, y, Color::black(), buf);
    }
}

impl Drawable for Button {
    fn draw(&mut self, buf: &mut FrameBuffer) {
        self.rect.draw(buf);
        self.draw_text(buf);
    }
}

impl Debug for Button {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Button")
            .field("rect", &format!("{:?}", self.rect))
            .field("text", &format!("{:?}", self.text))
            .field("is_disabled", &format!("{:?}", self.is_disabled))
            .field("on_click", &"...")
            .finish()
    }
}
