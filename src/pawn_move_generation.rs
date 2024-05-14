use crate::board::*;
use crate::piece::*;
use crate::r#move::*;

const NOT_AFILE: Bitmap = 0xfefefefefefefefe;
const NOT_HFILE: Bitmap = 0x7f7f7f7f7f7f7f7f;

const RANK4: Bitmap = 0x00000000FF000000;
const RANK5: Bitmap = 0x000000FF00000000;

fn north_east_one(bitboard: Bitmap) -> Bitmap {
    (bitboard << 9) & NOT_AFILE
}

fn south_east_one(bitboard: Bitmap) -> Bitmap {
    (bitboard >> 7) & NOT_AFILE
}

fn south_west_one(bitboard: Bitmap) -> Bitmap {
    (bitboard >> 9) & NOT_HFILE
}

fn north_west_one(bitboard: Bitmap) -> Bitmap {
    (bitboard << 7) & NOT_HFILE
}

fn north_one(bitboard: Bitmap) -> Bitmap {
    bitboard << 8
}

fn south_one(bitboard: Bitmap) -> Bitmap {
    bitboard >> 8
}

fn white_pawn_single_pushes(white_pawns: Bitmap, empty: Bitmap) -> Bitmap {
    north_one(white_pawns) & empty
}

fn white_pawn_double_pushes(white_pawns: Bitmap, empty: Bitmap) -> Bitmap {
    north_one(white_pawn_single_pushes(white_pawns, empty)) & empty & RANK4
}

fn black_pawn_single_pushes(black_pawns: Bitmap, empty: Bitmap) -> Bitmap {
    south_one(black_pawns) & empty
}

fn black_pawn_double_pushes(black_pawns: Bitmap, empty: Bitmap) -> Bitmap {
    south_one(black_pawn_single_pushes(black_pawns, empty)) & empty & RANK5
}

fn white_pawn_pushes(white_pawns: Bitmap, empty: Bitmap) -> Bitmap {
    white_pawn_single_pushes(white_pawns, empty) | white_pawn_double_pushes(white_pawns, empty)
}

fn black_pawn_pushes(black_pawns: Bitmap, empty: Bitmap) -> Bitmap {
    black_pawn_single_pushes(black_pawns, empty) | black_pawn_double_pushes(black_pawns, empty)
}

fn white_pawn_attacks(white_pawns: Bitmap) -> Bitmap {
    north_east_one(white_pawns) | north_west_one(white_pawns)
}

fn black_pawn_attacks(black_pawns: Bitmap) -> Bitmap {
    south_east_one(black_pawns) | south_west_one(black_pawns)
}

impl Board {
    pub fn generate_pawn_moves(&self) -> Vec<Move> {
        let mut moves = Vec::new();
        let empty = !(self.white_pieces | self.black_pieces);
        let mut pawns = match self.turn {
            Color::White => self.white_pieces & self.pawns,
            Color::Black => self.black_pieces & self.pawns,
            Color::Empty => unreachable!(),
        };

        let enemy_bitboard = match self.turn {
            Color::White => {
                self.black_pieces
                    | if self.en_passant_target != -1 {
                        1 << self.en_passant_target
                    } else {
                        0
                    }
            }
            Color::Black => {
                self.white_pieces
                    | if self.en_passant_target != -1 {
                        1 << self.en_passant_target
                    } else {
                        0
                    }
            }
            Color::Empty => unreachable!(),
        };

        while pawns > 0 {
            let start_square: Square = pawns.pop_lsb();
            let pawn = 1 << start_square;

            let mut end_squares = match self.turn {
                Color::White => {
                    white_pawn_pushes(pawn, empty) | (white_pawn_attacks(pawn) & enemy_bitboard)
                }
                Color::Black => {
                    black_pawn_pushes(pawn, empty) | (black_pawn_attacks(pawn) & enemy_bitboard)
                }
                Color::Empty => unreachable!(),
            };

            while end_squares > 0 {
                let end_square: Square = end_squares.pop_lsb();

                if end_square >= 56 {
                    for promotion in [
                        PieceType::Knight,
                        PieceType::Bishop,
                        PieceType::Rook,
                        PieceType::Queen,
                    ] {
                        moves.push(Move::new(start_square, end_square, promotion));
                    }
                } else {
                    moves.push(Move::new(start_square, end_square, PieceType::Empty));
                }
            }
        }

        moves
    }
}
