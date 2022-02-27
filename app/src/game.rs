use crate::{Bitmap, ChessBoard, Color, Drawable, Frame, FrameBuffer, SpriteSheet};
use minifb::{MouseButton, MouseMode, Window};
use pleco::Player;

pub struct Game {
    state: GameState,
    board: ChessBoard,
}

impl Game {
    pub fn new(config: GameConfig) -> Self {
        Self {
            state: GameState::default(),
            board: ChessBoard::new(
                Frame {
                    x: 0,
                    y: 0,
                    w: config.window_height,
                    h: config.window_height,
                },
                config.light_color,
                config.dark_color,
                init_pieces_sprite(),
                Player::Black,
            ),
        }
    }

    pub fn update(&mut self, window: &Window) {
        self.state.update(window);
    }
}

impl Drawable for Game {
    fn draw(&mut self, buf: &mut FrameBuffer) {
        self.board.draw(buf);

        if self.state.is_left_mouse_down {
            if let Some((mouse_x, mouse_y)) = self.state.mouse_pos {
                match self.state.floating_piece {
                    Some((offset_x, offset_y)) => {
                        self.board.draw_taken_piece(
                            mouse_x.saturating_sub(offset_x),
                            mouse_y.saturating_sub(offset_y),
                            buf,
                        );
                        // TODO: good UX
                        // board.highlight_square_at(mouse_x, mouse_y, buf);
                    }
                    None => {
                        if self.board.take_piece_at(mouse_x, mouse_y).is_some() {
                            self.state.floating_piece = Some((mouse_x % 90, mouse_y % 90));
                        }
                    }
                }
            }
        } else {
            if self.state.floating_piece.is_some() {
                match self
                    .state
                    .mouse_pos
                    .and_then(|(x, y)| self.board.get_square(x, y))
                {
                    Some(sq) => {
                        if !self.board.make_move_to(sq) {
                            self.board.return_taken_piece();
                        }
                    }
                    None => {
                        self.board.return_taken_piece();
                    }
                }
                self.state.floating_piece = None;
            }
        }
    }
}

pub struct GameConfig {
    pub window_width: u32,
    pub window_height: u32,
    pub light_color: Color,
    pub dark_color: Color,
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
impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
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
        .ignore_color(Color::green())
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
