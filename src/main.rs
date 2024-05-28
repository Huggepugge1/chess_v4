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

mod pgn_to_fen;

use crate::board::*;

fn main() {
    let fen_file = "fens.txt";
    board::Board::run_perft_multi_test(fen_file, 4);
}
