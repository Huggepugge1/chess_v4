use crate::board::Board;
use crate::piece::PieceType;

pub type Eval = i64;

impl Board {
    pub fn eval(&mut self) -> Eval {
        let white_pawns = self.white_pieces & self.pawns;
        let white_knights = self.white_pieces & self.knights;
        let white_bishops = self.white_pieces & self.bishops;
        let white_rooks = self.white_pieces & self.rooks;
        let white_queens = self.white_pieces & self.queens;

        let black_pawns = self.black_pieces & self.pawns;
        let black_knights = self.black_pieces & self.knights;
        let black_bishops = self.black_pieces & self.bishops;
        let black_rooks = self.black_pieces & self.rooks;
        let black_queens = self.black_pieces & self.queens;

        PieceType::Queen as Eval * (white_queens.count_ones() - black_queens.count_ones()) as Eval
            + PieceType::Rook as Eval
                * (white_rooks.count_ones() - black_rooks.count_ones()) as Eval
            + PieceType::Bishop as Eval
                * (white_bishops.count_ones() - black_bishops.count_ones()) as Eval
            + PieceType::Knight as Eval
                * (white_knights.count_ones() - black_knights.count_ones()) as Eval
            + PieceType::Pawn as Eval
                * (white_pawns.count_ones() - black_pawns.count_ones()) as Eval
    }
}
