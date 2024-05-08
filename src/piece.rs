use crate::board::Color;

#[derive(Debug, PartialEq)]
pub enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
    Empty,
}

#[derive(Debug)]
pub struct Piece {
    pub typ: PieceType,
    pub color: Color,
}
