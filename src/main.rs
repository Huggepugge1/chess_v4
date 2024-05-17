mod board;
mod r#move;
mod piece;

mod king_move_generation;
mod knight_move_generation;
mod move_generation;
mod pawn_move_generation;
mod sliding_pieces_move_generation;

mod enemy_attacks;

mod perft;

use crate::board::*;

fn main() {
    board::Board::from_fen("8/6b1/8/8/R1pPb1k1/4P3/P7/K7 w - - 1 2".to_string()).perft_test(
        1,
        5,
        &mut Vec::new(),
    );
}
