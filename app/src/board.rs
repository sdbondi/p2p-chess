use crate::{Colour, Drawable, FrameBuffer, Rect};

pub struct Board {}

impl Board {
    pub fn new() -> Self {
        Self {}
    }
}

impl Drawable for Board {
    fn draw(&self, buf: &mut FrameBuffer) {
        for i in 0..4 * 8 {
            Rect {
                x: (i * 200) % 800,
                y: 100 * (i / 4),
                w: 100,
                h: 100,
                colour: if (i / 4) % 2 == 0 {
                    Colour::white()
                } else {
                    Colour::black()
                },
            }
            .draw(buf);
            Rect {
                x: ((i + 1) * 200) % 800,
                y: 100 * (i / 4),
                w: 100,
                h: 100,
                colour: if (i / 4) % 2 == 0 {
                    Colour::black()
                } else {
                    Colour::white()
                },
            }
            .draw(buf);
        }
    }
}
