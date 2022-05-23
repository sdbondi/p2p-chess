use std::cmp;

use minifb::{Key, KeyRepeat, MouseButton, MouseMode, Window};

use crate::{
    color::Color,
    drawable::{Drawable, FrameBuffer},
    letters::LETTERS,
    rect::{Frame, Rect},
};

#[derive(Debug)]
pub struct ListBox {
    values: Vec<String>,
    text_color: Color,
    rect: Rect,
    is_active: bool,
    selected: usize,
}

impl ListBox {
    pub fn new(dims: Frame) -> Self {
        Self {
            values: Vec::new(),
            text_color: Color::white(),
            rect: {
                let mut r = Rect::from_frame(dims, Color::dark_grey());
                r.set_border(2, Color::light_grey());
                r
            },
            is_active: false,
            selected: 0,
        }
    }

    pub fn selected_index(&self) -> Option<usize> {
        if self.values.is_empty() {
            return None;
        }
        Some(self.selected)
    }

    pub fn selected(&self) -> Option<&str> {
        self.values.get(self.selected).map(|s| &**s)
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
        self
    }

    pub fn update(&mut self, window: &Window) {
        if let Some((x, y)) = window.get_mouse_pos(MouseMode::Discard) {
            if window.get_mouse_down(MouseButton::Left) {
                let x = x.round() as u32;
                let y = y.round() as u32;
                let active = self.is_in_boundary(x, y);
                self.set_active(active);
                if active {
                    if let Some(rel_y) = y.checked_sub(self.rect.y()) {
                        if rel_y <= self.rect.h() {
                            let idx = (rel_y / 25) as usize;
                            if idx < self.values.len() {
                                self.selected = idx;
                            }
                        }
                    }
                }
            }
        }

        if self.is_active {
            self.collect_keystrokes(window);
        }
    }

    fn collect_keystrokes(&mut self, window: &Window) {
        if let Some(keys) = window.get_keys_pressed(KeyRepeat::No) {
            for key in keys {
                match key {
                    Key::Up => {
                        self.selected = self.selected.saturating_sub(1);
                    },
                    Key::Down => self.selected = cmp::min(self.selected + 1, self.values.len() - 1),
                    _ => {},
                }
            }
        }
    }

    pub fn set_values(&mut self, values: Vec<String>) -> &mut Self {
        self.values = values;
        self
    }

    fn draw_items(&self, buf: &mut FrameBuffer) {
        let num_items = self.rect.y() as usize / 40;
        for (i, s) in self.values.iter().take(num_items).enumerate() {
            if self.selected == i {
                Rect::new(
                    self.rect.x() + 1,
                    self.rect.y() + (i as u32) * 25,
                    self.rect.w() - 1,
                    22,
                    Color::light_grey(),
                )
                .draw(buf);
            }
            LETTERS.draw_string(
                s,
                self.rect.x() + 2,
                self.rect.y() + (i as u32) * 25,
                self.text_color,
                buf,
            );
        }
    }
}

impl Drawable for ListBox {
    fn draw(&mut self, buf: &mut FrameBuffer) {
        self.rect.draw(buf);
        self.draw_items(buf);
    }
}
