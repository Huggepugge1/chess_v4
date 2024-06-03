use crate::board::{Board, Color};
use crate::piece::PieceType;
use crate::search::SearchMove;

use std::cmp::Ordering;
use std::ops::{Add, Div, Mul, Neg, Sub};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
pub struct Eval {
    pub score: i64,
    pub mate: Option<u16>,
}

impl Eval {
    pub const MAX: Eval = Eval {
        score: i64::MAX - 100,
        mate: None,
    };

    pub const MIN: Eval = Eval {
        score: i64::MIN + 101,
        mate: None,
    };
}

impl From<i64> for Eval {
    fn from(score: i64) -> Eval {
        Eval { score, mate: None }
    }
}

impl From<u32> for Eval {
    fn from(score: u32) -> Eval {
        Eval {
            score: score as i64,
            mate: None,
        }
    }
}

impl From<usize> for Eval {
    fn from(score: usize) -> Eval {
        Eval {
            score: score as i64,
            mate: None,
        }
    }
}

impl From<PieceType> for Eval {
    fn from(piece: PieceType) -> Eval {
        Eval {
            score: piece as i64,
            mate: None,
        }
    }
}

impl From<Color> for Eval {
    fn from(color: Color) -> Eval {
        Eval {
            score: color as i64,
            mate: None,
        }
    }
}

impl Add for Eval {
    type Output = Eval;

    fn add(self, other: Eval) -> Eval {
        Eval {
            score: self.score + other.score,
            mate: self.mate,
        }
    }
}

impl Sub for Eval {
    type Output = Eval;

    fn sub(self, other: Eval) -> Eval {
        Eval {
            score: self.score - other.score,
            mate: self.mate,
        }
    }
}

impl Neg for Eval {
    type Output = Eval;

    fn neg(self) -> Eval {
        Eval {
            score: -self.score,
            mate: self.mate,
        }
    }
}

impl Mul for Eval {
    type Output = Eval;

    fn mul(self, other: Eval) -> Eval {
        Eval {
            score: self.score * other.score,
            mate: self.mate,
        }
    }
}

impl Div for Eval {
    type Output = Eval;

    fn div(self, other: Eval) -> Eval {
        Eval {
            score: self.score / other.score,
            mate: self.mate,
        }
    }
}

impl Ord for Eval {
    fn cmp(&self, other: &Eval) -> Ordering {
        // 0 = Opponent has mate
        // 1 = You have mate
        match (self.mate, other.mate) {
            (Some(self_mate), Some(other_mate)) => {
                let self_turn = self_mate % 2;
                let other_turn = other_mate % 2;

                match (self_turn, other_turn) {
                    (0, 0) => return other_mate.cmp(&self_mate),
                    (0, 1) => return Ordering::Less,
                    (1, 0) => return Ordering::Greater,
                    (1, 1) => return self_mate.cmp(&other_mate),
                    _ => unreachable!(),
                }
            }
            (Some(mate), None) => match mate % 2 {
                0 => return Ordering::Less,
                1 => return Ordering::Greater,
                _ => unreachable!(),
            },
            (None, Some(mate)) => match mate % 2 {
                0 => return Ordering::Greater,
                1 => return Ordering::Less,
                _ => unreachable!(),
            },
            _ => (),
        }

        self.score.cmp(&other.score)
    }
}

impl Board {
    pub fn eval(&mut self, self_moves: Vec<SearchMove>) -> Eval {
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

        let en_passant_target = self.en_passant_target;

        if self_moves.len() == 0 {
            if self.is_check() {
                return Eval {
                    score: 0,
                    mate: Some(0),
                };
            } else {
                return Eval::from(0i64);
            }
        }

        self.en_passant_target = -1;

        self.change_turn();
        let other_moves = self.generate_moves().len();
        self.change_turn();

        self.en_passant_target = en_passant_target;

        (Eval::from(PieceType::Queen)
            * (Eval::from(white_queens.count_ones()) - Eval::from(black_queens.count_ones()))
            + Eval::from(PieceType::Rook)
                * (Eval::from(white_rooks.count_ones()) - Eval::from(black_rooks.count_ones()))
            + Eval::from(PieceType::Bishop)
                * (Eval::from(white_bishops.count_ones()) - Eval::from(black_bishops.count_ones()))
            + Eval::from(PieceType::Knight)
                * (Eval::from(white_knights.count_ones()) - Eval::from(black_knights.count_ones()))
            + Eval::from(PieceType::Pawn)
                * (Eval::from(white_pawns.count_ones()) - Eval::from(black_pawns.count_ones()))
            + Eval::from(10i64) * (Eval::from(self_moves.len()) - Eval::from(other_moves)))
            * Eval::from(self.turn)
            / Eval::from(8i64)
    }
}
