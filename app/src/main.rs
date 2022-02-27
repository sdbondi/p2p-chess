mod bitmap;
mod board;
mod colour;
mod components;
mod drawable;
mod game;
mod letters;
mod palette;
mod rect;
mod sprite;
mod start_screen;

use crate::bitmap::Bitmap;
use crate::board::ChessBoard;
use crate::colour::Color;
use crate::drawable::{Drawable, FrameBuffer};
use crate::game::{Game, GameConfig};
use crate::rect::{Frame, Rect};
use crate::sprite::SpriteSheet;
use crate::start_screen::StartScreen;
use minifb::{Key, ScaleMode, Window, WindowOptions};
use std::time::Duration;

const WINDOW_WIDTH: usize = 1024;
const WINDOW_HEIGHT: usize = 90 * 8;
const BACKGROUND_COLOUR: Color = Color::black();

fn main() -> anyhow::Result<()> {
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
    let mut buf = FrameBuffer::new(WINDOW_WIDTH, WINDOW_HEIGHT, BACKGROUND_COLOUR);

    let config = GameConfig {
        window_width: WINDOW_WIDTH as u32,
        window_height: WINDOW_HEIGHT as u32,
        light_color: Color::cream(),
        dark_color: Color::dark_green(),
    };
    let mut game = Game::new(config);

    let mut start_screen = StartScreen::new();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        start_screen.draw(&mut buf);
        start_screen.update(&window);
        // game.draw(&mut buf);
        // game.update(&window);

        window.update_with_buffer(buf.as_slice(), WINDOW_WIDTH, WINDOW_HEIGHT)?;
    }
    Ok(())
}
