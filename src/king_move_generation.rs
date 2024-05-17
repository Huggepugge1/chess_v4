use const_for::const_for;

use crate::board::*;
use crate::piece::*;
use crate::r#move::*;

const NOT_AFILE: Bitmap = 0x7f7f7f7f7f7f7f7f;
const NOT_HFILE: Bitmap = 0xfefefefefefefefe;

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

const fn generate_king_attack_bitboards() -> [Bitmap; 64] {
    let mut result = [0; 64];
    const_for!(square in 0..64 => {
        result[square as usize] = king_attacks(1 << square);
    });
    result
}

const KING_ATTACK_BITBOARDS: [Bitmap; 64] = generate_king_attack_bitboards();

const fn king_attacks(kings: Bitmap) -> Bitmap {
    let attacks = east_one(kings) | west_one(kings);
    let new_kings = kings | attacks;
    new_kings | north_one(new_kings) | south_one(new_kings)
}

impl Board {
    pub fn king_attacks(kings: Bitmap) -> Bitmap {
        let attacks = east_one(kings) | west_one(kings);
        let new_kings = kings | attacks;
        new_kings | north_one(new_kings) | south_one(new_kings)
    }

    pub fn generate_king_moves(&self) -> Vec<Move> {
        let mut moves = Vec::new();
        let own_pieces = self.own_pieces();
        let king = own_pieces & self.kings;
        let start_square: Square = king.lsb();
        let enemy_attacks = self.generate_attack_bitboard();
        let mut attacks =
            KING_ATTACK_BITBOARDS[start_square as usize] & !(own_pieces | enemy_attacks);

        while attacks > 0 {
            let end_square: Square = attacks.pop_lsb();
            moves.push(Move::new(start_square, end_square, PieceType::Empty));
        }

        if enemy_attacks & king > 0 {
            return moves;
        }

        match self.turn {
            Color::White => {
                if self.castling_rights.white_queen
                    && (self.white_pieces | self.black_pieces | enemy_attacks) & !king & 0x1E == 0
                {
                    moves.push(Move::new(4, 2, PieceType::Empty))
                }
                if self.castling_rights.white_king
                    && (self.white_pieces | self.black_pieces | enemy_attacks) & !king & 0x70 == 0
                {
                    moves.push(Move::new(4, 6, PieceType::Empty))
                }
            }
            Color::Black => {
                if self.castling_rights.black_queen
                    && (self.white_pieces | self.black_pieces | enemy_attacks)
                        & !king
                        & 0x1E00000000000000
                        == 0
                {
                    moves.push(Move::new(60, 58, PieceType::Empty))
                }
                if self.castling_rights.black_king
                    && (self.white_pieces | self.black_pieces | enemy_attacks)
                        & !king
                        & 0x7000000000000000
                        == 0
                {
                    moves.push(Move::new(60, 62, PieceType::Empty))
                }
            }
            Color::Empty => unreachable!(),
        };

        moves
    }
}
