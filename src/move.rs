use crate::board::*;
use crate::piece::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

    pub fn null() -> Self {
        Move {
            start_square: -1,
            end_square: -1,
            promotion: PieceType::Empty,
        }
    }

    #[allow(dead_code)]
    pub fn print(&self) {
        println!("{}", self.as_string());
    }

    pub fn as_string(&self) -> String {
        let promotion: char = match self.promotion {
            PieceType::Rook => 'r',
            PieceType::Knight => 'n',
            PieceType::Bishop => 'b',
            PieceType::Queen => 'q',
            _ => ' ',
        };

        format!(
            "{}{}{}",
            self.start_square.as_string(),
            self.end_square.as_string(),
            promotion
        )
        .trim()
        .into()
    }

    pub fn reverse(&self) -> Self {
        Self::new(self.end_square, self.start_square, self.promotion)
    }

    pub fn from_string(mov: String) -> Self {
        let start_square = mov[0..2].to_string().to_square();
        let end_square = mov[2..4].to_string().to_square();

        let promotion: PieceType = match mov.len() {
            5 => match mov.chars().nth(4).unwrap() {
                'r' => PieceType::Rook,
                'n' => PieceType::Knight,
                'b' => PieceType::Bishop,
                'q' => PieceType::Queen,
                _ => PieceType::Empty,
            },
            _ => PieceType::Empty,
        };
        Move {
            start_square,
            end_square,
            promotion,
        }
    }
}

impl Board {
    pub fn is_quiet(&mut self, mov: &Move) -> bool {
        let end_piece = self.get_piece(mov.end_square);

        if end_piece.typ != PieceType::Empty {
            return false;
        }

        if mov.promotion != PieceType::Empty {
            return false;
        }

        if self.is_checking_move(mov) {
            return false;
        }

        true
    }
    pub fn promote_pawn(&mut self, mov: &Move) {
        self.pawns ^= match self.turn {
            Color::White => 1 << i32::min(mov.start_square, mov.end_square),
            Color::Black => 1 << i32::max(mov.start_square, mov.end_square),
            Color::Empty => unreachable!(),
        };

        match self.turn {
            Color::White => {
                self.zobrist ^= self.zobrist_array[ZobristPosition::WhitePawn as usize
                    + (i32::min(mov.start_square, mov.end_square)) as usize]
            }
            Color::Black => {
                self.zobrist ^= self.zobrist_array[ZobristPosition::BlackPawn as usize
                    + (i32::max(mov.start_square, mov.end_square)) as usize]
            }
            Color::Empty => unreachable!(),
        }

        let bitmap = match self.turn {
            Color::White => 1 << i32::max(mov.start_square, mov.end_square),
            Color::Black => 1 << i32::min(mov.start_square, mov.end_square),
            Color::Empty => unreachable!(),
        };

        match self.turn {
            Color::White => {
                if mov.start_square > mov.end_square {
                    self.zobrist_change_square(i32::max(mov.start_square, mov.end_square));
                }
            }
            Color::Black => {
                if mov.start_square < mov.end_square {
                    self.zobrist_change_square(i32::min(mov.start_square, mov.end_square));
                }
            }
            Color::Empty => unreachable!(),
        }

        match mov.promotion {
            PieceType::Knight => self.knights ^= bitmap,
            PieceType::Bishop => self.bishops ^= bitmap,
            PieceType::Rook => self.rooks ^= bitmap,
            PieceType::Queen => self.queens ^= bitmap,
            _ => panic!("Tried to promote to a {:?}!", mov.promotion),
        }

        match self.turn {
            Color::White => {
                if mov.start_square < mov.end_square {
                    self.zobrist_change_square(i32::max(mov.start_square, mov.end_square));
                }
            }
            Color::Black => {
                if mov.start_square > mov.end_square {
                    self.zobrist_change_square(i32::min(mov.start_square, mov.end_square));
                }
            }
            Color::Empty => unreachable!(),
        }
    }

    pub fn no_side_effect_move(&mut self, mov: &Move) {
        let piece = self.get_piece(mov.start_square);
        let bitmap = (1 << mov.start_square) | (1 << mov.end_square);

        match piece.color {
            Color::White => self.white_pieces ^= bitmap,
            Color::Black => self.black_pieces ^= bitmap,
            Color::Empty => {
                self.print_board();
                println!("fen: {}", self.fen);
                for irr in &self.irreversible {
                    print!("{}", irr.mov.as_string());
                }
                panic!("Tried to move an empty piece!: {piece:?} -> {mov:?}")
            }
        }

        match piece.typ {
            PieceType::Pawn => self.pawns ^= bitmap,
            PieceType::Knight => self.knights ^= bitmap,
            PieceType::Bishop => self.bishops ^= bitmap,
            PieceType::Rook => self.rooks ^= bitmap,
            PieceType::Queen => self.queens ^= bitmap,
            PieceType::King => self.kings ^= bitmap,
            PieceType::Empty => {
                self.print_board();
                println!("Fen: {}", self.fen);
                for irr in &self.irreversible {
                    print!("{} ", irr.mov.as_string());
                }
                panic!("Tried to move an empty piece!: {piece:?} -> {mov:?}")
            }
        }
    }

    pub fn move_piece(&mut self, mov: &Move) {
        let mut piece = self.get_piece(mov.start_square);
        let bitmap = (1 << mov.start_square) | (1 << mov.end_square);

        if mov.promotion != PieceType::Empty {
            piece.typ = PieceType::Pawn;
            self.promote_pawn(mov);
        } else {
            self.zobrist_change_square(mov.start_square);
        }

        match piece.color {
            Color::White => self.white_pieces ^= bitmap,
            Color::Black => self.black_pieces ^= bitmap,
            Color::Empty => {
                self.print_board();
                println!("fen: {}", self.fen);
                for irr in &self.irreversible {
                    print!("{}", irr.mov.as_string());
                }
                panic!("Tried to move an empty piece!: {piece:?} -> {mov:?}")
            }
        }

        match piece.typ {
            PieceType::Pawn => {
                if mov.promotion == PieceType::Empty {
                    self.pawns ^= bitmap;
                    if i32::abs(mov.start_square - mov.end_square) == 16 {
                        self.zobrist ^= self.zobrist_array
                            [ZobristPosition::EnPassant as usize + mov.end_square.file() as usize];
                        self.en_passant_target = mov.end_square - self.turn as i32;
                    }
                }
            }
            PieceType::Knight => self.knights ^= bitmap,
            PieceType::Bishop => self.bishops ^= bitmap,
            PieceType::Rook => {
                self.rooks ^= bitmap;
                match mov.start_square {
                    0 => {
                        if self.castling_rights.white_queen {
                            self.castling_rights.white_queen = false;
                            self.zobrist ^=
                                self.zobrist_array[ZobristPosition::WhiteQueenCastle as usize];
                        }
                    }
                    7 => {
                        if self.castling_rights.white_king {
                            self.castling_rights.white_king = false;
                            self.zobrist ^=
                                self.zobrist_array[ZobristPosition::WhiteKingCastle as usize];
                        }
                    }
                    56 => {
                        if self.castling_rights.black_queen {
                            self.castling_rights.black_queen = false;
                            self.zobrist ^=
                                self.zobrist_array[ZobristPosition::BlackQueenCastle as usize];
                        }
                    }
                    63 => {
                        if self.castling_rights.black_king {
                            self.castling_rights.black_king = false;
                            self.zobrist ^=
                                self.zobrist_array[ZobristPosition::BlackKingCastle as usize];
                        }
                    }
                    _ => (),
                }
            }
            PieceType::Queen => self.queens ^= bitmap,
            PieceType::King => {
                self.kings ^= bitmap;
                match self.turn {
                    Color::White => {
                        if self.castling_rights.white_queen {
                            self.castling_rights.white_queen = false;
                            self.zobrist ^=
                                self.zobrist_array[ZobristPosition::WhiteQueenCastle as usize];
                        }
                        if self.castling_rights.white_king {
                            self.castling_rights.white_king = false;
                            self.zobrist ^=
                                self.zobrist_array[ZobristPosition::WhiteKingCastle as usize];
                        }
                    }
                    Color::Black => {
                        if self.castling_rights.black_queen {
                            self.castling_rights.black_queen = false;
                            self.zobrist ^=
                                self.zobrist_array[ZobristPosition::BlackQueenCastle as usize];
                        }
                        if self.castling_rights.black_king {
                            self.castling_rights.black_king = false;
                            self.zobrist ^=
                                self.zobrist_array[ZobristPosition::BlackKingCastle as usize];
                        }
                    }
                    Color::Empty => unreachable!(),
                }
            }
            PieceType::Empty => {
                self.print_board();
                println!("Fen: {}", self.fen);
                for irr in &self.irreversible {
                    print!("{} ", irr.mov.as_string());
                }
                panic!("Tried to move an empty piece!: {piece:?} -> {mov:?}")
            }
        }

        if mov.promotion != PieceType::Empty {
            self.zobrist_change_square(mov.end_square);
        }

        match mov.end_square {
            0 => {
                if self.castling_rights.white_queen {
                    self.castling_rights.white_queen = false;
                    self.zobrist ^= self.zobrist_array[ZobristPosition::WhiteQueenCastle as usize];
                }
            }
            7 => {
                if self.castling_rights.white_king {
                    self.castling_rights.white_king = false;
                    self.zobrist ^= self.zobrist_array[ZobristPosition::WhiteKingCastle as usize];
                }
            }
            56 => {
                if self.castling_rights.black_queen {
                    self.castling_rights.black_queen = false;
                    self.zobrist ^= self.zobrist_array[ZobristPosition::BlackQueenCastle as usize];
                }
            }
            63 => {
                if self.castling_rights.black_king {
                    self.castling_rights.black_king = false;
                    self.zobrist ^= self.zobrist_array[ZobristPosition::BlackKingCastle as usize];
                }
            }
            _ => (),
        }
    }

    pub fn toggle_piece(&mut self, square: Square, piece: Piece) {
        let bitmap = 1 << square;

        match piece.color {
            Color::White => self.white_pieces ^= bitmap,
            Color::Black => self.black_pieces ^= bitmap,
            Color::Empty => panic!("Tried to toggle an empty piece!"),
        }

        match piece.typ {
            PieceType::Pawn => self.pawns ^= bitmap,
            PieceType::Knight => self.knights ^= bitmap,
            PieceType::Bishop => self.bishops ^= bitmap,
            PieceType::Rook => self.rooks ^= bitmap,
            PieceType::Queen => self.queens ^= bitmap,
            PieceType::King => self.kings ^= bitmap,
            PieceType::Empty => {
                self.print_board();
                println!("fen: {}", self.fen);
                for irr in &self.irreversible {
                    print!("{} ", irr.mov.as_string());
                }
                panic!(
                    "Tried to toggle an empty piece!: {} -> {piece:?}",
                    square.as_string()
                )
            }
        }
    }

    pub fn remove_piece(&mut self, square: Square) {
        let piece = self.get_piece(square);
        self.zobrist_change_square(square);
        self.toggle_piece(square, piece);
    }

    pub fn capture_en_passant(&mut self, mov: &Move) {
        let piece = self.get_piece(mov.start_square);
        let target_square = self.en_passant_target - self.turn as Square;

        if piece.typ == PieceType::Pawn && mov.end_square == self.en_passant_target {
            self.toggle_piece(target_square, self.get_piece(target_square));
        }
    }

    pub fn restore_en_passant(&mut self, mov: &Move) {
        let piece = self.get_piece(mov.start_square);
        let target_square = self.en_passant_target - self.turn as Square;

        if piece.typ == PieceType::Pawn && mov.end_square == self.en_passant_target {
            let captured_piece = match self.turn {
                Color::White => Piece {
                    color: Color::Black,
                    typ: PieceType::Pawn,
                },
                Color::Black => Piece {
                    color: Color::White,
                    typ: PieceType::Pawn,
                },
                Color::Empty => unreachable!(),
            };
            self.toggle_piece(target_square, captured_piece);
        }
    }

    pub fn castle(&mut self, mov: &Move) {
        let piece = self.get_piece(mov.start_square);

        if piece.typ == PieceType::King && i32::abs(mov.start_square - mov.end_square) == 2 {
            match mov.end_square {
                2 => self.move_piece(&Move::new(0, 3, PieceType::Empty)),
                6 => self.move_piece(&Move::new(7, 5, PieceType::Empty)),
                62 => self.move_piece(&Move::new(63, 61, PieceType::Empty)),
                58 => self.move_piece(&Move::new(56, 59, PieceType::Empty)),
                _ => unreachable!(),
            }
        }
    }

    pub fn un_castle(&mut self, mov: &Move) {
        let piece = self.get_piece(mov.start_square);

        if piece.typ == PieceType::King && i32::abs(mov.start_square - mov.end_square) == 2 {
            match mov.end_square {
                2 => self.move_piece(&Move::new(3, 0, PieceType::Empty)),
                6 => self.move_piece(&Move::new(5, 7, PieceType::Empty)),
                62 => self.move_piece(&Move::new(61, 63, PieceType::Empty)),
                58 => self.move_piece(&Move::new(59, 56, PieceType::Empty)),
                _ => unreachable!(),
            }
        }
    }

    pub fn change_turn(&mut self) {
        match self.turn {
            Color::White => self.turn = Color::Black,
            Color::Black => {
                self.turn = Color::White;
                self.full_move_clock += 1;
                self.zobrist ^= self.zobrist_array[ZobristPosition::SideToMove as usize];
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

    pub fn make_move(&mut self, mov: &Move) {
        let captured_piece = self.get_piece(mov.end_square);

        self.irreversible.push(Irreversible {
            en_passant_target: self.en_passant_target,
            castling_rights: self.castling_rights,
            half_move_clock: self.half_move_clock,
            captured_piece,
            mov: mov.clone(),
        });

        if captured_piece.color != Color::Empty {
            self.remove_piece(mov.end_square);
        }

        self.capture_en_passant(mov);
        self.castle(mov);
        if self.en_passant_target != -1 {
            self.zobrist ^= self.zobrist_array
                [ZobristPosition::EnPassant as usize + self.en_passant_target.file() as usize];
            self.en_passant_target = -1;
        }
        self.move_piece(mov);

        self.change_half_move_clock(&mov);
        self.change_turn();
    }

    pub fn unmake_move(&mut self, mov: &Move) {
        self.change_turn();

        let reverse_mov = mov.reverse();
        self.move_piece(&reverse_mov);
        self.un_castle(mov);

        let captured_piece;
        Irreversible {
            en_passant_target: self.en_passant_target,
            castling_rights: self.castling_rights,
            half_move_clock: self.half_move_clock,
            captured_piece,
            mov: _,
        } = self.irreversible.pop().unwrap();

        self.restore_en_passant(mov);
        if captured_piece.color != Color::Empty {
            self.toggle_piece(mov.end_square, captured_piece);
        }
    }
}
