use crate::letters::Letters;
use crate::{Bitmap, Color, Drawable, FrameBuffer, Rect, SpriteSheet};
use minifb::{MouseButton, MouseMode, Window};
use std::rc::Rc;

pub struct TextBox {
    label: String,
    value: String,
    rect: Rect,
    is_active: bool,
    letters: Letters,
}

impl TextBox {
    pub fn new(letters: Letters) -> Self {
        Self {
            label: "ABCDEFGHIJKLMNOPQRSTUVWXYZ".to_string(),
            value: String::new(),
            rect: {
                let mut r = Rect::new(100, 100, 500, 40, Color::dark_grey());
                r.set_border(2, Color::light_grey());
                r
            },
            is_active: false,
            letters,
        }
    }

    pub fn is_in_boundary(&self, x: u32, y: u32) -> bool {
        self.rect.is_in_boundary(x, y)
    }

    pub fn set_bg_color(&mut self, color: Color) -> &mut Self {
        self.rect.set_bg_colour(color);
        self
    }

    pub fn set_active(&mut self, active: bool) -> &mut Self {
        self.is_active = active;
        if active {
            self.rect.set_bg_colour(Color::grey(0x60));
        } else {
            self.rect.set_bg_colour(Color::dark_grey());
        }
        self
    }

    pub fn update(&mut self, window: &Window) {
        if let Some((x, y)) = window.get_mouse_pos(MouseMode::Discard) {
            let active = self.is_active
                || (self.is_in_boundary(x.round() as u32, y.round() as u32)
                    && window.get_mouse_down(MouseButton::Left));

            self.set_active(active);
        }
    }

    pub fn draw_label(&self, buf: &mut FrameBuffer) {
        let mid = (self.rect.h() / 2) - 10;
        self.letters
            .draw_string(&self.label, self.rect.x() + 2, self.rect.y() + mid, buf);
    }
}

impl Drawable for TextBox {
    fn draw(&mut self, buf: &mut FrameBuffer) {
        self.rect.draw(buf);
        self.draw_label(buf);
    }
}
