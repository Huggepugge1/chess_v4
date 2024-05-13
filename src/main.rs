mod board;
mod r#move;
mod piece;

mod king_move_generation;
mod knight_move_generation;
mod pawn_move_generation;
mod sliding_pieces_move_generation;

use crate::board::*;

fn main() {
    let board =
        Board::from_fen("rnbqkbnr/ppppp3/8/4Pppp/8/3B1N2/PPPP1PPP/RNBQK2R w KQkq - 0 6".into());

    board.print_board();
    println!("Bishop moves: {:?}", board.generate_bishop_moves());
    println!("Rook moves: {:?}", board.generate_rook_moves());
    println!("Queen moves: {:?}", board.generate_queen_moves());
}
