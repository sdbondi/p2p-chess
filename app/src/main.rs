mod board;
mod drawable;
mod rect;
mod sprite;

use crate::board::ChessBoard;
use crate::drawable::{Drawable, FrameBuffer};
use crate::rect::{Colour, Frame, Rect};
use crate::sprite::Sprite;
use minifb::{HasRawWindowHandle, Key, Scale, ScaleMode, Window, WindowOptions};
use rand::rngs::OsRng;
use rand::Rng;
use std::cmp;
use std::fs::File;
use std::time::Duration;
use tokio::task;

const WINDOW_WIDTH: usize = 1024;
const WINDOW_HEIGHT: usize = 90 * 8;
const BACKGROUND_COLOUR: Colour = Colour::black();

// #[tokio::main]
fn main() -> anyhow::Result<()> {
    // task::spawn_blocking(main_loop).await??;
    let opts = WindowOptions {
        // scale: Scale::X4,
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
    let mut buf = FrameBuffer::new(WINDOW_WIDTH, WINDOW_HEIGHT);

    buf.clear(BACKGROUND_COLOUR);
    let board = ChessBoard::new(
        Frame {
            x: 0,
            y: 0,
            w: WINDOW_HEIGHT as u32,
            h: WINDOW_HEIGHT as u32,
        },
        Colour::white(),
        Colour::black(),
    );

    while window.is_open() && !window.is_key_down(Key::Escape) {
        board.draw(&mut buf);
        for i in 0..100 {
            buf.put_pixel(100 + i, 100 + i, Colour::green().to_rgb());
        }
        window.update_with_buffer(buf.as_slice(), WINDOW_WIDTH, WINDOW_HEIGHT)?;
    }
    Ok(())
}
