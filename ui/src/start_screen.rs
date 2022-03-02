use crate::clipboard::Clipboard;
use crate::color::Color;
use crate::components::{Button, Label, TextBox};
use crate::drawable::{Drawable, FrameBuffer};
use crate::letters::Letters;
use crate::rect::{Frame, Rect};
use minifb::Window;

#[derive(Debug)]
pub struct Drawables<T> {
    items: Vec<T>,
}

impl<T: Drawable> Drawable for Drawables<T> {
    fn draw(&mut self, buf: &mut FrameBuffer) {
        for item in self.items.iter_mut() {
            item.draw(buf);
        }
    }
}

#[derive(Debug)]
pub struct StartScreen {
    public_key_input: TextBox,
    start_button: Button,
    labels: Drawables<Label>,
    submitted_public_key: Option<String>,
}

impl StartScreen {
    pub fn new(clipboard: Clipboard, public_key: &str) -> Self {
        let letters = Letters::new();

        let mut title_label = Label::new(Frame::new(495, 10, 500, 40), letters.clone());
        title_label
            .set_text("P2P Chess")
            .set_text_color(Color::dark_green());

        let mut my_pk_label = Label::new(Frame::new(10, 50, 500, 40), letters.clone());
        my_pk_label
            .set_text(format!("Player public key {}", public_key))
            .set_text_color(Color::light_grey());

        let mut enter_pk_label = Label::new(Frame::new(10, 100, 500, 40), letters.clone());
        enter_pk_label.set_text("Enter player public key");
        let labels = Drawables {
            items: vec![title_label, my_pk_label, enter_pk_label],
        };

        let public_key_input =
            TextBox::new(Frame::new(10, 150, 750, 40), letters.clone(), clipboard);
        let mut start_button = Button::new(
            Rect::new(Frame::new(10, 200, 100, 30), Color::white()),
            letters,
        );
        start_button.set_text("New Game");

        Self {
            public_key_input,
            start_button,
            labels,
            submitted_public_key: None,
        }
    }

    pub fn update(&mut self, window: &Window) {
        self.public_key_input.update(window);
        self.start_button.update(window);
        if self.start_button.was_clicked() {
            self.submitted_public_key = Some(self.public_key_input.value().to_string())
        }
    }
    pub fn new_game_clicked(&self) -> Option<&str> {
        self.submitted_public_key.as_deref()
    }
}

impl Drawable for StartScreen {
    fn draw(&mut self, buf: &mut FrameBuffer) {
        self.public_key_input.draw(buf);
        self.start_button.draw(buf);
        self.labels.draw(buf);
    }
}
