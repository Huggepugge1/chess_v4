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

use std::fs::File;
use std::io::prelude::*;

fn main() {
    let mut file = File::open("fens.txt").unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    let fens = contents.trim();
    for fen in fens.split("\n") {
        board::Board::from_fen(fen.to_string() + " 0 0").perft_test(1, 3, &mut Vec::new());
    }
}
