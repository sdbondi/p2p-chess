mod bitmap;
mod board;
mod colour;
mod drawable;
mod palette;
mod rect;
mod sprite;

use crate::bitmap::Bitmap;
use crate::board::ChessBoard;
use crate::colour::Colour;
use crate::drawable::{Drawable, FrameBuffer};
use crate::rect::{Frame, Rect};
use crate::sprite::SpriteSheet;
use minifb::{
    HasRawWindowHandle, Key, MouseButton, MouseMode, Scale, ScaleMode, Window, WindowOptions,
};
use pleco::{Player, SQ};
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
    let mut buf = FrameBuffer::new(WINDOW_WIDTH, WINDOW_HEIGHT);

    buf.clear(BACKGROUND_COLOUR);
    let mut board = ChessBoard::new(
        Frame {
            x: 0,
            y: 0,
            w: WINDOW_HEIGHT as u32,
            h: WINDOW_HEIGHT as u32,
        },
        Colour::cream(),
        Colour::dark_green(),
        init_pieces_sprite(),
        Player::Black,
    );

    let mut game_state = GameState::new();

    let ls = init_letters_sprite();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        board.draw(&mut buf);
        game_state.update(&window);
        ls.get_sprite_drawable(&'A', 100, 100)
            .unwrap()
            .draw(&mut buf);

        if game_state.is_left_mouse_down {
            if let Some((mouse_x, mouse_y)) = game_state.mouse_pos {
                match game_state.floating_piece {
                    Some((offset_x, offset_y)) => {
                        board.draw_taken_piece(
                            mouse_x.saturating_sub(offset_x),
                            mouse_y.saturating_sub(offset_y),
                            &mut buf,
                        );
                        // TODO: good UX
                        // board.highlight_square_at(mouse_x, mouse_y, buf);
                    }
                    None => {
                        if board.take_piece_at(mouse_x, mouse_y).is_some() {
                            game_state.floating_piece = Some((mouse_x % 90, mouse_y % 90));
                        }
                    }
                }
            }
        } else {
            if game_state.floating_piece.is_some() {
                match game_state
                    .mouse_pos
                    .and_then(|(x, y)| board.get_square(x, y))
                {
                    Some(sq) => {
                        if !board.make_move_to(sq) {
                            board.return_taken_piece();
                        }
                    }
                    None => {
                        board.return_taken_piece();
                    }
                }
                game_state.floating_piece = None;
            }
        }

        window.update_with_buffer(buf.as_slice(), WINDOW_WIDTH, WINDOW_HEIGHT)?;
    }
    Ok(())
}

fn init_pieces_sprite() -> SpriteSheet<&'static str, Bitmap> {
    let image =
        Bitmap::from_reader(&mut include_bytes!("../../assets/pieces.bmp").as_slice()).unwrap();
    let mut sprite_sheet = SpriteSheet::new(image);
    let pieces = Frame {
        x: 0,
        y: 0,
        w: 90,
        h: 90,
    };
    sprite_sheet
        .ignore_colour(Colour::green())
        .add_area("king-white", pieces)
        .add_area("queen-white", pieces.offset_xy(90, 0))
        .add_area("rook-white", pieces.offset_xy(180, 0))
        .add_area("bishop-white", pieces.offset_xy(270, 0))
        .add_area("knight-white", pieces.offset_xy(360, 0))
        .add_area("pawn-white", pieces.offset_xy(0, 90))
        .add_area("king-black", pieces.offset_xy(90, 90))
        .add_area("queen-black", pieces.offset_xy(180, 90))
        .add_area("rook-black", pieces.offset_xy(270, 90))
        .add_area("bishop-black", pieces.offset_xy(360, 90))
        .add_area("knight-black", pieces.offset_xy(0, 180))
        .add_area("pawn-black", pieces.offset_xy(90, 180));
    sprite_sheet
}

fn init_letters_sprite() -> SpriteSheet<char, Bitmap> {
    let image =
        Bitmap::from_reader(&mut include_bytes!("../../assets/letters.bmp").as_slice()).unwrap();
    let mut sprite_sheet = SpriteSheet::new(image);
    let letters = Frame {
        x: 0,
        y: 0,
        w: 17,
        h: 17,
    };
    sprite_sheet.ignore_colour(Colour::green());
    for (i, ch) in ('A'..'Z').enumerate() {
        sprite_sheet.add_area(ch, letters.offset_xy(i as u32 * 17, 0));
    }
    sprite_sheet
}

struct GameState {
    floating_piece: Option<(u32, u32)>,
    mouse_pos: Option<(u32, u32)>,
    is_left_mouse_down: bool,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            floating_piece: None,
            mouse_pos: None,
            is_left_mouse_down: false,
        }
    }

    pub fn update(&mut self, window: &Window) {
        self.mouse_pos = window
            .get_mouse_pos(MouseMode::Discard)
            .map(|(x, y)| (x.round() as u32, y.round() as u32));
        self.is_left_mouse_down = window.get_mouse_down(MouseButton::Left);
    }
}
