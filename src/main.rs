mod board;
mod r#move;
mod piece;

mod king_move_generation;
mod knight_move_generation;
mod move_generation;
mod pawn_move_generation;
mod sliding_pieces_move_generation;

mod perft;

use crate::board::*;

fn main() {
    let mut board =
        Board::from_fen("rnbqkbnr/ppppp3/8/4Pppp/8/3B1N2/PPPP1PPP/RNBQK2R w KQkq - 0 6".into());

    board.print_board();
    println!("Moves: {:?}", board.perft(2, 2));
}
