use crate::board::Color;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
    Empty,
}

#[derive(Debug, Clone, Copy)]
pub struct Piece {
    pub typ: PieceType,
    pub color: Color,
}

impl Piece {
    pub fn new() -> Self {
        Piece {
            typ: PieceType::Empty,
            color: Color::Empty,
        }
    }
}
