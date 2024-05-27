use crate::board::*;
use crate::r#move::*;

impl Board {
    pub fn generate_moves(&mut self) -> Vec<Move> {
        let mut moves = Vec::new();

        let enemy_pieces = self.enemy_pieces();
        let own_pieces = self.own_pieces();
        let occupied = self.white_pieces | self.black_pieces;
        let king = self.own_pieces() & self.kings;

        let checkers = if self.turn == Color::White {
            Self::white_pawn_attacks(king) & enemy_pieces & self.pawns
        } else {
            Self::black_pawn_attacks(king) & enemy_pieces & self.pawns
        } | (Self::knight_attacks(king) & enemy_pieces & self.knights)
            | (Self::bishop_attacks(king, occupied, own_pieces) & enemy_pieces & self.bishops)
            | (Self::rook_attacks(king, occupied, own_pieces) & enemy_pieces & self.rooks)
            | (Self::queen_attacks(king, occupied, own_pieces) & enemy_pieces & self.queens);

        let mut capture_mask = 0xFFFFFFFFFFFFFFFF;
        let mut push_mask = 0xFFFFFFFFFFFFFFFF;

        if checkers.count_ones() == 1 {
            capture_mask = checkers;

            let checker_square = checkers.lsb();
            push_mask = if self.get_piece(checker_square).is_slider() {
                Self::from_to_square(king.lsb(), checker_square)
            } else {
                0
            }
        } else if checkers.count_ones() > 1 {
            return self.generate_king_moves();
        }

        let king_square = king.lsb();
        let pinned = self.get_pinned(own_pieces);

        moves.append(&mut self.generate_pawn_moves(capture_mask, push_mask, pinned));
        moves.append(&mut self.generate_knight_moves(capture_mask | push_mask, pinned));
        moves.append(&mut self.generate_bishop_moves(capture_mask | push_mask, pinned));
        moves.append(&mut self.generate_rook_moves(capture_mask | push_mask, pinned));
        moves.append(&mut self.generate_queen_moves(capture_mask | push_mask, pinned));
        moves.append(&mut self.generate_king_moves());
        moves
    }
}
