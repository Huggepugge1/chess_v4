use crate::board::Color;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PieceType {
    Pawn = 100,
    Knight = 349,
    Bishop = 350,
    Rook = 525,
    Queen = 1000,
    King = 20000,
    Empty,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Piece {
    pub typ: PieceType,
    pub color: Color,
}

impl Piece {
    pub fn is_slider(&self) -> bool {
        match self.typ {
            PieceType::Pawn => false,
            PieceType::Knight => false,
            PieceType::Bishop => true,
            PieceType::Rook => true,
            PieceType::Queen => true,
            PieceType::King => false,
            PieceType::Empty => false,
        }
    }
}
