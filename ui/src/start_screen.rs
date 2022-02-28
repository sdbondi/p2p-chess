use crate::clipboard::Clipboard;
use crate::color::Color;
use crate::components::{Button, Label, TextBox};
use crate::drawable::{Drawable, FrameBuffer};
use crate::letters::Letters;
use crate::rect::{Frame, Rect};
use minifb::Window;

pub struct StartScreen {
    public_key_input: TextBox,
    button: Button,
    label: Label,
}

impl StartScreen {
    pub fn new(clipboard: Clipboard) -> Self {
        let letters = Letters::new();
        let public_key_input =
            TextBox::new(Frame::new(10, 50, 750, 40), letters.clone(), clipboard);
        let mut button = Button::new(
            Rect::new(Frame::new(10, 100, 100, 30), Color::cream()),
            letters.clone(),
        );
        button.set_text("OK");

        let mut label = Label::new(Frame::new(10, 10, 500, 40), letters);
        label.set_text("Enter player public key");

        Self {
            public_key_input,
            button,
            label,
        }
    }

    pub fn on_submitted<H: FnMut(String) + 'static>(&mut self, mut handler: H) -> &mut Self {
        self.button.on_click(move || {
            // let value = self.public_key_input.value().to_string();
            handler("todo".to_string())
        });
        self
    }

    pub fn update(&mut self, window: &Window) {
        self.public_key_input.update(window);
        self.button.update(window);
    }
}

impl Drawable for StartScreen {
    fn draw(&mut self, buf: &mut FrameBuffer) {
        self.public_key_input.draw(buf);
        self.button.draw(buf);
        self.label.draw(buf);
    }
}
