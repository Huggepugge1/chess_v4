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
    let mut board = Board::from_fen("8/8/8/1k6/3Pp3/8/8/4KQ2 b - d3 0 1".into());

    board.print_board();
    println!("Moves: {:?}", board.perft(1, 1));
}
