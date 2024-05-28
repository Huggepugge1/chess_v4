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

impl Board {
    pub fn white_pawn_single_pushes(white_pawns: Bitmap, empty: Bitmap) -> Bitmap {
        north_one(white_pawns) & empty
    }

    pub fn white_pawn_double_pushes(white_pawns: Bitmap, empty: Bitmap) -> Bitmap {
        north_one(Self::white_pawn_single_pushes(white_pawns, empty)) & empty & RANK4
    }

    pub fn black_pawn_single_pushes(black_pawns: Bitmap, empty: Bitmap) -> Bitmap {
        south_one(black_pawns) & empty
    }

    pub fn black_pawn_double_pushes(black_pawns: Bitmap, empty: Bitmap) -> Bitmap {
        south_one(Self::black_pawn_single_pushes(black_pawns, empty)) & empty & RANK5
    }

    pub fn white_pawn_pushes(white_pawns: Bitmap, empty: Bitmap) -> Bitmap {
        Self::white_pawn_single_pushes(white_pawns, empty)
            | Self::white_pawn_double_pushes(white_pawns, empty)
    }

    pub fn black_pawn_pushes(black_pawns: Bitmap, empty: Bitmap) -> Bitmap {
        Self::black_pawn_single_pushes(black_pawns, empty)
            | Self::black_pawn_double_pushes(black_pawns, empty)
    }

    pub fn white_pawn_attacks(white_pawns: Bitmap) -> Bitmap {
        north_east_one(white_pawns) | north_west_one(white_pawns)
    }

    pub fn black_pawn_attacks(black_pawns: Bitmap) -> Bitmap {
        south_east_one(black_pawns) | south_west_one(black_pawns)
    }

    fn en_passant_check(&mut self, start_square: Square, end_square: Square) -> bool {
        let enemy_square = end_square - self.turn as Square;
        let enemy_piece = self.get_piece(enemy_square);

        self.toggle_piece(enemy_square, enemy_piece);
        self.no_side_effect_move(&Move::new(start_square, end_square, PieceType::Empty));

        if self.is_check() {
            self.toggle_piece(enemy_square, enemy_piece);
            self.no_side_effect_move(&Move::new(end_square, start_square, PieceType::Empty));
            return false;
        }
        self.toggle_piece(enemy_square, enemy_piece);
        self.no_side_effect_move(&Move::new(end_square, start_square, PieceType::Empty));

        true
    }

    pub fn generate_pawn_moves(
        &mut self,
        mut check_capture_mask: Bitmap,
        check_push_mask: Bitmap,
        pinned: Bitmap,
    ) -> Vec<Move> {
        let mut pawns = match self.turn {
            Color::White => self.white_pieces & self.pawns,
            Color::Black => self.black_pieces & self.pawns,
            Color::Empty => unreachable!(),
        };

        let mut moves = Vec::new();
        let empty = !(self.white_pieces | self.black_pieces);

        let mut enemy_pieces = self.enemy_pieces();

        if self.en_passant_target != -1
            && (check_capture_mask & (1 << (self.en_passant_target - self.turn as Square)) > 0
                || check_push_mask & (1 << (self.en_passant_target)) > 0)
        {
            enemy_pieces |= 1 << self.en_passant_target;
            check_capture_mask |= 1 << self.en_passant_target;
        }

        while pawns > 0 {
            let start_square: Square = pawns.pop_lsb();
            let pawn = 1 << start_square;

            let mut end_squares = match self.turn {
                Color::White => {
                    Self::white_pawn_pushes(pawn, empty)
                        | (Self::white_pawn_attacks(pawn) & enemy_pieces)
                }
                Color::Black => {
                    Self::black_pawn_pushes(pawn, empty)
                        | (Self::black_pawn_attacks(pawn) & enemy_pieces)
                }
                Color::Empty => unreachable!(),
            } & (check_capture_mask | check_push_mask)
                & if pinned & pawn > 0 {
                    self.get_full_pinned_ray(pawn)
                } else {
                    Bitmap::MAX
                };

            while end_squares > 0 {
                let end_square: Square = end_squares.pop_lsb();

                if end_square == self.en_passant_target
                    && !self.en_passant_check(start_square, end_square)
                {
                    continue;
                }

                if end_square >= 56 || end_square < 8 {
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
