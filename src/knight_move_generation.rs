use crate::board::*;
use crate::piece::*;
use crate::r#move::*;

use const_for::const_for;

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

const fn generate_knight_attack_bitboards() -> [Bitmap; 64] {
    let mut result = [0; 64];
    const_for!(square in 0..64 => {
        result[square as usize] = knight_attacks(1 << square);
    });
    result
}

const KNIGHT_ATTACK_BITBOARDS: [Bitmap; 64] = generate_knight_attack_bitboards();

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
    pub fn knight_attacks(knights: Bitmap) -> Bitmap {
        let l1 = (knights >> 1) & NOT_HFILE;
        let l2 = (knights >> 2) & NOT_GHFILE;
        let r1 = (knights << 1) & NOT_AFILE;
        let r2 = (knights << 2) & NOT_ABFILE;
        let h1 = l1 | r1;
        let h2 = l2 | r2;
        (h1 << 16) | (h1 >> 16) | (h2 << 8) | (h2 >> 8)
    }

    pub fn generate_knight_moves(&self, check_evation_mask: Bitmap) -> Vec<Move> {
        let mut moves = Vec::new();
        let own_pieces = self.own_pieces();
        let mut knights = own_pieces & self.knights;
        while knights > 0 {
            let start_square: Square = knights.pop_lsb();
            let mut attacks =
                KNIGHT_ATTACK_BITBOARDS[start_square as usize] & !own_pieces & check_evation_mask;
            while attacks > 0 {
                let end_square: Square = attacks.pop_lsb();
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
