use std::mem::transmute;

use pleco::{core::CastleType, BitMove, Board, File, Piece, Player, Rank, SQ};

use crate::{
    bitmap::Bitmap,
    color::Color,
    drawable::{Drawable, FrameBuffer},
    rect::{Frame, Rect},
    sprite::SpriteSheet,
};

pub const INITIAL_BOARD: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

#[derive(Debug)]
pub struct ChessBoard {
    frame: Frame,
    light_colour: Color,
    dark_colour: Color,
    board: Board,
    sprite_sheet: SpriteSheet<&'static str, Bitmap>,
    player: Player,
    taken_piece: Option<(SQ, Piece)>,
}

impl ChessBoard {
    pub fn new(
        frame: Frame,
        light_colour: Color,
        dark_colour: Color,
        sprite_sheet: SpriteSheet<&'static str, Bitmap>,
        player: Player,
    ) -> Self {
        Self {
            frame,
            light_colour,
            dark_colour,
            board: Board::from_fen(INITIAL_BOARD).unwrap(),
            sprite_sheet,
            player,
            taken_piece: None,
        }
    }

    pub fn take_piece_at(&mut self, x: u32, y: u32) -> Option<pleco::Piece> {
        let sq = self.coords_to_sq(x, y)?;
        match self.board.piece_at_sq(sq) {
            Piece::None => None,
            p => {
                // Only can take your piece
                if p.player().filter(|p| *p == self.player).is_some() {
                    self.taken_piece = Some((sq, p));
                    Some(p)
                } else {
                    None
                }
            },
        }
    }

    pub fn return_taken_piece(&mut self) {
        self.taken_piece = None;
    }

    pub fn make_move_to(&mut self, dest: SQ) -> bool {
        if let Some((src, _)) = self.taken_piece {
            let all_moves = self.board.generate_moves();
            if let Some(mv) = all_moves.iter().find(|m| m.get_src() == src && m.get_dest() == dest) {
                dbg!(src.to_string(), dest.to_string(), mv.to_string());
                self.board.apply_move(*mv);
                return true;
            } else {
                if let Some(castle) = self.castle_move(src, dest) {
                    dbg!(src.to_string(), dest.to_string(), castle);
                    self.board.apply_move(castle);
                    return true;
                }
            }
        }
        self.return_taken_piece();
        false
    }

    pub fn castle_move(&self, src: SQ, dest: SQ) -> Option<BitMove> {
        const CAPTURE: u16 = 1 << 13;
        // TODO: I'm sure there's a compact way to do this
        match self.player {
            Player::White => {
                if src == SQ::E1 {
                    if dest == SQ::C1 && self.board.can_castle(self.player, CastleType::QueenSide) {
                        let rook = self.board.castling_rook_square(CastleType::QueenSide);
                        return Some(BitMove::make(BitMove::FLAG_QUEEN_CASTLE | CAPTURE, src, rook));
                    }
                    if dest == SQ::G1 && self.board.can_castle(self.player, CastleType::KingSide) {
                        let rook = self.board.castling_rook_square(CastleType::KingSide);
                        return Some(BitMove::make(BitMove::FLAG_KING_CASTLE | CAPTURE, src, rook));
                    }
                }
            },
            Player::Black => {
                if src == SQ::E8 {
                    if dest == SQ::C8 && self.board.can_castle(self.player, CastleType::QueenSide) {
                        let rook = self.board.castling_rook_square(CastleType::QueenSide);
                        let mv = BitMove::make(BitMove::FLAG_QUEEN_CASTLE | CAPTURE, src, rook);
                        return Some(mv);
                    }
                    if dest == SQ::G8 && self.board.can_castle(self.player, CastleType::KingSide) {
                        let rook = self.board.castling_rook_square(CastleType::KingSide);
                        return Some(BitMove::make(BitMove::FLAG_KING_CASTLE | CAPTURE, src, rook));
                    }
                }
            },
        }
        None
    }

    pub fn get_square(&self, x: u32, y: u32) -> Option<SQ> {
        self.coords_to_sq(x, y)
    }

    fn draw_squares(&self, buf: &mut FrameBuffer) {
        for x in 0..8 {
            for y in 0..8 {
                Rect::new(
                    Frame::new(
                        x * self.frame.w / 8,
                        y * self.frame.h / 8,
                        self.frame.w / 8,
                        self.frame.h / 8,
                    ),
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
            if self.taken_piece.map(|(taken_sq, _)| taken_sq == sq).unwrap_or(false) {
                continue;
            }
            let (x, y) = self.sq_to_coords(sq);
            self.draw_piece(piece, x, y, 0xff, buf);
        }
    }

    fn draw_piece(&self, piece: Piece, x: u32, y: u32, _: u8, buf: &mut FrameBuffer) {
        let name = piece_to_sprite_name(piece);
        self.sprite_sheet.get_sprite_drawable(&name, x, y).unwrap().draw(buf);
    }

    pub fn draw_taken_piece(&self, x: u32, y: u32, buf: &mut FrameBuffer) {
        if let Some((_, piece)) = self.taken_piece {
            self.draw_piece(piece, x, y, 0xff, buf);
        }
    }

    pub fn is_stalemate(&self) -> bool {
        self.board.stalemate()
    }

    pub fn is_checkmate(&self) -> bool {
        self.board.checkmate()
    }

    pub fn is_draw(&self) -> bool {
        // 2 Kings remaining
        self.board.count_all_pieces() <= 2
    }

    pub fn turn(&self) -> Player {
        self.board.turn()
    }

    fn sq_to_coords(&self, sq: SQ) -> (u32, u32) {
        let x = if self.player == Player::White {
            sq.file() as u32
        } else {
            File::H as u32 - sq.file() as u32
        };
        let y = if self.player == Player::White {
            Rank::R8 as u32 - sq.rank() as u32
        } else {
            sq.rank() as u32
        };
        (x * 90, y * 90)
    }

    fn coords_to_sq(&self, mut x: u32, mut y: u32) -> Option<SQ> {
        x /= 90;
        y /= 90;
        if x > 7 || y > 7 {
            return None;
        }

        let file = if self.player == Player::White {
            x as u8
        } else {
            File::H as u8 - x as u8
        };
        let rank = if self.player == Player::White {
            Rank::R8 as u8 - y as u8
        } else {
            y as u8
        };
        let sq = unsafe { transmute::<u8, SQ>(file + (rank << 3)) };
        Some(sq)
    }

    pub fn apply_move(&mut self, mv: BitMove) {
        self.board.apply_move(mv)
    }

    pub fn set_board_state(&mut self, fen: &str) -> &mut Self {
        if let Ok(b) = Board::from_fen(fen) {
            self.board = b;
        }
        self
    }
}

impl Drawable for ChessBoard {
    fn draw(&mut self, buf: &mut FrameBuffer) {
        self.draw_squares(buf);
        self.draw_pieces(buf);
    }
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

// fn invert_colour(piece: Piece) -> Piece {
//     use Piece::*;
//     match piece {
//         None => None,
//         WhitePawn => BlackPawn,
//         WhiteKnight => BlackKnight,
//         WhiteBishop => BlackBishop,
//         WhiteRook => BlackRook,
//         WhiteQueen => BlackQueen,
//         WhiteKing => BlackKing,
//         BlackPawn => WhitePawn,
//         BlackKnight => WhiteKnight,
//         BlackBishop => WhiteBishop,
//         BlackRook => WhiteRook,
//         BlackQueen => WhiteQueen,
//         BlackKing => WhiteKing,
//     }
// }
