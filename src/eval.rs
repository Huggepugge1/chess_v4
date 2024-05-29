use crate::board::{Board, Color};
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

        let turn = self.turn;
        let en_passant_target = self.en_passant_target;

        self.en_passant_target = -1;
        self.turn = Color::White;
        let white_moves = self.generate_moves().len();
        self.turn = Color::Black;
        let black_moves = self.generate_moves().len();

        self.turn = turn;
        self.en_passant_target = en_passant_target;

        (PieceType::Queen as Eval
            * (white_queens.count_ones() as Eval - black_queens.count_ones() as Eval)
            + PieceType::Rook as Eval
                * (white_rooks.count_ones() as Eval - black_rooks.count_ones() as Eval)
            + PieceType::Bishop as Eval
                * (white_bishops.count_ones() as Eval - black_bishops.count_ones() as Eval)
            + PieceType::Knight as Eval
                * (white_knights.count_ones() as Eval - black_knights.count_ones() as Eval)
            + PieceType::Pawn as Eval
                * (white_pawns.count_ones() as Eval - black_pawns.count_ones() as Eval)
            + 10 * (white_moves as Eval - black_moves as Eval))
            * self.turn as Eval
            / 8
    }
}
