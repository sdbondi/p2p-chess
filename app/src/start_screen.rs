use crate::components::TextBox;
use crate::letters::Letters;
use crate::{Bitmap, Color, Drawable, Frame, FrameBuffer, SpriteSheet};
use minifb::Window;
use std::rc::Rc;

pub struct StartScreen {
    public_key_input: TextBox,
}

impl StartScreen {
    pub fn new() -> Self {
        let letters = Letters::new();
        Self {
            public_key_input: TextBox::new(letters),
        }
    }

    pub fn update(&mut self, window: &Window) {
        self.public_key_input.update(window);
    }
}

impl Drawable for StartScreen {
    fn draw(&mut self, buf: &mut FrameBuffer) {
        self.public_key_input.draw(buf);
    }
}
