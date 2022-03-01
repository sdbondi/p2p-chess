use crate::clipboard::Clipboard;
use crate::color::Color;
use crate::drawable::{Drawable, FrameBuffer};
use crate::events::{ChessUiEvent, EventPublisher};
use crate::game::{GameConfig, GameScreen, GameStatus};
use crate::start_screen::StartScreen;
use minifb::Window;

#[derive(Debug)]
pub struct ScreenManager {
    config: GameConfig,
    active_screen: Screen,
    clipboard: Clipboard,
    publisher: EventPublisher,
}

impl ScreenManager {
    pub fn initialize(publisher: EventPublisher, config: GameConfig) -> anyhow::Result<Self> {
        let clipboard = Clipboard::initialize()?;

        Ok(Self {
            config,
            active_screen: Screen::Start(StartScreen::new(clipboard.clone(), "TODO")),
            clipboard,
            publisher,
        })
    }

    pub fn render(&mut self, window: &Window, buf: &mut FrameBuffer) {
        match self.active_screen {
            Screen::Start(ref mut main_screen) => {
                main_screen.draw(buf);
                main_screen.update(window);
                // TODO
                if let Some(pk) = main_screen
                    .submitted_public_key_str()
                    .map(ToString::to_string)
                {
                    buf.clear(Color::black());
                    self.active_screen = Screen::Game(GameScreen::new(self.config.clone()));
                    self.publisher
                        .publish(ChessUiEvent::NewGame { public_key: pk })
                        .unwrap();
                }
            }
            Screen::Game(ref mut game) => {
                game.draw(buf);
                game.update(&window);
                match game.state().game_status() {
                    // TODO
                    GameStatus::StaleMate | GameStatus::CheckMate(_) | GameStatus::Resign(_) => {
                        self.publisher.publish(ChessUiEvent::GameEnded).unwrap();
                        buf.clear(Color::black());
                        self.active_screen =
                            Screen::Start(StartScreen::new(self.clipboard.clone(), "TODO"));
                    }
                    _ => {}
                }
            }
        }
    }
}

#[derive(Debug)]
enum Screen {
    Start(StartScreen),
    Game(GameScreen),
}
