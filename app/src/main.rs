mod board;
mod drawable;
mod rect;

use crate::board::Board;
use crate::drawable::{Drawable, FrameBuffer};
use crate::rect::{Colour, Rect};
use minifb::{Key, Scale, ScaleMode, Window, WindowOptions};
use rand::rngs::OsRng;
use rand::Rng;
use std::time::Duration;
use tokio::task;

const WINDOW_WIDTH: usize = 1024;
const WINDOW_HEIGHT: usize = 768;

// #[tokio::main]
fn main() -> anyhow::Result<()> {
    // task::spawn_blocking(main_loop).await??;
    let opts = WindowOptions {
        // scale: Scale::X4,
        // scale_mode: ScaleMode::Center,
        // resize: true,
        ..Default::default()
    };
    let mut window = Window::new("P2P Chess", WINDOW_WIDTH, WINDOW_HEIGHT, opts)?;
    window.limit_update_rate(Some(Duration::from_millis(1)));

    ui_loop(window)?;

    Ok(())
}

fn ui_loop(mut window: Window) -> anyhow::Result<()> {
    let mut buf = FrameBuffer::new(WINDOW_WIDTH, WINDOW_HEIGHT);
    let board = Board::new();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        board.draw(&mut buf);
        // buf.clear(0);
        window.update_with_buffer(buf.as_slice(), WINDOW_WIDTH, WINDOW_HEIGHT);
    }
    Ok(())
}
