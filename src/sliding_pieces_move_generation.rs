use crate::board::*;
use crate::piece::*;
use crate::r#move::*;

use const_for::const_for;

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest,
}

impl Direction {
    pub fn into_array() -> [Direction; 8] {
        [
            Direction::North,
            Direction::NorthEast,
            Direction::East,
            Direction::SouthEast,
            Direction::South,
            Direction::SouthWest,
            Direction::West,
            Direction::NorthWest,
        ]
    }
}

const AFILE: Bitmap = 0x0101010101010101;

const fn generate_rays() -> [[Bitmap; 64]; 8] {
    let mut result = [[0; 64]; 8];
    const_for!(square in 0..64 => {
        let mut file = AFILE;
        file <<= square % 8;
        let ray = file & !(((1 << square) - 1) | (1 << square));

        result[Direction::North as usize][square as usize] = ray;

        let mut file = AFILE;
        file <<= square % 8;
        let ray = file & ((1 << square) - 1);

        result[Direction::South as usize][square as usize] = ray;

        let rank = 0xFF << (square / 8) * 8;
        let ray = rank & !(((1 << square) - 1) | (1 << square));

        result[Direction::East as usize][square as usize] = ray;

        let rank = 0xFF << (square / 8) * 8;
        let ray = rank & ((1 << square) - 1);

        result[Direction::West as usize][square as usize] = ray;

        let mut ray = 0;
        if square < 56 && square % 8 > 0 {
            const_for!(offset_square in 1..(square % 8 + 1) => {
                let bitmap: Bitmap = 1 << (square + (offset_square * 7));
                ray ^= bitmap;

                if bitmap.leading_zeros() < 8 {
                    break;
                }
            });
        }
        result[Direction::NorthWest as usize][square as usize] = ray;

        let mut ray = 0;
        if square < 56 && square % 8 < 7 {
            const_for!(offset_square in 1..(8 - square % 8) => {
                let bitmap: Bitmap = 1 << (square + offset_square * 9);
                ray ^= bitmap;

                if bitmap.leading_zeros() < 9 {
                    break;
                }
            });
        }
        result[Direction::NorthEast as usize][square as usize] = ray;

        let mut ray = 0;
        if square > 7 && square % 8 > 0 {
            const_for!(offset in 1..(square % 8 + 1) => {
                let bitmap: Bitmap = 1 << (square - (offset * 9));
                ray ^= bitmap;

                if bitmap.trailing_zeros() < 9 {
                    break;
                }

            });
        }
        result[Direction::SouthWest as usize][square as usize] = ray;

        let mut ray = 0;
        if square > 7 && square % 8 != 7 {
            const_for!(offset in 1..(8 - square % 8) => {
                let bitmap: Bitmap = 1 << (square - (offset * 7));
                ray ^= bitmap;

                if bitmap.trailing_zeros() < 8 {
                    break;
                }
            });
        }
        result[Direction::SouthEast as usize][square as usize] = ray;
    });

    result
}

const RAYS: [[Bitmap; 64]; 8] = generate_rays();

pub fn mask_positive_ray(square: Square, direction: Direction, occupied: Bitmap) -> Bitmap {
    let ray = RAYS[direction as usize][square as usize];
    let blocked = ray & occupied;
    ray ^ RAYS[direction as usize][(blocked | (1 << 63)).lsb() as usize]
}

pub fn mask_negative_ray(square: Square, direction: Direction, occupied: Bitmap) -> Bitmap {
    let ray = RAYS[direction as usize][square as usize];
    let blocked = ray & occupied;
    ray ^ RAYS[direction as usize][((blocked | 1).msb()) as usize]
}

impl Board {
    pub fn bishop_attacks(mut bishops: Bitmap, occupied: Bitmap, own_pieces: Bitmap) -> Bitmap {
        let mut attacks = 0;
        while bishops > 0 {
            let start_square: Square = bishops.pop_lsb();
            attacks |= (mask_positive_ray(start_square, Direction::NorthWest, occupied)
                | mask_positive_ray(start_square, Direction::NorthEast, occupied)
                | mask_negative_ray(start_square, Direction::SouthEast, occupied)
                | mask_negative_ray(start_square, Direction::SouthWest, occupied))
                & !own_pieces;
        }
        attacks
    }

    pub fn xray_bishop_attacks(occupied: Bitmap, mut blockers: Bitmap, square: Square) -> Bitmap {
        let attacks = Self::bishop_attacks(1 << square, occupied, 0);
        blockers &= attacks;
        attacks ^ Self::bishop_attacks(1 << square, occupied ^ blockers, 0)
    }

    pub fn generate_bishop_moves(
        &self,
        check_evation_mask: Bitmap,
        pinned: Bitmap,
        pinned_ray: Bitmap,
    ) -> Vec<Move> {
        let mut moves = Vec::new();
        let own_pieces = self.own_pieces();
        let mut bishops = own_pieces & self.bishops;
        while bishops > 0 {
            let start_square: Square = bishops.pop_lsb();
            let mut attacks = (mask_positive_ray(
                start_square,
                Direction::NorthWest,
                self.white_pieces | self.black_pieces,
            ) | mask_positive_ray(
                start_square,
                Direction::NorthEast,
                self.white_pieces | self.black_pieces,
            ) | mask_negative_ray(
                start_square,
                Direction::SouthEast,
                self.white_pieces | self.black_pieces,
            ) | mask_negative_ray(
                start_square,
                Direction::SouthWest,
                self.white_pieces | self.black_pieces,
            )) & !own_pieces
                & check_evation_mask;

            if (1 << start_square) & pinned > 0 {
                attacks &= pinned_ray;
            }

            while attacks > 0 {
                let end_square: Square = attacks.pop_lsb();
                moves.push(Move::new(start_square, end_square, PieceType::Empty));
            }
        }

        moves
    }

    pub fn rook_attacks(mut rooks: Bitmap, occupied: Bitmap, own_pieces: Bitmap) -> Bitmap {
        let mut attacks = 0;
        while rooks > 0 {
            let start_square: Square = rooks.pop_lsb();
            attacks |= (mask_positive_ray(start_square, Direction::North, occupied)
                | mask_positive_ray(start_square, Direction::East, occupied)
                | mask_negative_ray(start_square, Direction::South, occupied)
                | mask_negative_ray(start_square, Direction::West, occupied))
                & !own_pieces;
        }
        attacks
    }

    pub fn xray_rook_attacks(occupied: Bitmap, mut blockers: Bitmap, square: Square) -> Bitmap {
        let attacks = Self::rook_attacks(1 << square, occupied, 0);
        blockers &= attacks;
        attacks ^ Self::rook_attacks(1 << square, occupied ^ blockers, 0)
    }

    pub fn generate_rook_moves(
        &self,
        check_evation_mask: Bitmap,
        pinned: Bitmap,
        pinned_ray: Bitmap,
    ) -> Vec<Move> {
        let mut moves = Vec::new();
        let own_pieces = self.own_pieces();
        let mut rooks = own_pieces & self.rooks;
        while rooks > 0 {
            let start_square: Square = rooks.pop_lsb();
            let mut attacks = (mask_positive_ray(
                start_square,
                Direction::North,
                self.white_pieces | self.black_pieces,
            ) | mask_positive_ray(
                start_square,
                Direction::East,
                self.white_pieces | self.black_pieces,
            ) | mask_negative_ray(
                start_square,
                Direction::South,
                self.white_pieces | self.black_pieces,
            ) | mask_negative_ray(
                start_square,
                Direction::West,
                self.white_pieces | self.black_pieces,
            )) & !own_pieces
                & check_evation_mask;

            if (1 << start_square) & pinned > 0 {
                attacks &= pinned_ray;
            }

            while attacks > 0 {
                let end_square: Square = attacks.pop_lsb();
                moves.push(Move::new(start_square, end_square, PieceType::Empty));
            }
        }

        moves
    }

    pub fn queen_attacks(mut queens: Bitmap, occupied: Bitmap, own_pieces: Bitmap) -> Bitmap {
        let mut attacks = 0;
        while queens > 0 {
            let start_square: Square = queens.pop_lsb();
            attacks |= (mask_positive_ray(start_square, Direction::North, occupied)
                | mask_positive_ray(start_square, Direction::East, occupied)
                | mask_negative_ray(start_square, Direction::South, occupied)
                | mask_negative_ray(start_square, Direction::West, occupied)
                | mask_positive_ray(start_square, Direction::NorthWest, occupied)
                | mask_positive_ray(start_square, Direction::NorthEast, occupied)
                | mask_negative_ray(start_square, Direction::SouthEast, occupied)
                | mask_negative_ray(start_square, Direction::SouthWest, occupied))
                & !own_pieces;
        }
        attacks
    }

    pub fn generate_queen_moves(
        &self,
        check_evation_mask: Bitmap,
        pinned: Bitmap,
        pinned_ray: Bitmap,
    ) -> Vec<Move> {
        let mut moves = Vec::new();
        let own_pieces = self.own_pieces();
        let mut queens = own_pieces & self.queens;
        while queens > 0 {
            let start_square: Square = queens.pop_lsb();
            let mut attacks = (mask_positive_ray(
                start_square,
                Direction::North,
                self.white_pieces | self.black_pieces,
            ) | mask_positive_ray(
                start_square,
                Direction::East,
                self.white_pieces | self.black_pieces,
            ) | mask_negative_ray(
                start_square,
                Direction::South,
                self.white_pieces | self.black_pieces,
            ) | mask_negative_ray(
                start_square,
                Direction::West,
                self.white_pieces | self.black_pieces,
            ) | mask_positive_ray(
                start_square,
                Direction::NorthWest,
                self.white_pieces | self.black_pieces,
            ) | mask_positive_ray(
                start_square,
                Direction::NorthEast,
                self.white_pieces | self.black_pieces,
            ) | mask_negative_ray(
                start_square,
                Direction::SouthEast,
                self.white_pieces | self.black_pieces,
            ) | mask_negative_ray(
                start_square,
                Direction::SouthWest,
                self.white_pieces | self.black_pieces,
            )) & !own_pieces
                & check_evation_mask;

            if (1 << start_square) & pinned > 0 {
                attacks &= pinned_ray;
            }

            while attacks > 0 {
                let end_square: Square = attacks.pop_lsb();
                moves.push(Move::new(start_square, end_square, PieceType::Empty));
            }
        }

        moves
    }
}
