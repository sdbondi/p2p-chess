use minifb::{Key, KeyRepeat, MouseButton, MouseMode, Window};

use crate::{
    clipboard::Clipboard,
    color::Color,
    drawable::{Drawable, FrameBuffer},
    letters::LETTERS,
    rect::{Frame, Rect},
};

#[derive(Debug)]
pub struct TextBox {
    value: String,
    text_color: Color,
    rect: Rect,
    is_active: bool,
    clipboard: Clipboard,
}

impl TextBox {
    pub fn new(dims: Frame, clipboard: Clipboard) -> Self {
        Self {
            value: String::new(),
            text_color: Color::white(),
            rect: {
                let mut r = Rect::from_frame(dims, Color::dark_grey());
                r.set_border(2, Color::light_grey());
                r
            },
            is_active: false,
            clipboard,
        }
    }

    pub fn value(&self) -> &str {
        &self.value
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
            if window.get_mouse_down(MouseButton::Left) {
                let active = self.is_in_boundary(x.round() as u32, y.round() as u32);
                self.set_active(active);
            }
        }

        if self.is_active {
            self.collect_keystrokes(window);
        }
    }

    fn collect_keystrokes(&mut self, window: &Window) {
        if let Some(keys) = window.get_keys_pressed(KeyRepeat::No) {
            let modifiers = window.get_keys();
            let mut is_ctrl = false;
            let mut is_shift = false;
            let mut char = None;
            for key in modifiers.into_iter().flatten() {
                match key {
                    #[cfg(any(target_os = "windows", target_os = "linux"))]
                    Key::LeftCtrl | Key::RightCtrl => is_ctrl = true,
                    #[cfg(target_os = "macos")]
                    Key::LeftSuper | Key::RightSuper => is_ctrl = true,
                    Key::LeftShift | Key::RightShift => is_shift = true,
                    Key::Backspace => {
                        self.value.pop();
                        continue;
                    },
                    _ => {},
                }
            }
            for key in keys {
                char = key_to_char(key);
                if char.is_some() {
                    break;
                }
            }
            if is_ctrl && char.map(|c| c == 'v').unwrap_or(false) {
                if let Some(s) = self.paste_clipboard() {
                    self.value += &s;
                }
            } else if let Some(mut ch) = char {
                if is_shift {
                    ch = ch.to_ascii_uppercase();
                }
                self.value.push(ch);
            }
        }
    }

    fn paste_clipboard(&self) -> Option<String> {
        self.clipboard.get_contents().ok()
    }

    pub fn set_value(&mut self, value: String) -> &mut Self {
        self.value = value;
        self
    }

    fn draw_text(&self, buf: &mut FrameBuffer) {
        let mid = (self.rect.h() / 2) - 10;
        LETTERS.draw_string(
            &self.value,
            self.rect.x() + 2,
            self.rect.y() + mid,
            self.text_color,
            buf,
        );
    }
}

impl Drawable for TextBox {
    fn draw(&mut self, buf: &mut FrameBuffer) {
        self.rect.draw(buf);
        self.draw_text(buf);
    }
}

fn key_to_char(key: Key) -> Option<char> {
    if (Key::A as u32..=Key::Z as u32).contains(&(key as u32)) {
        return Some(char::from_u32('a' as u32 - Key::A as u32 + key as u32).unwrap());
    }
    if (Key::Key0 as u32..=Key::Key9 as u32).contains(&(key as u32)) {
        return Some(char::from_u32('0' as u32 - Key::Key0 as u32 + key as u32).unwrap());
    }
    if key == Key::Space {
        return Some(' ');
    }
    None
}
