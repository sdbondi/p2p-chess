use std::sync::Arc;

use once_cell::sync::Lazy;

use crate::{
    bitmap::Bitmap,
    color::Color,
    drawable::{Drawable, FrameBuffer},
    rect::Frame,
    sprite::SpriteSheet,
};

pub static LETTERS: Lazy<Letters> = Lazy::new(|| Letters::new());

#[derive(Debug, Clone)]
pub struct Letters {
    sprite_sheet: Arc<SpriteSheet<char, Bitmap>>,
}

impl Letters {
    pub fn new() -> Self {
        Self {
            sprite_sheet: Arc::new(init_letters_sprite()),
        }
    }

    pub fn draw_string(&self, s: &str, mut x: u32, y: u32, color: Color, buf: &mut FrameBuffer) {
        const LEADING: u32 = 11;
        for ch in s.chars() {
            if ch == ' ' {
                x += LEADING;
                continue;
            }
            match self.sprite_sheet.get_sprite_drawable(&ch, x, y) {
                Some(mut drawable) => drawable.with_substitute_color(Color::black(), color).draw(buf),
                None => {
                    self.sprite_sheet
                        .get_sprite_drawable(&'?', x, y)
                        .unwrap()
                        .with_substitute_color(Color::black(), color)
                        .draw(buf);
                },
            }
            x += LEADING;
        }
    }
}

fn init_letters_sprite() -> SpriteSheet<char, Bitmap> {
    let image = Bitmap::from_reader(&mut include_bytes!("../assets/letters.bmp").as_slice()).unwrap();
    let mut sprite_sheet = SpriteSheet::new(image);
    let letters = Frame {
        x: 0,
        y: 0,
        w: 15,
        h: 20,
    };
    // sprite_sheet.ignore_colour(Color::white());
    // TODO: This comes out of bitmap, not sure why
    sprite_sheet.ignore_color(Color::from_rgba(4294836220));
    for (i, ch) in ('A'..'Z').enumerate() {
        sprite_sheet.add_area(ch, letters.offset_xy(i as u32 * 15, 0));
    }
    for (i, ch) in ('a'..'z').enumerate() {
        sprite_sheet.add_area(ch, letters.offset_xy(i as u32 * 15, 20));
    }
    for (i, ch) in ('0'..='9').enumerate() {
        sprite_sheet.add_area(ch, letters.offset_xy(i as u32 * 15, 40));
    }
    sprite_sheet.add_area('?', letters.offset_xy(165, 40));
    sprite_sheet.add_area('!', letters.offset_xy(180, 40));
    sprite_sheet.add_area(':', letters.offset_xy(195, 40));
    sprite_sheet.add_area('-', letters.offset_xy(210, 40));
    sprite_sheet.add_area('#', letters.offset_xy(225, 40));

    sprite_sheet
}
