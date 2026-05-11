

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
    // Zobrist hash key for the current position
    pub zobrist_key: u64,
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
            zobrist_key: 0,
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

        board.zobrist_key = board.generate_zobrist_key();

        board
    }

    pub fn generate_zobrist_key(&self) -> u64 {
        let mut key = 0;
        for c in 0..NUM_COLORS {
            for pt in 0..NUM_PIECE_TYPES {
                let mut bb = self.pieces[c][pt];
                while bb != 0 {
                    let sq = pop_lsb(&mut bb);
                    key ^= crate::zobrist::ZOBRIST.pieces[c][pt][sq.0 as usize];
                }
            }
        }
        if self.side_to_move == Color::Black {
            key ^= crate::zobrist::ZOBRIST.side_to_move;
        }
        key ^= crate::zobrist::ZOBRIST.castling_rights[self.castling_rights as usize];
        if let Some(ep) = self.en_passant {
            key ^= crate::zobrist::ZOBRIST.en_passant[ep.file() as usize];
        }
        key
    }

    #[inline]
    fn remove_piece(&mut self, pt: PieceType, color: Color, sq: Square) {
        clear_bit(&mut self.pieces[color as usize][pt as usize], sq);
        clear_bit(&mut self.colors[color as usize], sq);
    }

    #[inline]
    fn add_piece(&mut self, pt: PieceType, color: Color, sq: Square) {
        set_bit(&mut self.pieces[color as usize][pt as usize], sq);
        set_bit(&mut self.colors[color as usize], sq);
    }

    #[inline]
    fn move_piece(&mut self, pt: PieceType, color: Color, from: Square, to: Square) {
        self.remove_piece(pt, color, from);
        self.add_piece(pt, color, to);
        self.zobrist_key ^= crate::zobrist::ZOBRIST.pieces[color as usize][pt as usize][from.0 as usize];
        self.zobrist_key ^= crate::zobrist::ZOBRIST.pieces[color as usize][pt as usize][to.0 as usize];
    }

    pub fn piece_type_on(&self, sq: Square, color: Color) -> Option<PieceType> {
        let bit = 1u64 << sq.0;
        if (self.colors[color as usize] & bit) == 0 { return None; }
        for pt in 0..NUM_PIECE_TYPES {
            if (self.pieces[color as usize][pt] & bit) != 0 {
                return Some(match pt {
                    0 => PieceType::Pawn,
                    1 => PieceType::Knight,
                    2 => PieceType::Bishop,
                    3 => PieceType::Rook,
                    4 => PieceType::Queen,
                    _ => PieceType::King,
                });
            }
        }
        None
    }

    pub fn is_square_attacked(&self, sq: Square, by_color: Color) -> bool {
        let occ = self.colors[Color::White as usize] | self.colors[Color::Black as usize];
        
        let pawns = self.pieces[by_color as usize][PieceType::Pawn as usize];
        if (crate::attacks::pawn_attacks(by_color.opposite(), sq) & pawns) != 0 { return true; }
        
        let knights = self.pieces[by_color as usize][PieceType::Knight as usize];
        if (crate::attacks::knight_attacks(sq) & knights) != 0 { return true; }
        
        let kings = self.pieces[by_color as usize][PieceType::King as usize];
        if (crate::attacks::king_attacks(sq) & kings) != 0 { return true; }
        
        let bishops = self.pieces[by_color as usize][PieceType::Bishop as usize];
        let queens = self.pieces[by_color as usize][PieceType::Queen as usize];
        if (crate::attacks::bishop_attacks(sq, occ) & (bishops | queens)) != 0 { return true; }
        
        let rooks = self.pieces[by_color as usize][PieceType::Rook as usize];
        if (crate::attacks::rook_attacks(sq, occ) & (rooks | queens)) != 0 { return true; }
        
        false
    }

    pub fn is_in_check(&self, color: Color) -> bool {
        let kings = self.pieces[color as usize][PieceType::King as usize];
        if kings == 0 { return false; }
        let sq = lsb(kings);
        self.is_square_attacked(sq, color.opposite())
    }

    pub fn make_move(&mut self, m: Move) -> Option<UndoInfo> {
        let us = self.side_to_move;
        let them = us.opposite();
        let from = m.from();
        let to = m.to();
        let flags = m.flags();

        let pt = self.piece_type_on(from, us)?;
        let mut captured_piece = None;

        let mut undo = UndoInfo {
            en_passant: self.en_passant,
            castling_rights: self.castling_rights,
            halfmove_clock: self.halfmove_clock,
            captured_piece: None,
            zobrist_key: self.zobrist_key,
        };

        if let Some(ep) = self.en_passant {
            self.zobrist_key ^= crate::zobrist::ZOBRIST.en_passant[ep.file() as usize];
        }
        self.en_passant = None;
        self.halfmove_clock += 1;
        if pt == PieceType::Pawn {
            self.halfmove_clock = 0;
        }

        if m.is_capture() {
            self.halfmove_clock = 0;
            if flags == crate::board::MOVE_FLAG_EP_CAPTURE {
                let ep_pawn_sq = if us == Color::White { Square::new(to.0 - 8) } else { Square::new(to.0 + 8) };
                self.remove_piece(PieceType::Pawn, them, ep_pawn_sq);
                self.zobrist_key ^= crate::zobrist::ZOBRIST.pieces[them as usize][PieceType::Pawn as usize][ep_pawn_sq.0 as usize];
                captured_piece = Some(PieceType::Pawn);
            } else {
                captured_piece = self.piece_type_on(to, them);
                if let Some(cap_pt) = captured_piece {
                    self.remove_piece(cap_pt, them, to);
                    self.zobrist_key ^= crate::zobrist::ZOBRIST.pieces[them as usize][cap_pt as usize][to.0 as usize];
                }
            }
        }
        undo.captured_piece = captured_piece;

        self.move_piece(pt, us, from, to);

        if flags == crate::board::MOVE_FLAG_DOUBLE_PAWN_PUSH {
            let ep_sq = if us == Color::White { Square::new(to.0 - 8) } else { Square::new(to.0 + 8) };
            self.en_passant = Some(ep_sq);
            self.zobrist_key ^= crate::zobrist::ZOBRIST.en_passant[ep_sq.file() as usize];
        }

        if let Some(promo_pt) = m.promotion_piece_type() {
            self.remove_piece(PieceType::Pawn, us, to);
            self.zobrist_key ^= crate::zobrist::ZOBRIST.pieces[us as usize][PieceType::Pawn as usize][to.0 as usize];
            self.add_piece(promo_pt, us, to);
            self.zobrist_key ^= crate::zobrist::ZOBRIST.pieces[us as usize][promo_pt as usize][to.0 as usize];
        }

        if flags == crate::board::MOVE_FLAG_KING_CASTLE {
            if us == Color::White {
                self.move_piece(PieceType::Rook, us, Square::new(7), Square::new(5));
            } else {
                self.move_piece(PieceType::Rook, us, Square::new(63), Square::new(61));
            }
        } else if flags == crate::board::MOVE_FLAG_QUEEN_CASTLE {
            if us == Color::White {
                self.move_piece(PieceType::Rook, us, Square::new(0), Square::new(3));
            } else {
                self.move_piece(PieceType::Rook, us, Square::new(56), Square::new(59));
            }
        }

        self.zobrist_key ^= crate::zobrist::ZOBRIST.castling_rights[self.castling_rights as usize];

        let clear_castling = |rights: &mut u8, sq: u8| {
            if sq == 0 { *rights &= !2; }
            if sq == 4 { *rights &= !3; }
            if sq == 7 { *rights &= !1; }
            if sq == 56 { *rights &= !8; }
            if sq == 60 { *rights &= !12; }
            if sq == 63 { *rights &= !4; }
        };
        
        clear_castling(&mut self.castling_rights, from.0);
        clear_castling(&mut self.castling_rights, to.0);
        
        self.zobrist_key ^= crate::zobrist::ZOBRIST.castling_rights[self.castling_rights as usize];

        self.side_to_move = them;
        self.zobrist_key ^= crate::zobrist::ZOBRIST.side_to_move;

        if self.is_in_check(us) {
            self.unmake_move(m, undo);
            return None;
        }

        Some(undo)
    }

    pub fn unmake_move(&mut self, m: Move, undo: UndoInfo) {
        let us = self.side_to_move.opposite();
        let them = self.side_to_move;
        
        self.side_to_move = us;
        self.en_passant = undo.en_passant;
        self.castling_rights = undo.castling_rights;
        self.halfmove_clock = undo.halfmove_clock;

        let from = m.from();
        let to = m.to();
        let flags = m.flags();

        let pt = if m.is_promotion() {
            let promo_pt = m.promotion_piece_type().unwrap();
            self.remove_piece(promo_pt, us, to);
            self.add_piece(PieceType::Pawn, us, to);
            PieceType::Pawn
        } else {
            self.piece_type_on(to, us).unwrap()
        };

        self.move_piece(pt, us, to, from);

        if m.is_capture() {
            if flags == crate::board::MOVE_FLAG_EP_CAPTURE {
                let ep_pawn_sq = if us == Color::White { Square::new(to.0 - 8) } else { Square::new(to.0 + 8) };
                self.add_piece(PieceType::Pawn, them, ep_pawn_sq);
            } else if let Some(cap_pt) = undo.captured_piece {
                self.add_piece(cap_pt, them, to);
            }
        }

        if flags == crate::board::MOVE_FLAG_KING_CASTLE {
            if us == Color::White {
                self.move_piece(PieceType::Rook, us, Square::new(5), Square::new(7));
            } else {
                self.move_piece(PieceType::Rook, us, Square::new(61), Square::new(63));
            }
        } else if flags == crate::board::MOVE_FLAG_QUEEN_CASTLE {
            if us == Color::White {
                self.move_piece(PieceType::Rook, us, Square::new(3), Square::new(0));
            } else {
                self.move_piece(PieceType::Rook, us, Square::new(59), Square::new(56));
            }
        }

        self.zobrist_key = undo.zobrist_key;
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

