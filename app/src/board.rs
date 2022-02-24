use crate::{Bitmap, Colour, Drawable, Frame, FrameBuffer, Rect, SpriteSheet};
use pleco::{BitMove, Board, File, Piece, Player, Rank, SQ};
use std::mem::transmute;

pub struct ChessBoard {
    frame: Frame,
    light_colour: Colour,
    dark_colour: Colour,
    board: Board,
    sprite_sheet: SpriteSheet<&'static str, Bitmap>,
    player: Player,
    taken_piece: Option<(SQ, Piece)>,
}

impl ChessBoard {
    pub fn new(
        frame: Frame,
        light_colour: Colour,
        dark_colour: Colour,
        sprite_sheet: SpriteSheet<&'static str, Bitmap>,
        player: Player,
    ) -> Self {
        let board = match player {
            Player::White => {
                Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap()
            }
            Player::Black => {
                Board::from_fen("RNBQKBNR/PPPPPPPP/8/8/8/8/pppppppp/rnbqkbnr w KQkq - 0 1").unwrap()
            }
        };
        Self {
            frame,
            light_colour,
            dark_colour,
            board,
            sprite_sheet,
            player,
            taken_piece: None,
        }
    }

    pub fn take_piece_at(&mut self, x: u32, y: u32) -> Option<pleco::Piece> {
        let sq = coords_to_sq(x, y, self.player == Player::Black)?;
        match self.board.piece_at_sq(sq) {
            Piece::None => None,
            p => {
                self.taken_piece = Some((sq, p));
                Some(p)
            }
        }
    }

    pub fn return_taken_piece(&mut self) {
        self.taken_piece = None;
    }

    pub fn make_move_to(&mut self, dest: SQ) -> bool {
        if let Some((src, _)) = self.taken_piece {
            let all_moves = self.board.generate_moves();
            if let Some(mv) = all_moves
                .iter()
                .find(|m| m.get_src() == src && m.get_dest() == dest)
            {
                self.board.apply_move(*mv);
                return true;
            }
        }
        self.return_taken_piece();
        false
    }

    pub fn get_square(&self, x: u32, y: u32) -> Option<SQ> {
        coords_to_sq(x, y, self.player == Player::Black)
    }

    fn draw_squares(&self, buf: &mut FrameBuffer) {
        for x in 0..8 {
            for y in 0..8 {
                Rect::new(
                    x * self.frame.w / 8,
                    y * self.frame.h / 8,
                    self.frame.w / 8,
                    self.frame.h / 8,
                    if y % 2 == 0 {
                        if x % 2 == 0 {
                            self.light_colour
                        } else {
                            self.dark_colour
                        }
                    } else {
                        if x % 2 == 0 {
                            self.dark_colour
                        } else {
                            self.light_colour
                        }
                    },
                )
                .draw(buf);
            }
        }
    }

    fn draw_pieces(&self, buf: &mut FrameBuffer) {
        let locations = self.board.get_piece_locations();

        for (sq, piece) in locations {
            if self
                .taken_piece
                .map(|(taken_sq, _)| taken_sq == sq)
                .unwrap_or(false)
            {
                continue;
            }
            let (x, y) = sq_to_coords(sq, self.player == Player::Black);
            self.draw_piece(piece, x, y, 0xff, buf);
        }
    }

    fn draw_piece(&self, piece: Piece, x: u32, y: u32, alpha: u8, buf: &mut FrameBuffer) {
        let name = piece_to_sprite_name(piece);
        self.sprite_sheet.get_sprite(&name, x, y).unwrap().draw(buf);
    }

    pub fn draw_taken_piece(&self, x: u32, y: u32, buf: &mut FrameBuffer) {
        if let Some((_, piece)) = self.taken_piece {
            self.draw_piece(piece, x, y, 0x77, buf);
        }
    }
}

impl Drawable for ChessBoard {
    fn draw(&self, buf: &mut FrameBuffer) {
        self.draw_squares(buf);
        self.draw_pieces(buf);
    }
}

fn sq_to_coords(sq: SQ, rank_inv: bool) -> (u32, u32) {
    let x = sq.file() as u32;
    let y = if rank_inv {
        Rank::R8 as u32 - sq.rank() as u32
    } else {
        sq.rank() as u32
    };
    (x * 90, y * 90)
}

fn coords_to_sq(mut x: u32, mut y: u32, rank_inv: bool) -> Option<SQ> {
    x /= 90;
    y /= 90;
    if x > 7 || y > 7 {
        return None;
    }

    let file = x as u8;
    let rank = if rank_inv {
        Rank::R8 as u8 - y as u8
    } else {
        y as u8
    };
    let sq = unsafe { transmute::<u8, SQ>(file + (rank << 3)) };
    Some(sq)
}

fn piece_to_sprite_name(piece: pleco::Piece) -> &'static str {
    use pleco::Piece::*;
    match piece {
        None => todo!(),
        WhitePawn => "pawn-white",
        WhiteKnight => "knight-white",
        WhiteBishop => "bishop-white",
        WhiteRook => "rook-white",
        WhiteQueen => "queen-white",
        WhiteKing => "king-white",
        BlackPawn => "pawn-black",
        BlackKnight => "knight-black",
        BlackBishop => "bishop-black",
        BlackRook => "rook-black",
        BlackQueen => "queen-black",
        BlackKing => "king-black",
    }
}
