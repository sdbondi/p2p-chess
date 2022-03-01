use crate::clipboard::Clipboard;
use crate::color::Color;
use crate::drawable::{Drawable, FrameBuffer};
use crate::game::{GameConfig, GameScreen, GameStatus};
use crate::start_screen::StartScreen;
use minifb::Window;

#[derive(Debug)]
pub struct ScreenManager {
    config: GameConfig,
    active_screen: Screen,
    clipboard: Clipboard,
}

impl ScreenManager {
    pub fn initialize(config: GameConfig) -> anyhow::Result<Self> {
        let clipboard = Clipboard::initialize()?;

        Ok(Self {
            config,
            active_screen: Screen::Start(StartScreen::new(clipboard.clone(), "TODO")),
            clipboard,
        })
    }

    pub fn render(&mut self, window: &Window, buf: &mut FrameBuffer) {
        match self.active_screen {
            Screen::Start(ref mut main_screen) => {
                main_screen.draw(buf);
                main_screen.update(window);
                // TODO
                if main_screen.submitted_public_key_str().is_some() {
                    buf.clear(Color::black());
                    self.active_screen = Screen::Game(GameScreen::new(self.config.clone()));
                }
            }
            Screen::Game(ref mut game) => {
                game.draw(buf);
                game.update(&window);
                match game.state().game_status() {
                    // TODO
                    GameStatus::StaleMate | GameStatus::CheckMate(_) | GameStatus::Resign(_) => {
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
