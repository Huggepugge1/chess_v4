use crate::piece::*;
use crate::r#move::*;

use const_for::const_for;

pub type Square = i32;
pub type Bitmap = u64;

pub trait BitOperations {
    #[allow(dead_code)]
    fn print(self);
    fn lsb(&self) -> Square;
    fn pop_lsb(&mut self) -> Square;
    fn msb(&self) -> Square;
}

impl BitOperations for Bitmap {
    fn print(self) {
        for i in (0..8).rev() {
            let byte = ((self >> (i * 8)) as u8).reverse_bits();
            println!("{:08b}", byte);
        }
        println!();
    }

    fn lsb(&self) -> Square {
        self.trailing_zeros() as Square
    }

    fn pop_lsb(&mut self) -> Square {
        let lsb = self.trailing_zeros() as Square;
        *self ^= 1 << lsb;
        lsb
    }

    fn msb(&self) -> Square {
        63 - self.leading_zeros() as Square
    }
}

pub trait SquareOperations {
    fn as_string(self) -> String;
    #[allow(dead_code)]
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
    pub white_king: bool,
    pub white_queen: bool,
    pub black_king: bool,
    pub black_queen: bool,
}

impl CastlingRights {
    pub fn new() -> Self {
        CastlingRights {
            white_king: false,
            white_queen: false,
            black_king: false,
            black_queen: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Irreversible {
    pub en_passant_target: Square,
    pub castling_rights: CastlingRights,
    pub half_move_clock: u8,
    pub captured_piece: Piece,
    pub mov: Move,
}

const fn generate_rectangular() -> [[Bitmap; 64]; 64] {
    let mut result = [[0; 64]; 64];

    const_for!(from in 0..64u64 => {
        const_for!(to in 0..64u64 => {
            let m1   = u64::MAX;
            let a2a7 = 0x0001010101010100;
            let b2g7 = 0x0040201008040200;
            let h1b7 = 0x0002040810204080;

            let btwn: u64  = (m1 << from) ^ (m1 << to);
            let file: u64  =   (to & 7).wrapping_sub(from   & 7);
            let rank: u64  =  ((to | 7).wrapping_sub(from)) >> 3 ;
            let mut line  =      (   (file  &  7).wrapping_sub(1)) & a2a7; /* a2a7 if same file */
            line += 2 * ((   (rank  &  7).wrapping_sub(1)) >> 58); /* b1g1 if same rank */
            line += (((rank.wrapping_sub(file)) & 15).wrapping_sub(1)) & b2g7; /* b2g7 if same diagonal */
            line += (((rank.wrapping_add(file)) & 15).wrapping_sub(1)) & h1b7; /* h1b7 if same antidiag */
            line = line.wrapping_mul(btwn & btwn.wrapping_neg()); /* mul acts like shift by smaller square */
            result[from as usize][to as usize] = line & btwn;   /* return the bits on that line in-between */
        });
    });

    result
}

const RECTANGULAR: [[Bitmap; 64]; 64] = generate_rectangular();

#[derive(Debug, Clone)]
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

    pub fen: String,
}

impl Board {
    pub fn new() -> Self {
        Board {
            white_pieces: 0xFFFF,
            black_pieces: 0xFFFF000000000000,

            pawns: 0x00FF00000000FF00,
            knights: 0x4200000000000042,
            bishops: 0x2400000000000024,
            rooks: 0x8100000000000081,
            queens: 0x0800000000000008,
            kings: 0x1000000000000010,

            castling_rights: CastlingRights::new(),
            turn: Color::White,
            en_passant_target: -1,
            half_move_clock: 0,
            full_move_clock: 0,

            irreversible: Vec::new(),
            fen: String::new(),
        }
    }

    pub fn empty_board() -> Self {
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
            fen: String::new(),
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
        } else if self.knights & (1 << square) > 0 {
            PieceType::Knight
        } else if self.bishops & (1 << square) > 0 {
            PieceType::Bishop
        } else if self.rooks & (1 << square) > 0 {
            PieceType::Rook
        } else if self.queens & (1 << square) > 0 {
            PieceType::Queen
        } else if self.kings & (1 << square) > 0 {
            PieceType::King
        } else {
            PieceType::Empty
        };

        Piece { typ, color }
    }

    pub fn own_pieces(&self) -> Bitmap {
        match self.turn {
            Color::White => self.white_pieces,
            Color::Black => self.black_pieces,
            Color::Empty => unreachable!(),
        }
    }

    pub fn enemy_pieces(&self) -> Bitmap {
        match self.turn {
            Color::White => self.black_pieces,
            Color::Black => self.white_pieces,
            Color::Empty => unreachable!(),
        }
    }

    pub fn from_to_square(from: Square, to: Square) -> Bitmap {
        RECTANGULAR[from as usize][to as usize]
    }

    pub fn obstructed(from: Square, to: Square, occupied: Bitmap) -> Bitmap {
        Self::from_to_square(from, to) & occupied
    }

    pub fn get_pinned(&self, blockers: Bitmap) -> Bitmap {
        let enemy_pieces = self.enemy_pieces();
        let occupied = self.white_pieces | self.black_pieces;
        let king = self.own_pieces() & self.kings;
        let king_square = king.lsb();

        let mut pinned = 0;
        let mut pinner = (Self::xray_rook_attacks(occupied, blockers, king_square)
            & enemy_pieces
            & (self.rooks | self.queens))
            | (Self::xray_bishop_attacks(occupied, blockers, king_square)
                & enemy_pieces
                & (self.bishops | self.queens));

        while pinner > 0 {
            let square = pinner.pop_lsb();
            pinned |= Self::obstructed(square, king_square, blockers);
        }
        pinned
    }

    pub fn get_full_pinned_ray(&self, pinned: Bitmap) -> Bitmap {
        let enemy_pieces = self.enemy_pieces();
        let occupied = self.white_pieces | self.black_pieces;
        let king = self.own_pieces() & self.kings;
        let king_square = king.lsb();
        let blockers = self.own_pieces();

        let mut pinner = (Self::xray_rook_attacks(occupied, blockers, king_square)
            & enemy_pieces
            & (self.rooks | self.queens))
            | (Self::xray_bishop_attacks(occupied, blockers, king_square)
                & enemy_pieces
                & (self.bishops | self.queens));

        while pinner > 0 {
            let square = pinner.pop_lsb();
            if pinned & Self::obstructed(square, king_square, blockers) > 0 {
                return Self::from_to_square(square, king_square) | (1 << square);
            }
        }

        0
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

    #[allow(dead_code)]
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
        let mut board = Board::empty_board();
        board.fen = fen.clone();

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
