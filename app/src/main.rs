use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc;
use std::time::Duration;
use tokio::task;
use ui::bitmap::Bitmap;
use ui::board::ChessBoard;
use ui::clipboard::Clipboard;
use ui::color::Color;
use ui::drawable::{Drawable, FrameBuffer};
use ui::game::{Game, GameConfig};
use ui::rect::{Frame, Rect};
use ui::sprite::SpriteSheet;
use ui::start_screen::StartScreen;
use ui::{Key, ScaleMode, Window, WindowOptions};

const WINDOW_WIDTH: usize = 1024;
const WINDOW_HEIGHT: usize = 90 * 8;
const BACKGROUND_COLOUR: Color = Color::black();

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // let ui = Ui::new();
    // let (game_state_tx, game_state_update_rx) = watch::channel(());
    // let commands = ui.command_subscription();
    // ui.set_state_update_watch(game_state);
    //
    // task::spawn(async move{
    //     loop {
    //         tokio::select! {
    //             cmd = commands.recv() => {
    //
    //             },
    //             _ = shutdown_signal.wait() => {
    //                 break;
    //             }
    //         }
    //     }
    // });
    //
    // task::spawn_blocking(move ||{
    //     ui.run();
    // }).await?;

    let opts = WindowOptions {
        title: true,
        scale_mode: ScaleMode::Center,
        resize: true,
        ..Default::default()
    };
    let mut window = Window::new("P2P Chess", WINDOW_WIDTH, WINDOW_HEIGHT, opts)?;

    // ~60fps
    window.limit_update_rate(Some(Duration::from_micros(16600)));

    ui_loop(window)?;

    Ok(())
}

fn ui_loop(mut window: Window) -> anyhow::Result<()> {
    let clipboard = Clipboard::initialize()?;
    let mut buf = FrameBuffer::new(WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32, BACKGROUND_COLOUR);

    let config = GameConfig {
        window_width: WINDOW_WIDTH as u32,
        window_height: WINDOW_HEIGHT as u32,
        light_color: Color::cream(),
        dark_color: Color::dark_green(),
    };
    let mut game = Game::new(config);

    let mut active_screen = Rc::new(RefCell::new(Screen::MainScreen));

    let mut start_screen = StartScreen::new(clipboard);
    start_screen.on_submitted({
        let active_screen = active_screen.clone();
        move |public_key| {
            *active_screen.borrow_mut() = Screen::Game;
        }
    });

    let mut should_exit = false;

    while window.is_open() && !should_exit {
        let screen = *active_screen.borrow();
        match screen {
            Screen::MainScreen => {
                start_screen.draw(&mut buf);
                start_screen.update(&window);
            }
            Screen::Game => {
                game.draw(&mut buf);
                game.update(&window);
            }
        }

        if let Some(keys) = window.get_keys() {
            if (keys.contains(&Key::LeftCtrl) || keys.contains(&Key::RightCtrl))
                && keys.contains(&Key::Q)
            {
                should_exit = true;
            }
        }

        window.update_with_buffer(buf.as_slice(), WINDOW_WIDTH, WINDOW_HEIGHT)?;
    }
    Ok(())
}

#[derive(Debug, Clone, Copy)]
enum Screen {
    MainScreen,
    Game,
}
