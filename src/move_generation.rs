use crate::board::*;
use crate::r#move::*;

impl Board {
    pub fn generate_moves(&self) -> Vec<Move> {
        let mut moves = self.generate_pawn_moves();
        moves.append(&mut self.generate_knight_moves());
        moves.append(&mut self.generate_bishop_moves());
        moves.append(&mut self.generate_rook_moves());
        moves.append(&mut self.generate_queen_moves());
        moves.append(&mut self.generate_king_moves());

        moves
    }
}
