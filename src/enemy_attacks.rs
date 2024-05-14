use crate::board::*;

impl Board {
    pub fn generate_attack_bitboard(&self) -> Bitmap {
        let enemy_pieces = self.enemy_pieces();

        let occupied = (self.white_pieces | self.black_pieces) ^ (self.own_pieces() & self.kings);
        return if self.turn == Color::White {
            Self::black_pawn_attacks(enemy_pieces & self.pawns)
        } else {
            Self::white_pawn_attacks(enemy_pieces & self.pawns)
        } | Self::knight_attacks(enemy_pieces & self.knights)
            | Self::bishop_attacks(enemy_pieces & self.bishops, occupied, enemy_pieces)
            | Self::rook_attacks(enemy_pieces & self.rooks, occupied, enemy_pieces)
            | Self::queen_attacks(enemy_pieces & self.queens, occupied, enemy_pieces)
            | Self::king_attacks(enemy_pieces & self.kings);
    }
}
