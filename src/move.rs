use crate::board::*;
use crate::piece::*;

#[derive(Debug)]
pub struct Move {
    pub start_square: Square,
    pub end_square: Square,
    pub promotion: PieceType,
}

impl Move {
    pub fn new(start_square: Square, end_square: Square, promotion: PieceType) -> Self {
        Move {
            start_square,
            end_square,
            promotion,
        }
    }

    pub fn from_string(mov: String, promotion: PieceType) -> Self {
        let start_square = mov[0..2].to_string().to_square();
        let end_square = mov[2..4].to_string().to_square();
        Move {
            start_square,
            end_square,
            promotion,
        }
    }
}

impl Board {
    pub fn promote_pawn(&mut self, mov: &Move) {
        self.pawns ^= 1 << mov.start_square;

        let bitmap = 1 << mov.end_square;

        match mov.promotion {
            PieceType::Knight => self.knights ^= bitmap,
            PieceType::Bishop => self.bishops ^= bitmap,
            PieceType::Rook => self.rooks ^= bitmap,
            PieceType::Queen => self.queens ^= bitmap,
            _ => panic!("Tried to promote to a {:?}!", mov.promotion),
        }
    }

    pub fn move_piece(&mut self, mov: &Move) {
        let piece = self.get_piece(mov.start_square);
        let bitmap = (1 << mov.start_square) | (1 << mov.end_square);

        match piece.color {
            Color::White => self.white_pieces ^= bitmap,
            Color::Black => self.black_pieces ^= bitmap,
            Color::Empty => panic!("Tried to move an empty piece!"),
        }

        match piece.typ {
            PieceType::Pawn => {
                if mov.end_square.get_rank() == 0 || mov.end_square.get_rank() == 7 {
                    self.promote_pawn(mov);
                } else {
                    self.pawns ^= bitmap;
                    if i32::abs(mov.start_square - mov.end_square) == 16 {
                        self.en_passant_target = mov.end_square - self.turn as i32;
                    }
                }
            }
            PieceType::Knight => self.knights ^= bitmap,
            PieceType::Bishop => self.bishops ^= bitmap,
            PieceType::Rook => self.rooks ^= bitmap,
            PieceType::Queen => self.queens ^= bitmap,
            PieceType::King => self.kings ^= bitmap,
            PieceType::Empty => panic!("Tried to move an empty piece!"),
        }
    }

    pub fn remove_piece(&mut self, square: Square) {
        let piece = self.get_piece(square);
        let bitmap = 1 << square;

        match piece.color {
            Color::White => self.white_pieces ^= bitmap,
            Color::Black => self.black_pieces ^= bitmap,
            Color::Empty => panic!("Tried to remove an empty piece!"),
        }

        match piece.typ {
            PieceType::Pawn => self.pawns ^= bitmap,
            PieceType::Knight => self.knights ^= bitmap,
            PieceType::Bishop => self.bishops ^= bitmap,
            PieceType::Rook => self.rooks ^= bitmap,
            PieceType::Queen => self.queens ^= bitmap,
            PieceType::King => self.kings ^= bitmap,
            PieceType::Empty => panic!("Tried to remove an empty piece!"),
        }
    }

    pub fn capture_en_passant(&mut self, mov: &Move) {
        let piece = self.get_piece(mov.start_square);

        if piece.typ == PieceType::Pawn && mov.end_square == self.en_passant_target {
            self.remove_piece(self.en_passant_target - self.turn as Square);
        }
    }

    pub fn castle(&mut self, mov: &Move) {
        let piece = self.get_piece(mov.start_square);

        if piece.typ == PieceType::King && i32::abs(mov.start_square - mov.end_square) == 2 {
            match mov.end_square {
                2 => self.move_piece(&Move::new(0, 3, PieceType::Empty)),
                6 => self.move_piece(&Move::new(7, 5, PieceType::Empty)),
                62 => self.move_piece(&Move::new(63, 61, PieceType::Empty)),
                58 => self.move_piece(&Move::new(56, 58, PieceType::Empty)),
                _ => unreachable!(),
            }
        }
    }

    pub fn change_turn(&mut self) {
        match self.turn {
            Color::White => self.turn = Color::Black,
            Color::Black => {
                self.turn = Color::White;
                self.full_move_clock = 1;
            }
            Color::Empty => unreachable!(),
        }
    }

    pub fn change_half_move_clock(&mut self, mov: &Move) {
        let start_piece = self.get_piece(mov.start_square);
        let end_piece = self.get_piece(mov.end_square);

        if end_piece.color != Color::Empty || start_piece.typ == PieceType::Pawn {
            self.half_move_clock = 0;
        } else {
            self.half_move_clock += 1;
        }
    }

    pub fn make_move(&mut self, mov: Move) {
        self.irreversible.push(Irreversible {
            en_passant_target: self.en_passant_target,
            castling_rights: self.castling_rights,
            half_move_clock: self.half_move_clock,
        });

        let end_piece = self.get_piece(mov.end_square);

        if end_piece.color != Color::Empty {
            self.remove_piece(mov.end_square);
        }

        self.capture_en_passant(&mov);
        self.castle(&mov);
        self.en_passant_target = -1;
        self.move_piece(&mov);

        self.change_half_move_clock(&mov);
        self.change_turn();
    }
}
