use crate::board::*;
use crate::piece::*;
use crate::r#move::*;

const NOT_AFILE: Bitmap = 0xfefefefefefefefe;
const NOT_HFILE: Bitmap = 0x7f7f7f7f7f7f7f7f;

const fn north_one(bitboard: Bitmap) -> Bitmap {
    bitboard << 8
}

const fn south_one(bitboard: Bitmap) -> Bitmap {
    bitboard >> 8
}

const fn east_one(bitboard: Bitmap) -> Bitmap {
    (bitboard >> 1) & NOT_AFILE
}

const fn west_one(bitboard: Bitmap) -> Bitmap {
    (bitboard << 1) & NOT_HFILE
}

const fn generate_king_attack_bitboards(square: Square, mut result: [Bitmap; 64]) -> [Bitmap; 64] {
    if square < 64 {
        result = generate_king_attack_bitboards(square + 1, result);
        result[square as usize] = king_attacks(1 << square);
    }
    result
}

const KING_ATTACK_BITBOARDS: [Bitmap; 64] = generate_king_attack_bitboards(0, [0; 64]);

const fn king_attacks(kings: Bitmap) -> Bitmap {
    let attacks = east_one(kings) | west_one(kings);
    let new_kings = kings | attacks;
    new_kings | north_one(new_kings) | south_one(new_kings)
}

impl Board {
    pub fn generate_king_moves(&self) -> Vec<Move> {
        let mut moves = Vec::new();
        let own_pieces = match self.turn {
            Color::White => self.white_pieces,
            Color::Black => self.black_pieces,
            Color::Empty => unreachable!(),
        };
        let mut kings = own_pieces & self.kings;
        while kings > 0 {
            let start_square: Square = kings.trailing_zeros() as i32;
            kings ^= 1 << start_square;
            let mut attacks = KING_ATTACK_BITBOARDS[start_square as usize] & !own_pieces;
            while attacks > 0 {
                let end_square: Square = attacks.trailing_zeros() as i32;
                attacks ^= 1 << end_square;
                moves.push(Move::new(start_square, end_square, PieceType::Empty));
            }
        }

        match self.turn {
            Color::White => {
                if self.castling_rights.white_queen
                    && (self.white_pieces | self.black_pieces) & 0x0E == 0
                {
                    moves.push(Move::new(4, 2, PieceType::Empty))
                }
                if self.castling_rights.white_king
                    && (self.white_pieces | self.black_pieces) & 0x60 == 0
                {
                    moves.push(Move::new(4, 6, PieceType::Empty))
                }
            }
            Color::Black => {
                if self.castling_rights.black_queen
                    && (self.white_pieces | self.black_pieces) & 0x0E000000000000 == 0
                {
                    moves.push(Move::new(4, 2, PieceType::Empty))
                }
                if self.castling_rights.black_king
                    && (self.white_pieces | self.black_pieces) & 0x60000000000000 == 0
                {
                    moves.push(Move::new(4, 6, PieceType::Empty))
                }
            }
            Color::Empty => unreachable!(),
        };

        moves
    }
}
