use crate::{Bitmap, Color, Drawable, Frame, FrameBuffer, SpriteSheet};
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct Letters {
    sprite_sheet: Rc<SpriteSheet<char, Bitmap>>,
}

impl Letters {
    pub fn new() -> Self {
        Self {
            sprite_sheet: Rc::new(init_letters_sprite()),
        }
    }

    pub fn draw_string(&self, s: &str, mut x: u32, y: u32, buf: &mut FrameBuffer) {
        for ch in s.chars() {
            match self.sprite_sheet.get_sprite_drawable(&ch, x, y) {
                Some(mut drawable) => drawable
                    .with_substitute_color(Color::black(), Color::light_grey())
                    .draw(buf),
                None => {
                    self.sprite_sheet
                        .get_sprite_drawable(&'?', x, y)
                        .unwrap()
                        .with_substitute_color(Color::black(), Color::light_grey())
                        .draw(buf);
                }
            }
            x += 13;
        }
    }
}

fn init_letters_sprite() -> SpriteSheet<char, Bitmap> {
    let image =
        Bitmap::from_reader(&mut include_bytes!("../../assets/letters.bmp").as_slice()).unwrap();
    let mut sprite_sheet = SpriteSheet::new(image);
    let letters = Frame {
        x: 0,
        y: 0,
        w: 15,
        h: 15,
    };
    // sprite_sheet.ignore_colour(Color::white());
    // TODO: This comes out of bitmap, not sure why
    sprite_sheet.ignore_color(Color::from_rgba(4294836220));
    for (i, ch) in ('A'..'Z').enumerate() {
        sprite_sheet.add_area(ch, letters.offset_xy(i as u32 * 15, 0));
    }
    for (i, ch) in ('a'..'z').enumerate() {
        sprite_sheet.add_area(ch, letters.offset_xy(i as u32 * 15, 15));
    }
    for (i, ch) in ('0'..'9').enumerate() {
        sprite_sheet.add_area(ch, letters.offset_xy(i as u32 * 15, 30));
    }
    sprite_sheet.add_area('?', letters.offset_xy(150, 30));

    sprite_sheet
}
