mod board;
mod r#move;
mod piece;

mod king_move_generation;
mod knight_move_generation;
mod move_generation;
mod pawn_move_generation;
mod sliding_pieces_move_generation;

mod check;
mod enemy_attacks;

mod uci;

mod eval;
mod perft;
mod search;

mod pgn_to_fen;

use std::sync::atomic::AtomicBool;
use std::sync::Arc;

fn main() {
    let stopper = Arc::new(AtomicBool::new(true));
    let mut board = board::Board::new();

    loop {
        let mut line = String::new();
        std::io::stdin().read_line(&mut line).unwrap();

        board = uci::handle_input(line, board, &stopper);
    }
}
