mod bitmap;
mod board;
mod colour;
mod drawable;
mod palette;
mod png;
mod rect;
mod sprite;

use crate::bitmap::Bitmap;
use crate::board::ChessBoard;
use crate::colour::Colour;
use crate::drawable::{Drawable, FrameBuffer};
use crate::png::Png;
use crate::rect::{Frame, Rect};
use crate::sprite::SpriteSheet;
use minifb::{
    HasRawWindowHandle, Key, MouseButton, MouseMode, Scale, ScaleMode, Window, WindowOptions,
};
use pleco::SQ;
use rand::rngs::OsRng;
use rand::Rng;
use std::cmp;
use std::fs::File;
use std::io::Read;
use std::time::Duration;
use tokio::task;

const WINDOW_WIDTH: usize = 1024;
const WINDOW_HEIGHT: usize = 90 * 8;
const BACKGROUND_COLOUR: Colour = Colour::black();

fn main() -> anyhow::Result<()> {
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
    let sprite_sheet = init_sprite_sheet();
    let mut board = ChessBoard::new(
        Frame {
            x: 0,
            y: 0,
            w: WINDOW_HEIGHT as u32,
            h: WINDOW_HEIGHT as u32,
        },
        Colour::cream(),
        Colour::dark_green(),
        sprite_sheet,
    );
    let mut floating_piece = None;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        board.draw(&mut buf);

        let mouse_pos = window.get_mouse_pos(MouseMode::Discard);

        if window.get_mouse_down(MouseButton::Left) {
            if let Some((mouse_x, mouse_y)) = mouse_pos {
                let mouse_x = mouse_x.floor() as u32;
                let mouse_y = mouse_y.floor() as u32;
                if let Some((offset_x, offset_y)) = floating_piece {
                    board.draw_taken_piece(
                        mouse_x.saturating_sub(offset_x),
                        mouse_y.saturating_sub(offset_y),
                        &mut buf,
                    );
                    // TODO: good UX
                    // board.highlight_square_at(mouse_x, mouse_y, buf);
                } else {
                    if board.take_piece_at(mouse_x, mouse_y).is_some() {
                        floating_piece = Some((mouse_x % 90, mouse_y % 90));
                    }
                }
            }
        } else {
            if floating_piece.is_some() {
                board.return_taken_piece();
                floating_piece = None;
            }
        }
        window.update_with_buffer(buf.as_slice(), WINDOW_WIDTH, WINDOW_HEIGHT)?;
    }
    Ok(())
}

fn init_sprite_sheet() -> SpriteSheet<&'static str, Bitmap> {
    let image = Bitmap::from_bytes(include_bytes!("../../assets/sprite.bmp")).unwrap();
    // let image = Png::from_bytes(include_bytes!("../../assets/sprite.png"))?;
    let mut sprite_sheet = SpriteSheet::new(image);
    let pieces = Frame {
        x: 0,
        y: 93,
        w: 90,
        h: 90,
    };
    sprite_sheet
        // TODO: we get funny colour values from the bitmap, ignoring this rgba value
        .ignore_colour(Colour::from_rgba(14128226))
        .add_area("king-black", pieces.offset_x(90).offset_y(90))
        .add_area("queen-black", pieces.offset_x(180).offset_y(90))
        .add_area("rook-black", pieces.offset_x(270).offset_y(90))
        .add_area("bishop-black", pieces.offset_x(360).offset_y(90))
        .add_area("knight-black", pieces.offset_y(180))
        .add_area("pawn-black", pieces.offset_x(90).offset_y(180))
        .add_area("king-white", pieces)
        .add_area("queen-white", pieces.offset_x(90))
        .add_area("rook-white", pieces.offset_x(180))
        .add_area("bishop-white", pieces.offset_x(270))
        .add_area("knight-white", pieces.offset_x(360))
        .add_area("pawn-white", pieces.offset_y(90))
        .add_area(
            "icons",
            Frame {
                x: 0,
                y: 360,
                w: 135,
                h: 46,
            },
        );
    sprite_sheet
}

fn sq_to_coords(sq: SQ) -> (u32, u32) {
    let x = sq.file() as u32;
    let y = sq.rank() as u32;
    (x, y)
}
