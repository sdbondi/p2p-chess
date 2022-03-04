use minifb::Window;
use tari_comms::types::CommsPublicKey;
use tari_utilities::hex::Hex;

use crate::{
    clipboard::Clipboard,
    color::Color,
    components::{Button, Label, TextBox},
    drawable::{Drawable, FrameBuffer},
    letters::Letters,
    rect::{Frame, Rect},
};

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
    copy_button: Button,
    labels: Drawables<Label>,
    submitted_public_key: Option<String>,
}

impl StartScreen {
    pub fn new(clipboard: Clipboard, public_key: CommsPublicKey) -> Self {
        let letters = Letters::new();

        let mut title_label = Label::new(Frame::new(495, 10, 500, 40), letters.clone());
        title_label.set_text("P2P Chess").set_text_color(Color::dark_green());

        let mut my_pk_label = Label::new(Frame::new(10, 50, 500, 40), letters.clone());
        my_pk_label
            .set_text(format!("Player public key {}", public_key))
            .set_text_color(Color::light_grey());

        let mut enter_pk_label = Label::new(Frame::new(10, 150, 500, 40), letters.clone());
        enter_pk_label.set_text("Enter player public key");

        let mut error_label = Label::new(Frame::new(10, 350, 500, 40), letters.clone());
        error_label.set_text("").set_text_color(Color::red());
        let labels = Drawables {
            items: vec![title_label, my_pk_label, enter_pk_label, error_label],
        };

        let public_key_input = TextBox::new(Frame::new(10, 200, 750, 40), letters.clone(), clipboard);
        let mut start_button = Button::new(Rect::new(Frame::new(10, 300, 100, 30), Color::white()), letters.clone());
        start_button.set_text("New Game");

        let mut copy_button = Button::new(Rect::new(Frame::new(10, 100, 100, 30), Color::white()), letters);
        copy_button.set_text("Copy").on_click(move || {
            Clipboard::initialize()
                .unwrap()
                .set_contents(public_key.to_hex())
                .unwrap()
        });

        Self {
            public_key_input,
            start_button,
            copy_button,
            labels,
            submitted_public_key: None,
        }
    }

    pub fn update(&mut self, window: &Window) {
        self.public_key_input.update(window);
        self.start_button.update(window);
        self.copy_button.update(window);
        if self.start_button.was_clicked() {
            self.submitted_public_key = Some(self.public_key_input.value().to_string())
        }
    }

    pub fn new_game_clicked(&self) -> Option<&str> {
        self.submitted_public_key.as_deref()
    }

    pub fn set_input_error<T: Into<String>>(&mut self, msg: T) -> &mut Self {
        // TODO: bleh
        self.labels.items.last_mut().unwrap().set_text(msg);
        self
    }
}

impl Drawable for StartScreen {
    fn draw(&mut self, buf: &mut FrameBuffer) {
        self.public_key_input.draw(buf);
        self.start_button.draw(buf);
        self.copy_button.draw(buf);
        self.labels.draw(buf);
    }
}
