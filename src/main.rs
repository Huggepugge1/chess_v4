mod board;
mod r#move;
mod piece;

use crate::board::*;

fn main() {
    let mut board =
        Board::from_fen("rnbqkbnr/ppppppp1/8/8/4P2p/5N2/PPPPBPPP/RNBQK2R w KQkq - 0 4".into());

    let mov1 = r#move::Move::from_string("d2d4".into(), piece::PieceType::Empty);
    let mov2 = r#move::Move::from_string("d7d5".into(), piece::PieceType::Empty);

    board.make_move(mov1);
    board.make_move(mov2);
    board.print_board();
    println!("{:?}", board);
}
