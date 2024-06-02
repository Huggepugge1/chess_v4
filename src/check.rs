use crate::board::{Bitmap, Board, Color};
use crate::piece::PieceType;
use crate::r#move::Move;

impl Board {
    pub fn is_checking_move(&self, mov: &Move) -> bool {
        let own_pieces = self.own_pieces();
        let occupied = self.white_pieces | self.black_pieces;
        let king = self.enemy_pieces() & self.kings;

        match self.get_piece(mov.start_square).typ {
            PieceType::Pawn => match self.turn {
                Color::White => Self::black_pawn_attacks(king) & (1 << mov.end_square) > 0,
                Color::Black => Self::black_pawn_attacks(king) & (1 << mov.end_square) > 0,
                Color::Empty => unreachable!(),
            },
            PieceType::Knight => Self::knight_attacks(king) & (1 << mov.end_square) > 0,
            PieceType::Bishop => {
                Self::bishop_attacks(king, occupied, own_pieces) & (1 << mov.end_square) > 0
            }
            PieceType::Rook => {
                Self::rook_attacks(king, occupied, own_pieces) & (1 << mov.end_square) > 0
            }
            PieceType::Queen => {
                Self::queen_attacks(king, occupied, own_pieces) & (1 << mov.end_square) > 0
            }
            _ => false,
        }
    }

    pub fn get_checkers(&self) -> Bitmap {
        let enemy_pieces = self.enemy_pieces();
        let own_pieces = self.own_pieces();
        let occupied = self.white_pieces | self.black_pieces;
        let king = self.own_pieces() & self.kings;

        return if self.turn == Color::White {
            Self::white_pawn_attacks(king) & enemy_pieces & self.pawns
        } else {
            Self::black_pawn_attacks(king) & enemy_pieces & self.pawns
        } | (Self::knight_attacks(king) & enemy_pieces & self.knights)
            | (Self::bishop_attacks(king, occupied, own_pieces) & enemy_pieces & self.bishops)
            | (Self::rook_attacks(king, occupied, own_pieces) & enemy_pieces & self.rooks)
            | (Self::queen_attacks(king, occupied, own_pieces) & enemy_pieces & self.queens);
    }

    pub fn is_check(&self) -> bool {
        self.get_checkers() > 0
    }
}
