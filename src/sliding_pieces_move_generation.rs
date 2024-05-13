use crate::board::*;
use crate::piece::*;
use crate::r#move::*;

use const_for::const_for;

#[derive(Clone, Copy)]
enum Direction {
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
const NOT_AFILE: Bitmap = 0xfefefefefefefefe;
const HFILE: Bitmap = 0x0808080808080808;
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

const fn generate_rays() -> [[Bitmap; 65]; 8] {
    let mut result = [[0; 65]; 8];
    const_for!(square in 0..64 => {
        let mut file = AFILE;
        file <<= square % 8;
        let ray = file & !(((1 << square) - 1) | (1 << square));

        result[Direction::North as usize][square as usize] = ray;

        let mut file = AFILE;
        file <<= square % 8;
        let ray = file & ((1 << square) - 1);

        result[Direction::South as usize][square as usize] = ray;

        let rank = 0xFF << (square / 8);
        let ray = rank & !(((1 << square) - 1) | (1 << square));

        result[Direction::East as usize][square as usize] = ray;

        let rank = 0xFF << (square / 8);
        let ray = rank & ((1 << square) - 1);

        result[Direction::West as usize][square as usize] = ray;

        let mut ray = 0;
        if square < 56 {
            const_for!(offset_square in 1..(square % 8 + 1) => {
                let bitmap: Bitmap = 1 << (square + (offset_square * 7));
                ray ^= bitmap;

                if bitmap.leading_zeros() < 7 {
                    break;
                }
            });
        }
        result[Direction::NorthWest as usize][square as usize] = ray & !(((1 << square) - 1) | (1 << square));

        let mut ray = 0;
        if square < 56 {
            const_for!(offset_square in 1..(8 - ((square - 1) % 8)) => {
                let bitmap: Bitmap = 1 << (square + (offset_square * 9));
                ray ^= bitmap;

                if bitmap.leading_zeros() < 9 {
                    break;
                }
            });
        }
        result[Direction::NorthEast as usize][square as usize] = ray & ((1 << square) - 1);

        let mut ray = 0;
        if square > 8 {
            const_for!(offset in 1..(square % 8 + 1) => {
                let bitmap: Bitmap = 1 << (square - (offset * 9));
                ray ^= bitmap;

                if bitmap.trailing_zeros() < 9 {
                    break;
                }

            });
        }
        result[Direction::SouthWest as usize][square as usize] = ray & !(((1 << square) - 1) | (1 << square));

        let mut ray = 0;
        if square % 8 != 7 && square >= 8 {
            const_for!(offset in 1..(9 - (square % 8)) => {
                let bitmap: Bitmap = 1 << (square - (offset * 7));
                ray ^= bitmap;

                if bitmap.trailing_zeros() < 7 {
                    break;
                }
            });
        }
        result[Direction::SouthEast as usize][square as usize] = ray & ((1 << square) - 1);
    });

    result[Direction::North as usize][64] = 0;
    result[Direction::NorthEast as usize][64] = 0;
    result[Direction::East as usize][64] = 0;
    result[Direction::SouthEast as usize][64] = 0;
    result[Direction::South as usize][64] = 0;
    result[Direction::SouthWest as usize][64] = 0;
    result[Direction::West as usize][64] = 0;
    result[Direction::NorthWest as usize][64] = 0;

    result
}

const RAYS: [[Bitmap; 65]; 8] = generate_rays();

fn mask_positive_ray(square: Square, direction: Direction, occupied: Bitmap) -> Bitmap {
    let ray = RAYS[direction as usize][square as usize];
    println!("{:064b}", 1 << square);
    println!("{ray:064b}");
    let blocked = ray & occupied;
    ray ^ RAYS[direction as usize][blocked.trailing_zeros() as usize]
}

fn mask_negative_ray(square: Square, direction: Direction, occupied: Bitmap) -> Bitmap {
    let ray = RAYS[direction as usize][square as usize];
    let blocked = ray & occupied;
    ray ^ RAYS[direction as usize][(64 - blocked.leading_zeros()) as usize]
}

impl Board {
    pub fn generate_bishop_moves(&self) -> Vec<Move> {
        let mut moves = Vec::new();
        let own_pieces = match self.turn {
            Color::White => self.white_pieces,
            Color::Black => self.black_pieces,
            Color::Empty => unreachable!(),
        };
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
            )) & !own_pieces;
            while attacks > 0 {
                let end_square: Square = attacks.pop_lsb();
                moves.push(Move::new(start_square, end_square, PieceType::Empty));
            }
        }

        moves
    }

    pub fn generate_rook_moves(&self) -> Vec<Move> {
        let mut moves = Vec::new();
        let own_pieces = match self.turn {
            Color::White => self.white_pieces,
            Color::Black => self.black_pieces,
            Color::Empty => unreachable!(),
        };
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
            )) & !own_pieces;
            while attacks > 0 {
                let end_square: Square = attacks.pop_lsb();
                moves.push(Move::new(start_square, end_square, PieceType::Empty));
            }
        }

        moves
    }

    pub fn generate_queen_moves(&self) -> Vec<Move> {
        let mut moves = Vec::new();
        let own_pieces = match self.turn {
            Color::White => self.white_pieces,
            Color::Black => self.black_pieces,
            Color::Empty => unreachable!(),
        };
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
            )) & !own_pieces;
            println!("{:064b}", attacks);
            attacks &= !own_pieces;
            println!("{:064b}", attacks);
            while attacks > 0 {
                let end_square: Square = attacks.pop_lsb();
                moves.push(Move::new(start_square, end_square, PieceType::Empty));
            }
        }

        moves
    }
}
