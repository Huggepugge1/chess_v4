use const_for::const_for;

use crate::board::*;
use crate::piece::*;
use crate::r#move::*;
use crate::sliding_pieces_move_generation::{mask_negative_ray, mask_positive_ray, Direction};

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

    pub fn ray_to_king(&self, king: Square, square: Square) -> Bitmap {
        let occupied = self.white_pieces | self.black_pieces;

        let king_bitmap = 1 << king;

        let north = mask_positive_ray(square, Direction::North, occupied);
        let east = mask_positive_ray(square, Direction::East, occupied);
        let south = mask_negative_ray(square, Direction::South, occupied);
        let west = mask_negative_ray(square, Direction::West, occupied);
        let north_west = mask_positive_ray(square, Direction::NorthWest, occupied);
        let north_east = mask_positive_ray(square, Direction::NorthEast, occupied);
        let south_east = mask_negative_ray(square, Direction::SouthEast, occupied);
        let south_west = mask_negative_ray(square, Direction::SouthWest, occupied);

        if north & king_bitmap > 0 {
            north ^ king_bitmap
        } else if east & king_bitmap > 0 {
            east ^ king_bitmap
        } else if south & king_bitmap > 0 {
            south ^ king_bitmap
        } else if west & king_bitmap > 0 {
            west ^ king_bitmap
        } else if north_west & king_bitmap > 0 {
            north_west ^ king_bitmap
        } else if north_east & king_bitmap > 0 {
            north_east ^ king_bitmap
        } else if south_east & king_bitmap > 0 {
            south_east ^ king_bitmap
        } else if south_west & king_bitmap > 0 {
            south_west ^ king_bitmap
        } else {
            0
        }
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
                    && (self.white_pieces | self.black_pieces | enemy_attacks) & 0x0E == 0
                {
                    moves.push(Move::new(4, 2, PieceType::Empty))
                }
                if self.castling_rights.white_king
                    && (self.white_pieces | self.black_pieces | enemy_attacks) & 0x60 == 0
                {
                    moves.push(Move::new(4, 6, PieceType::Empty))
                }
            }
            Color::Black => {
                if self.castling_rights.black_queen
                    && (self.white_pieces | self.black_pieces | enemy_attacks) & 0x0E00000000000000
                        == 0
                {
                    moves.push(Move::new(60, 58, PieceType::Empty))
                }
                if self.castling_rights.black_king
                    && (self.white_pieces | self.black_pieces | enemy_attacks) & 0x6000000000000000
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
