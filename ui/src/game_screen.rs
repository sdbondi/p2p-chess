use std::{
    fmt::{Display, Formatter},
    path::PathBuf,
};

use minifb::{MouseButton, MouseMode, Window};
use pleco::{
    core::piece_move::{MoveFlag, PreMoveInfo},
    BitMove,
    PieceType,
    Player,
};
use tari_comms::types::CommsPublicKey;

use crate::{
    bitmap::Bitmap,
    board::ChessBoard,
    color::Color,
    components::{Button, Label},
    drawable::{Drawable, FrameBuffer},
    rect::{Frame, Rect},
    sprite::SpriteSheet,
    start_screen::Drawables,
};

#[derive(Debug)]
pub struct GameScreen {
    config: GameConfig,
    id: u32,
    seq: u32,
    state: State,
    board: ChessBoard,
    floating_piece: Option<(u32, u32)>,
    opponent: CommsPublicKey,
    last_move_played: Option<BitMove>,
    back_button: Button,
}

impl GameScreen {
    pub fn new(
        id: u32,
        seq: u32,
        config: GameConfig,
        player: Player,
        opponent: CommsPublicKey,
        board_fen: &str,
    ) -> Self {
        let mut board = ChessBoard::new(
            Frame {
                x: 0,
                y: 0,
                w: config.window_height,
                h: config.window_height,
            },
            config.light_color,
            config.dark_color,
            init_pieces_sprite(),
            player,
        );
        board.set_board_state(board_fen);
        let mut back_button = Button::new(Rect::new(config.window_height + 10, 50, 100, 20, Color::white()));
        back_button.set_text("Back");

        Self {
            config,
            id,
            seq,
            state: State::default(),
            board,
            floating_piece: None,
            opponent,
            last_move_played: None,
            back_button,
        }
    }

    pub fn game_id(&self) -> u32 {
        self.id
    }

    pub fn set_seq(&mut self, seq: u32) -> &mut Self {
        self.seq = seq;
        self
    }

    pub fn inc_seq(&mut self) -> u32 {
        self.seq += 1;
        self.seq
    }

    pub fn set_board_state(&mut self, fen: &str, mv: Option<BitMove>) -> &mut Self {
        self.board.set_board_state(fen);
        if let Some(mv) = mv {
            self.board.set_last_move(mv);
        }
        self
    }

    pub fn update(&mut self, window: &Window) {
        self.state.update(window);
        self.back_button.update(window);

        if self.board.is_stalemate() {
            self.state.set_game_status(GameStatus::StaleMate);
        }

        if self.board.is_draw() {
            self.state.set_game_status(GameStatus::StaleMate);
        }

        if self.board.is_checkmate() {
            self.state
                .set_game_status(GameStatus::CheckMate(self.board.turn().other_player()));
        }
    }

    pub fn state(&self) -> &State {
        &self.state
    }

    pub fn apply_move(&mut self, mv: BitMove) {
        dbg!("apply_move", mv);
        self.board.apply_move(mv);
    }

    pub fn seq(&self) -> u32 {
        self.seq
    }

    pub fn opponent(&self) -> &CommsPublicKey {
        &self.opponent
    }

    pub fn take_last_move_played(&mut self) -> Option<BitMove> {
        self.last_move_played.take()
    }

    pub fn to_board_fen(&self) -> String {
        self.board.to_fen()
    }

    pub fn was_back_clicked(&mut self) -> bool {
        self.back_button.was_clicked()
    }

    fn labels(&self) -> Drawables<Label> {
        let mut label1 = Label::new(Frame::new(self.board.height() + 10, 80, 100, 20));
        label1.set_text(format!("ID: {}", self.id)).set_bg_color(Color::black());

        let mut label2 = Label::new(Frame::new(self.board.height() + 10, 110, 100, 20));
        label2
            .set_text(format!("MOVE: {}", self.seq))
            .set_bg_color(Color::black());

        let mut label3 = Label::new(Frame::new(self.board.height() + 10, 140, 100, 20));
        label3
            .set_text(format!("Turn: {}", self.board.turn()))
            .set_bg_color(Color::black());

        let mut label4 = Label::new(Frame::new(self.board.height() + 10, 170, 100, 20));
        label4
            .set_text(format!("Status: {}", self.state().game_status()))
            .set_bg_color(Color::black());

        Drawables {
            items: vec![label1, label2, label3, label4],
        }
    }
}

impl Drawable for GameScreen {
    fn draw(&mut self, buf: &mut FrameBuffer) {
        self.board.draw(buf);

        // Clear score board
        Rect::new(
            self.board.width(),
            0,
            self.config.window_width - self.board.width(),
            self.config.window_height,
            Color::black(),
        )
        .draw(buf);

        self.back_button.draw(buf);
        self.labels().draw(buf);

        if self.state.is_left_mouse_down {
            if let Some((mouse_x, mouse_y)) = self.state.mouse_pos {
                match self.floating_piece {
                    Some((offset_x, offset_y)) => {
                        self.board.draw_taken_piece(
                            mouse_x.saturating_sub(offset_x),
                            mouse_y.saturating_sub(offset_y),
                            buf,
                        );
                    },
                    None => {
                        if self.board.take_piece_at(mouse_x, mouse_y).is_some() {
                            self.floating_piece = Some((mouse_x % 90, mouse_y % 90));
                        }
                    },
                }
            }
        } else {
            if self.floating_piece.is_some() {
                match self.state.mouse_pos.and_then(|(x, y)| self.board.get_square(x, y)) {
                    Some(sq) => match self.board.get_move_to(sq) {
                        Some(mut legal_move) => {
                            if legal_move.is_promo() {
                                legal_move = BitMove::init(PreMoveInfo {
                                    src: legal_move.get_src(),
                                    dst: legal_move.get_dest(),
                                    flags: MoveFlag::Promotion {
                                        capture: legal_move.is_capture(),
                                        prom: PieceType::Q,
                                    },
                                });
                            }
                            self.board.make_legal_move(legal_move);
                            self.last_move_played = Some(legal_move);
                            self.board.set_last_move(legal_move);
                        },
                        None => {
                            self.board.return_taken_piece();
                        },
                    },
                    None => {
                        self.board.return_taken_piece();
                    },
                }
                self.floating_piece = None;
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct GameConfig {
    pub window_width: u32,
    pub window_height: u32,
    pub light_color: Color,
    pub dark_color: Color,
    pub save_path: PathBuf,
}

#[derive(Debug)]
pub struct State {
    mouse_pos: Option<(u32, u32)>,
    is_left_mouse_down: bool,
    game_status: GameStatus,
}

impl State {
    pub fn new() -> Self {
        Self {
            mouse_pos: None,
            is_left_mouse_down: false,
            game_status: Default::default(),
        }
    }

    pub fn update(&mut self, window: &Window) {
        self.mouse_pos = window
            .get_mouse_pos(MouseMode::Discard)
            .map(|(x, y)| (x.round() as u32, y.round() as u32));
        self.is_left_mouse_down = window.get_mouse_down(MouseButton::Left);
    }

    pub(crate) fn set_game_status(&mut self, status: GameStatus) -> &mut Self {
        self.game_status = status;
        self
    }

    pub fn game_status(&self) -> GameStatus {
        self.game_status
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

fn init_pieces_sprite() -> SpriteSheet<&'static str, Bitmap> {
    let image = Bitmap::from_reader(&mut include_bytes!("../assets/pieces.bmp").as_slice()).unwrap();
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

#[derive(Debug, Clone, Copy)]
pub enum GameStatus {
    InProgress,
    StaleMate,
    CheckMate(Player),
    Resign(Player),
}

impl Default for GameStatus {
    fn default() -> Self {
        Self::InProgress
    }
}

impl Display for GameStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            GameStatus::InProgress => write!(f, "In progress"),
            GameStatus::StaleMate => write!(f, "Stale mate"),
            GameStatus::CheckMate(player) => write!(f, "Checkmate! {} won", player),
            GameStatus::Resign(player) => write!(f, "{} resigned", player),
        }
    }
}
