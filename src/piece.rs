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
