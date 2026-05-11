

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Color {
    White,
    Black,
}

impl Color {
    pub fn opposite(self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

pub const NUM_SQUARES: usize = 64;
pub const NUM_PIECE_TYPES: usize = 6;
pub const NUM_COLORS: usize = 2;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Square(pub u8);

impl Square {
    pub fn new(sq: u8) -> Self {
        debug_assert!(sq < 64);
        Square(sq)
    }

    pub fn rank(self) -> u8 {
        self.0 / 8
    }

    pub fn file(self) -> u8 {
        self.0 % 8
    }
}

pub type Bitboard = u64;

// Bitboard utility functions
#[inline]
pub fn set_bit(bb: &mut Bitboard, sq: Square) {
    *bb |= 1u64 << sq.0;
}

#[inline]
pub fn clear_bit(bb: &mut Bitboard, sq: Square) {
    *bb &= !(1u64 << sq.0);
}

#[inline]
pub fn get_bit(bb: Bitboard, sq: Square) -> bool {
    (bb & (1u64 << sq.0)) != 0
}

#[inline]
pub fn popcount(bb: Bitboard) -> u32 {
    bb.count_ones()
}

#[inline]
pub fn lsb(bb: Bitboard) -> Square {
    debug_assert!(bb != 0);
    Square(bb.trailing_zeros() as u8)
}

#[inline]
pub fn msb(bb: Bitboard) -> Square {
    debug_assert!(bb != 0);
    Square(63 - bb.leading_zeros() as u8)
}

#[inline]
pub fn pop_lsb(bb: &mut Bitboard) -> Square {
    let sq = lsb(*bb);
    *bb &= *bb - 1;
    sq
}

#[derive(Clone, Debug)]
pub struct Board {
    // Bitboards for each piece type and color
    pub pieces: [[Bitboard; NUM_PIECE_TYPES]; NUM_COLORS],
    // Combined bitboards for each color
    pub colors: [Bitboard; NUM_COLORS],
    // Side to move
    pub side_to_move: Color,
    // En passant square (if any)
    pub en_passant: Option<Square>,
    // Castling rights: 4 bits (White Kingside, White Queenside, Black Kingside, Black Queenside)
    pub castling_rights: u8,
    // Halfmove clock for 50-move rule
    pub halfmove_clock: u16,
    // Fullmove number
    pub fullmove_number: u16,
}

impl Board {
    pub fn empty() -> Self {
        Board {
            pieces: [[0; NUM_PIECE_TYPES]; NUM_COLORS],
            colors: [0; NUM_COLORS],
            side_to_move: Color::White,
            en_passant: None,
            castling_rights: 0,
            halfmove_clock: 0,
            fullmove_number: 1,
        }
    }

    pub fn default() -> Self {
        // Basic starting position initialization
        let mut board = Board::empty();
        // Pawns
        board.pieces[Color::White as usize][PieceType::Pawn as usize] = 0x0000_0000_0000_FF00;
        board.pieces[Color::Black as usize][PieceType::Pawn as usize] = 0x00FF_0000_0000_0000;
        
        // Knights
        board.pieces[Color::White as usize][PieceType::Knight as usize] = 0x0000_0000_0000_0042;
        board.pieces[Color::Black as usize][PieceType::Knight as usize] = 0x4200_0000_0000_0000;

        // Bishops
        board.pieces[Color::White as usize][PieceType::Bishop as usize] = 0x0000_0000_0000_0024;
        board.pieces[Color::Black as usize][PieceType::Bishop as usize] = 0x2400_0000_0000_0000;

        // Rooks
        board.pieces[Color::White as usize][PieceType::Rook as usize] = 0x0000_0000_0000_0081;
        board.pieces[Color::Black as usize][PieceType::Rook as usize] = 0x8100_0000_0000_0000;

        // Queens
        board.pieces[Color::White as usize][PieceType::Queen as usize] = 0x0000_0000_0000_0008;
        board.pieces[Color::Black as usize][PieceType::Queen as usize] = 0x0800_0000_0000_0000;

        // Kings
        board.pieces[Color::White as usize][PieceType::King as usize] = 0x0000_0000_0000_0010;
        board.pieces[Color::Black as usize][PieceType::King as usize] = 0x1000_0000_0000_0000;

        // Colors
        for pt in 0..NUM_PIECE_TYPES {
            board.colors[Color::White as usize] |= board.pieces[Color::White as usize][pt];
            board.colors[Color::Black as usize] |= board.pieces[Color::Black as usize][pt];
        }

        board.castling_rights = 0b1111; // All rights

        board
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Move(pub u16);

// Move flags
pub const MOVE_FLAG_QUIET: u16 = 0;
pub const MOVE_FLAG_DOUBLE_PAWN_PUSH: u16 = 1;
pub const MOVE_FLAG_KING_CASTLE: u16 = 2;
pub const MOVE_FLAG_QUEEN_CASTLE: u16 = 3;
pub const MOVE_FLAG_CAPTURE: u16 = 4;
pub const MOVE_FLAG_EP_CAPTURE: u16 = 5;
pub const MOVE_FLAG_KNIGHT_PROMO: u16 = 8;
pub const MOVE_FLAG_BISHOP_PROMO: u16 = 9;
pub const MOVE_FLAG_ROOK_PROMO: u16 = 10;
pub const MOVE_FLAG_QUEEN_PROMO: u16 = 11;
pub const MOVE_FLAG_KNIGHT_PROMO_CAPTURE: u16 = 12;
pub const MOVE_FLAG_BISHOP_PROMO_CAPTURE: u16 = 13;
pub const MOVE_FLAG_ROOK_PROMO_CAPTURE: u16 = 14;
pub const MOVE_FLAG_QUEEN_PROMO_CAPTURE: u16 = 15;

impl Move {
    #[inline]
    pub fn new(from: Square, to: Square, flags: u16) -> Self {
        Move((from.0 as u16) | ((to.0 as u16) << 6) | (flags << 12))
    }

    #[inline]
    pub fn from(self) -> Square {
        Square((self.0 & 0x3F) as u8)
    }

    #[inline]
    pub fn to(self) -> Square {
        Square(((self.0 >> 6) & 0x3F) as u8)
    }

    #[inline]
    pub fn flags(self) -> u16 {
        self.0 >> 12
    }

    #[inline]
    pub fn is_capture(self) -> bool {
        (self.flags() & 4) != 0
    }

    #[inline]
    pub fn is_promotion(self) -> bool {
        (self.flags() & 8) != 0
    }

    #[inline]
    pub fn promotion_piece_type(self) -> Option<PieceType> {
        if !self.is_promotion() {
            return None;
        }
        match self.flags() & 3 {
            0 => Some(PieceType::Knight),
            1 => Some(PieceType::Bishop),
            2 => Some(PieceType::Rook),
            3 => Some(PieceType::Queen),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct UndoInfo {
    pub en_passant: Option<Square>,
    pub castling_rights: u8,
    pub halfmove_clock: u16,
    pub captured_piece: Option<PieceType>,
    pub zobrist_key: u64,
}

