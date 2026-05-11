use crate::board::{Move, Board, Color, PieceType, Square};
use crate::board::{
    MOVE_FLAG_QUIET, MOVE_FLAG_CAPTURE, MOVE_FLAG_DOUBLE_PAWN_PUSH, MOVE_FLAG_EP_CAPTURE,
    MOVE_FLAG_KING_CASTLE, MOVE_FLAG_QUEEN_CASTLE,
    MOVE_FLAG_KNIGHT_PROMO, MOVE_FLAG_BISHOP_PROMO, MOVE_FLAG_ROOK_PROMO, MOVE_FLAG_QUEEN_PROMO,
    MOVE_FLAG_KNIGHT_PROMO_CAPTURE, MOVE_FLAG_BISHOP_PROMO_CAPTURE, MOVE_FLAG_ROOK_PROMO_CAPTURE, MOVE_FLAG_QUEEN_PROMO_CAPTURE,
};
use crate::attacks::*;

pub const MAX_MOVES: usize = 256;

#[derive(Clone, Debug)]
pub struct MoveList {
    moves: [Move; MAX_MOVES],
    count: usize,
}

impl MoveList {
    #[inline]
    pub fn new() -> Self {
        MoveList {
            moves: [Move(0); MAX_MOVES],
            count: 0,
        }
    }

    #[inline]
    pub fn push(&mut self, m: Move) {
        debug_assert!(self.count < MAX_MOVES);
        self.moves[self.count] = m;
        self.count += 1;
    }

    #[inline]
    pub fn clear(&mut self) {
        self.count = 0;
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.count
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    #[inline]
    pub fn as_slice(&self) -> &[Move] {
        &self.moves[..self.count]
    }

    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [Move] {
        &mut self.moves[..self.count]
    }
}

impl std::ops::Index<usize> for MoveList {
    type Output = Move;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        debug_assert!(index < self.count);
        &self.moves[index]
    }
}

impl std::ops::IndexMut<usize> for MoveList {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        debug_assert!(index < self.count);
        &mut self.moves[index]
    }
}

impl<'a> IntoIterator for &'a MoveList {
    type Item = &'a Move;
    type IntoIter = std::slice::Iter<'a, Move>;

    fn into_iter(self) -> Self::IntoIter {
        self.as_slice().iter()
    }
}

pub fn generate_moves(board: &Board, move_list: &mut MoveList) {
    let us = board.side_to_move;
    let them = us.opposite();

    let our_pieces = board.colors[us as usize];
    let their_pieces = board.colors[them as usize];
    let all_pieces = our_pieces | their_pieces;
    let empty_squares = !all_pieces;

    // Helper macro to add moves
    macro_rules! add_moves {
        ($sq:expr, $attacks:expr, $flag_quiet:expr, $flag_capture:expr) => {
            let mut quiets = $attacks & empty_squares;
            while quiets != 0 {
                let target = crate::board::pop_lsb(&mut quiets);
                move_list.push(Move::new($sq, target, $flag_quiet));
            }
            let mut captures = $attacks & their_pieces;
            while captures != 0 {
                let target = crate::board::pop_lsb(&mut captures);
                move_list.push(Move::new($sq, target, $flag_capture));
            }
        };
    }

    // Knights
    let mut knights = board.pieces[us as usize][PieceType::Knight as usize];
    while knights != 0 {
        let sq = crate::board::pop_lsb(&mut knights);
        let attacks = knight_attacks(sq);
        add_moves!(sq, attacks, MOVE_FLAG_QUIET, MOVE_FLAG_CAPTURE);
    }

    // Bishops
    let mut bishops = board.pieces[us as usize][PieceType::Bishop as usize];
    while bishops != 0 {
        let sq = crate::board::pop_lsb(&mut bishops);
        let attacks = bishop_attacks(sq, all_pieces);
        add_moves!(sq, attacks, MOVE_FLAG_QUIET, MOVE_FLAG_CAPTURE);
    }

    // Rooks
    let mut rooks = board.pieces[us as usize][PieceType::Rook as usize];
    while rooks != 0 {
        let sq = crate::board::pop_lsb(&mut rooks);
        let attacks = rook_attacks(sq, all_pieces);
        add_moves!(sq, attacks, MOVE_FLAG_QUIET, MOVE_FLAG_CAPTURE);
    }

    // Queens
    let mut queens = board.pieces[us as usize][PieceType::Queen as usize];
    while queens != 0 {
        let sq = crate::board::pop_lsb(&mut queens);
        let attacks = queen_attacks(sq, all_pieces);
        add_moves!(sq, attacks, MOVE_FLAG_QUIET, MOVE_FLAG_CAPTURE);
    }

    // Kings (Normal moves only for now)
    let mut kings = board.pieces[us as usize][PieceType::King as usize];
    if kings != 0 {
        let sq = crate::board::pop_lsb(&mut kings);
        let attacks = king_attacks(sq);
        add_moves!(sq, attacks, MOVE_FLAG_QUIET, MOVE_FLAG_CAPTURE);
        
        // Castling logic (pseudo-legal, assuming we don't check for squares under attack here)
        if us == Color::White {
            if (board.castling_rights & 1) != 0 && (all_pieces & 0x60) == 0 {
                move_list.push(Move::new(sq, Square::new(6), MOVE_FLAG_KING_CASTLE));
            }
            if (board.castling_rights & 2) != 0 && (all_pieces & 0xE) == 0 {
                move_list.push(Move::new(sq, Square::new(2), MOVE_FLAG_QUEEN_CASTLE));
            }
        } else {
            if (board.castling_rights & 4) != 0 && (all_pieces & 0x6000000000000000) == 0 {
                move_list.push(Move::new(sq, Square::new(62), MOVE_FLAG_KING_CASTLE));
            }
            if (board.castling_rights & 8) != 0 && (all_pieces & 0x0E00000000000000) == 0 {
                move_list.push(Move::new(sq, Square::new(58), MOVE_FLAG_QUEEN_CASTLE));
            }
        }
    }

    // Pawns
    let pawns = board.pieces[us as usize][PieceType::Pawn as usize];
    let rank_3_or_6 = if us == Color::White { 0x0000000000FF0000 } else { 0x0000FF0000000000 };
    let promotion_rank = if us == Color::White { 0xFF00000000000000 } else { 0x00000000000000FF };

    // Single pushes
    let mut pushes = if us == Color::White { pawns << 8 } else { pawns >> 8 } & empty_squares;
    
    // Double pushes
    let mut double_pushes = if us == Color::White { (pushes & rank_3_or_6) << 8 } else { (pushes & rank_3_or_6) >> 8 } & empty_squares;
    
    // Add double pushes
    while double_pushes != 0 {
        let target = crate::board::pop_lsb(&mut double_pushes);
        let from = if us == Color::White { Square::new(target.0 - 16) } else { Square::new(target.0 + 16) };
        move_list.push(Move::new(from, target, MOVE_FLAG_DOUBLE_PAWN_PUSH));
    }

    // Add single pushes and promotions
    while pushes != 0 {
        let target = crate::board::pop_lsb(&mut pushes);
        let from = if us == Color::White { Square::new(target.0 - 8) } else { Square::new(target.0 + 8) };
        let target_bb = 1u64 << target.0;
        
        if (target_bb & promotion_rank) != 0 {
            move_list.push(Move::new(from, target, MOVE_FLAG_QUEEN_PROMO));
            move_list.push(Move::new(from, target, MOVE_FLAG_ROOK_PROMO));
            move_list.push(Move::new(from, target, MOVE_FLAG_BISHOP_PROMO));
            move_list.push(Move::new(from, target, MOVE_FLAG_KNIGHT_PROMO));
        } else {
            move_list.push(Move::new(from, target, MOVE_FLAG_QUIET));
        }
    }

    // Captures
    let mut capturing_pawns = pawns;
    while capturing_pawns != 0 {
        let sq = crate::board::pop_lsb(&mut capturing_pawns);
        let attacks = pawn_attacks(us, sq) & their_pieces;
        
        let mut caps = attacks;
        while caps != 0 {
            let target = crate::board::pop_lsb(&mut caps);
            let target_bb = 1u64 << target.0;
            if (target_bb & promotion_rank) != 0 {
                move_list.push(Move::new(sq, target, MOVE_FLAG_QUEEN_PROMO_CAPTURE));
                move_list.push(Move::new(sq, target, MOVE_FLAG_ROOK_PROMO_CAPTURE));
                move_list.push(Move::new(sq, target, MOVE_FLAG_BISHOP_PROMO_CAPTURE));
                move_list.push(Move::new(sq, target, MOVE_FLAG_KNIGHT_PROMO_CAPTURE));
            } else {
                move_list.push(Move::new(sq, target, MOVE_FLAG_CAPTURE));
            }
        }
    }

    // En Passant
    if let Some(ep_sq) = board.en_passant {
        let attackers = pawn_attacks(them, ep_sq) & pawns;
        let mut eps = attackers;
        while eps != 0 {
            let from_sq = crate::board::pop_lsb(&mut eps);
            move_list.push(Move::new(from_sq, ep_sq, MOVE_FLAG_EP_CAPTURE));
        }
    }
}
