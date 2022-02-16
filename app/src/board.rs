use crate::{Colour, Drawable, Frame, FrameBuffer, Rect};

pub struct ChessBoard {
    frame: Frame,
    light_colour: Colour,
    dark_colour: Colour,
}

impl ChessBoard {
    pub fn new(frame: Frame, light_colour: Colour, dark_colour: Colour) -> Self {
        Self {
            frame,
            light_colour,
            dark_colour,
        }
    }
}

impl Drawable for ChessBoard {
    fn draw(&self, buf: &mut FrameBuffer) {
        for x in 0..8 {
            for y in 0..8 {
                Rect::new(
                    x * self.frame.w / 8,
                    y * self.frame.h / 8,
                    self.frame.w / 8,
                    self.frame.h / 8,
                    if y % 2 == 0 {
                        if x % 2 == 0 {
                            self.light_colour
                        } else {
                            self.dark_colour
                        }
                    } else {
                        if x % 2 == 0 {
                            self.dark_colour
                        } else {
                            self.light_colour
                        }
                    },
                )
                .draw(buf);
            }
        }
    }
}
