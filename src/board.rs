use crate::piece::*;
use crate::r#move::*;

pub type Square = i32;
pub type Bitmap = u64;

pub trait SquareOperations {
    fn as_string(self) -> String;
    fn get_rank(self) -> u8;
}

pub trait ToSquare {
    fn to_square(self) -> Square;
}

impl SquareOperations for Square {
    fn as_string(self) -> String {
        (('a' as Square + (self % 8)) as u8 as char).to_string()
            + &(('1' as Square + (self / 8)) as u8 as char).to_string()
    }

    fn get_rank(self) -> u8 {
        (self / 8) as u8
    }
}

impl ToSquare for String {
    fn to_square(self) -> Square {
        (self.chars().nth(0).unwrap() as Square - 'a' as Square)
            + (self.chars().nth(1).unwrap() as Square - '1' as Square) * 8
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Color {
    White = 8,
    Black = -8,
    Empty,
}

#[derive(Debug, Clone, Copy)]
pub struct CastlingRights {
    white_king: bool,
    white_queen: bool,
    black_king: bool,
    black_queen: bool,
}

impl CastlingRights {
    pub fn new() -> Self {
        CastlingRights {
            white_king: true,
            white_queen: true,
            black_king: true,
            black_queen: true,
        }
    }
}

#[derive(Debug)]
pub struct Irreversible {
    pub en_passant_target: Square,
    pub castling_rights: CastlingRights,
    pub half_move_clock: u8,
}

#[derive(Debug)]
pub struct Board {
    pub white_pieces: Bitmap,
    pub black_pieces: Bitmap,

    pub pawns: Bitmap,
    pub knights: Bitmap,
    pub bishops: Bitmap,
    pub rooks: Bitmap,
    pub queens: Bitmap,
    pub kings: Bitmap,
    pub en_passant_target: Square,

    pub turn: Color,
    pub castling_rights: CastlingRights,

    pub half_move_clock: u8,
    pub full_move_clock: u8,

    pub irreversible: Vec<Irreversible>,
}

impl Board {
    pub fn new() -> Self {
        Board {
            white_pieces: 0,
            black_pieces: 0,

            pawns: 0,
            knights: 0,
            bishops: 0,
            rooks: 0,
            queens: 0,
            kings: 0,

            castling_rights: CastlingRights::new(),
            turn: Color::White,
            en_passant_target: -1,
            half_move_clock: 0,
            full_move_clock: 0,

            irreversible: Vec::new(),
        }
    }

    pub fn get_piece(&self, square: Square) -> Piece {
        let color = if self.white_pieces & (1 << square) > 0 {
            Color::White
        } else if self.black_pieces & (1 << square) > 0 {
            Color::Black
        } else {
            Color::Empty
        };

        let typ = if self.pawns & (1 << square) > 0 {
            PieceType::Pawn
        } else if self.rooks & (1 << square) > 0 {
            PieceType::Rook
        } else if self.knights & (1 << square) > 0 {
            PieceType::Knight
        } else if self.bishops & (1 << square) > 0 {
            PieceType::Bishop
        } else if self.queens & (1 << square) > 0 {
            PieceType::Queen
        } else if self.kings & (1 << square) > 0 {
            PieceType::King
        } else {
            PieceType::Empty
        };

        Piece { typ, color }
    }

    pub fn print_board(&self) {
        println!(" --- --- --- --- --- --- --- ---");
        for i in 0..8 {
            print!("|");
            for j in 0..8 {
                print!(
                    " {} |",
                    Self::converter(self.get_piece(63 - ((i * 8) + (7 - j))))
                );
            }
            println!("");
            println!(" --- --- --- --- --- --- --- ---");
        }
    }

    pub fn converter(piece: Piece) -> char {
        let chr: char = match piece.typ {
            PieceType::Pawn => 'p',
            PieceType::Rook => 'r',
            PieceType::Knight => 'n',
            PieceType::Bishop => 'b',
            PieceType::Queen => 'q',
            PieceType::King => 'k',
            PieceType::Empty => '_',
        };

        match piece.color {
            Color::White => chr.to_ascii_uppercase(),
            Color::Black => chr,
            Color::Empty => chr,
        }
    }

    pub fn print_move(mov: &Move) -> String {
        let promotion: char = match mov.promotion {
            PieceType::Rook => 'r',
            PieceType::Knight => 'n',
            PieceType::Bishop => 'b',
            PieceType::Queen => 'q',
            _ => ' ',
        };

        format!(
            "{}{}{}",
            mov.start_square.as_string(),
            mov.end_square.as_string(),
            promotion
        )
    }

    pub fn from_fen(fen: String) -> Self {
        let mut board = Board::new();
        let mut parts = fen.split(" ");
        let pieces = parts.next().unwrap();
        let turn = parts.next().unwrap();
        let castling = parts.next().unwrap();
        let en_passant = parts.next().unwrap();
        let halfmove_clock = parts.next().unwrap();
        let fullmove_clock = parts.next().unwrap();
        let mut pos: Square = 56;

        for piece in pieces.chars() {
            if piece == '/' {
                continue;
            } else if piece.is_digit(10) {
                pos += piece as Square - '0' as Square;
            } else {
                if piece.is_uppercase() {
                    board.white_pieces |= 1 << pos;
                    match piece {
                        'P' => board.pawns |= 1 << pos,
                        'R' => board.rooks |= 1 << pos,
                        'N' => board.knights |= 1 << pos,
                        'B' => board.bishops |= 1 << pos,
                        'Q' => board.queens |= 1 << pos,
                        'K' => board.kings |= 1 << pos,
                        _ => (),
                    }
                } else {
                    board.black_pieces |= 1 << pos;
                    match piece {
                        'p' => board.pawns |= 1 << pos,
                        'r' => board.rooks |= 1 << pos,
                        'n' => board.knights |= 1 << pos,
                        'b' => board.bishops |= 1 << pos,
                        'q' => board.queens |= 1 << pos,
                        'k' => board.kings |= 1 << pos,
                        _ => (),
                    }
                }
                pos += 1;
            }
            if pos > 8 && pos % 8 == 0 {
                pos -= 16;
            }
        }

        match turn {
            "w" => board.turn = Color::White,
            "b" => board.turn = Color::Black,
            _ => panic!("Fen needs a turn!"),
        }

        for castling_right in castling.chars() {
            match castling_right {
                'K' => board.castling_rights.white_king = true,
                'Q' => board.castling_rights.white_queen = true,
                'k' => board.castling_rights.black_king = true,
                'q' => board.castling_rights.black_queen = true,
                _ => (),
            }
        }

        if en_passant != "-" {
            board.en_passant_target = en_passant.to_string().to_square();
        } else {
            board.en_passant_target = -1;
        }

        board.half_move_clock = halfmove_clock.parse().unwrap();
        board.full_move_clock = fullmove_clock.parse().unwrap();

        board
    }
}
