use crate::board::*;
use crate::piece::*;
use crate::r#move::*;

const NOT_AFILE: Bitmap = 0xfefefefefefefefe;
const NOT_ABFILE: Bitmap = 0xfcfcfcfcfcfcfcfc;
const NOT_HFILE: Bitmap = 0x7f7f7f7f7f7f7f7f;
const NOT_GHFILE: Bitmap = 0x3f3f3f3f3f3f3f3f;

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

const fn generate_knight_attack_bitboards(
    square: Square,
    mut result: [Bitmap; 64],
) -> [Bitmap; 64] {
    if square < 64 {
        result = generate_knight_attack_bitboards(square + 1, result);
        result[square as usize] = knight_attacks(1 << square);
    }
    result
}

const KNIGHT_ATTACK_BITBOARDS: [Bitmap; 64] = generate_knight_attack_bitboards(0, [0; 64]);

const fn knight_attacks(knights: Bitmap) -> Bitmap {
    let l1 = (knights >> 1) & NOT_HFILE;
    let l2 = (knights >> 2) & NOT_GHFILE;
    let r1 = (knights << 1) & NOT_AFILE;
    let r2 = (knights << 2) & NOT_ABFILE;
    let h1 = l1 | r1;
    let h2 = l2 | r2;
    (h1 << 16) | (h1 >> 16) | (h2 << 8) | (h2 >> 8)
}

impl Board {
    pub fn generate_knight_moves(&self) -> Vec<Move> {
        let mut moves = Vec::new();
        let own_pieces = match self.turn {
            Color::White => self.white_pieces,
            Color::Black => self.black_pieces,
            Color::Empty => unreachable!(),
        };
        let mut knights = own_pieces & self.knights;
        while knights > 0 {
            let start_square: Square = knights.trailing_zeros() as i32;
            knights ^= 1 << start_square;
            let mut attacks = KNIGHT_ATTACK_BITBOARDS[start_square as usize] & !own_pieces;
            while attacks > 0 {
                let end_square: Square = attacks.trailing_zeros() as i32;
                attacks ^= 1 << end_square;
                moves.push(Move {
                    start_square,
                    end_square,
                    promotion: PieceType::Empty,
                });
            }
        }
        moves
    }
}
