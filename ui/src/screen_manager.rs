use crate::clipboard::Clipboard;
use crate::color::Color;
use crate::command::{ChessCommand, CommandPublisher};
use crate::drawable::{Drawable, FrameBuffer};
use crate::game::{GameConfig, GameScreen, GameStatus};
use crate::start_screen::StartScreen;
use commands::{ChessOperation, CommandPublisher};
use minifb::Window;
use tari_common_types::types::PublicKey;

#[derive(Debug)]
pub struct ScreenManager {
    config: GameConfig,
    active_screen: Screen,
    clipboard: Clipboard,
    publisher: CommandPublisher,
    games: Vec<Game>,
}

impl ScreenManager {
    pub fn initialize(publisher: CommandPublisher, config: GameConfig) -> anyhow::Result<Self> {
        let clipboard = Clipboard::initialize()?;

        Ok(Self {
            config,
            active_screen: Screen::Start(StartScreen::new(clipboard.clone(), "TODO")),
            clipboard,
            publisher,
            games: vec![],
        })
    }

    fn create_new_game(&mut self, pk: PublicKey) {
        buf.clear(Color::black());
        self.active_screen = Screen::Game(GameScreen::new(self.config.clone()));
        self.publisher
            .publish(ChessOperation::NewGame { public_key: pk })
            .unwrap();
        // TODO
    }

    pub fn render(&mut self, window: &Window, buf: &mut FrameBuffer) {
        match self.active_screen {
            Screen::Start(ref mut main_screen) => {
                main_screen.draw(buf);
                main_screen.update(window);
                if let Some(pk) = main_screen.new_game_clicked() {
                    match PublicKey::from_hex(pk) {
                        Ok(pk) => {
                            self.start_new_game(pk);
                        }
                        Err(_) => {
                            main_screen.set_input_error("Invalid public key");
                        }
                    }
                }
            }
            Screen::Game(ref mut game) => {
                game.draw(buf);
                game.update(&window);
                match game.state().game_status() {
                    // TODO
                    GameStatus::StaleMate | GameStatus::CheckMate(_) | GameStatus::Resign(_) => {
                        self.publisher.publish(ChessOperation::GameEnded).unwrap();
                        buf.clear(Color::black());
                        self.active_screen =
                            Screen::Start(StartScreen::new(self.clipboard.clone(), "TODO"));
                    }
                    _ => {}
                }
            }
        }
    }

    pub fn apply_event(&self, event: ChessOperation) {
        match event {
            ChessOperation::NewGame { public_key } => {}
            ChessOperation::MovePlayed(_) => {}
            ChessOperation::GameEnded => {}
        }
    }
}

#[derive(Debug)]
enum Screen {
    Start(StartScreen),
    Game(GameScreen),
}
