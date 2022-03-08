use minifb::Window;
use tari_comms::types::CommsPublicKey;
use tari_utilities::hex::Hex;

use crate::{
    clipboard::Clipboard,
    color::Color,
    components::{Button, Label, ListBox, TextBox},
    drawable::{Drawable, FrameBuffer},
    game::GameCollection,
    rect::{Frame, Rect},
};

#[derive(Debug)]
pub struct Drawables<T> {
    pub items: Vec<T>,
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
    show_game: Button,
    selected_game: Option<usize>,
    labels: Drawables<Label>,
    submitted_public_key: Option<String>,
    games_selector: ListBox,
}

impl StartScreen {
    pub fn new(clipboard: Clipboard, public_key: CommsPublicKey) -> Self {
        let mut title_label = Label::new(Frame::new(495, 10, 500, 40));
        title_label.set_text("P2P Chess").set_text_color(Color::dark_green());

        let mut my_pk_label = Label::new(Frame::new(10, 50, 500, 40));
        my_pk_label
            .set_text(format!("Player public key {}", public_key))
            .set_text_color(Color::light_grey());

        let mut enter_pk_label = Label::new(Frame::new(10, 150, 500, 40));
        enter_pk_label.set_text("Enter player public key");

        let mut error_label = Label::new(Frame::new(10, 350, 500, 40));
        error_label.set_text("").set_text_color(Color::red());
        let labels = Drawables {
            items: vec![title_label, my_pk_label, enter_pk_label, error_label],
        };

        let public_key_input = TextBox::new(Frame::new(10, 200, 750, 40), clipboard);
        let mut start_button = Button::new(Rect::new(10, 280, 100, 30, Color::white()));
        start_button.set_text("New Game");

        let mut copy_button = Button::new(Rect::new(10, 100, 100, 30, Color::white()));
        copy_button.set_text("Copy").on_click(move || {
            Clipboard::initialize()
                .unwrap()
                .set_contents(public_key.to_hex())
                .unwrap()
        });
        let mut show_game = Button::new(Rect::new(10, 580, 100, 30, Color::white()));
        show_game.set_text("Open Game");

        let games_selector = ListBox::new(Frame::new(10, 350, 900, 200));
        Self {
            public_key_input,
            start_button,
            copy_button,
            labels,
            selected_game: None,
            submitted_public_key: None,
            games_selector,
            show_game,
        }
    }

    pub fn update(&mut self, window: &Window) {
        self.public_key_input.update(window);
        self.start_button.update(window);
        self.copy_button.update(window);
        self.games_selector.update(window);
        self.show_game.update(window);
        if self.start_button.was_clicked() {
            self.submitted_public_key = Some(self.public_key_input.value().to_string())
        }
        if self.show_game.was_clicked() {
            dbg!("SHOW GAME CLICKED");
            self.selected_game = self.games_selector.selected_index();
        }
    }

    pub fn new_game_clicked(&self) -> Option<&str> {
        self.submitted_public_key.as_deref()
    }

    pub fn show_game_clicked(&self) -> Option<usize> {
        self.selected_game
    }

    pub fn set_input_error<T: Into<String>>(&mut self, msg: T) -> &mut Self {
        // TODO: bleh
        self.labels.items.last_mut().unwrap().set_text(msg);
        self
    }

    pub fn set_games(&mut self, games: &GameCollection) {
        self.games_selector
            .set_values(games.iter().map(|g| format!("{} {}", g.id, g.opponent)).collect());
    }
}

impl Drawable for StartScreen {
    fn draw(&mut self, buf: &mut FrameBuffer) {
        self.public_key_input.draw(buf);
        self.start_button.draw(buf);
        self.copy_button.draw(buf);
        self.labels.draw(buf);
        self.games_selector.draw(buf);
        self.show_game.draw(buf);
    }
}
