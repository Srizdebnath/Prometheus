use crate::board::{Board, Color, PieceType};

const PAWN_VALUE: i32 = 100;
const KNIGHT_VALUE: i32 = 300;
const BISHOP_VALUE: i32 = 320;
const ROOK_VALUE: i32 = 500;
const QUEEN_VALUE: i32 = 900;

// Simple PSTs from white's perspective
// A1 is index 0, H8 is index 63
const PAWN_PST: [i32; 64] = [
     0,  0,  0,  0,  0,  0,  0,  0,
    50, 50, 50, 50, 50, 50, 50, 50,
    10, 10, 20, 30, 30, 20, 10, 10,
     5,  5, 10, 25, 25, 10,  5,  5,
     0,  0,  0, 20, 20,  0,  0,  0,
     5, -5,-10,  0,  0,-10, -5,  5,
     5, 10, 10,-20,-20, 10, 10,  5,
     0,  0,  0,  0,  0,  0,  0,  0,
];

const KNIGHT_PST: [i32; 64] = [
    -50,-40,-30,-30,-30,-30,-40,-50,
    -40,-20,  0,  0,  0,  0,-20,-40,
    -30,  0, 10, 15, 15, 10,  0,-30,
    -30,  5, 15, 20, 20, 15,  5,-30,
    -30,  0, 15, 20, 20, 15,  0,-30,
    -30,  5, 10, 15, 15, 10,  5,-30,
    -40,-20,  0,  5,  5,  0,-20,-40,
    -50,-40,-30,-30,-30,-30,-40,-50,
];

const BISHOP_PST: [i32; 64] = [
    -20,-10,-10,-10,-10,-10,-10,-20,
    -10,  0,  0,  0,  0,  0,  0,-10,
    -10,  0,  5, 10, 10,  5,  0,-10,
    -10,  5,  5, 10, 10,  5,  5,-10,
    -10,  0, 10, 10, 10, 10,  0,-10,
    -10, 10, 10, 10, 10, 10, 10,-10,
    -10,  5,  0,  0,  0,  0,  5,-10,
    -20,-10,-10,-10,-10,-10,-10,-20,
];

const ROOK_PST: [i32; 64] = [
     0,  0,  0,  0,  0,  0,  0,  0,
     5, 10, 10, 10, 10, 10, 10,  5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
     0,  0,  0,  5,  5,  0,  0,  0,
];

const QUEEN_PST: [i32; 64] = [
    -20,-10,-10, -5, -5,-10,-10,-20,
    -10,  0,  0,  0,  0,  0,  0,-10,
    -10,  0,  5,  5,  5,  5,  0,-10,
     -5,  0,  5,  5,  5,  5,  0, -5,
      0,  0,  5,  5,  5,  5,  0, -5,
    -10,  5,  5,  5,  5,  5,  0,-10,
    -10,  0,  5,  0,  0,  0,  0,-10,
    -20,-10,-10, -5, -5,-10,-10,-20,
];

const KING_MIDGAME_PST: [i32; 64] = [
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -20,-30,-30,-40,-40,-30,-30,-20,
    -10,-20,-20,-20,-20,-20,-20,-10,
     20, 20,  0,  0,  0,  0, 20, 20,
     20, 30, 10,  0,  0, 10, 30, 20,
];

fn flip_sq(sq: usize) -> usize {
    sq ^ 56
}

pub fn evaluate(board: &Board) -> i32 {
    let mut score = 0;

    let white_pawns = board.pieces[Color::White as usize][PieceType::Pawn as usize];
    let black_pawns = board.pieces[Color::Black as usize][PieceType::Pawn as usize];
    let white_knights = board.pieces[Color::White as usize][PieceType::Knight as usize];
    let black_knights = board.pieces[Color::Black as usize][PieceType::Knight as usize];
    let white_bishops = board.pieces[Color::White as usize][PieceType::Bishop as usize];
    let black_bishops = board.pieces[Color::Black as usize][PieceType::Bishop as usize];
    let white_rooks = board.pieces[Color::White as usize][PieceType::Rook as usize];
    let black_rooks = board.pieces[Color::Black as usize][PieceType::Rook as usize];
    let white_queens = board.pieces[Color::White as usize][PieceType::Queen as usize];
    let black_queens = board.pieces[Color::Black as usize][PieceType::Queen as usize];
    let white_king = board.pieces[Color::White as usize][PieceType::King as usize];
    let black_king = board.pieces[Color::Black as usize][PieceType::King as usize];

    // Material
    score += (white_pawns.count_ones() as i32 - black_pawns.count_ones() as i32) * PAWN_VALUE;
    score += (white_knights.count_ones() as i32 - black_knights.count_ones() as i32) * KNIGHT_VALUE;
    score += (white_bishops.count_ones() as i32 - black_bishops.count_ones() as i32) * BISHOP_VALUE;
    score += (white_rooks.count_ones() as i32 - black_rooks.count_ones() as i32) * ROOK_VALUE;
    score += (white_queens.count_ones() as i32 - black_queens.count_ones() as i32) * QUEEN_VALUE;

    // Piece Square Tables
    let mut bb = white_pawns;
    while bb != 0 {
        let sq = crate::board::pop_lsb(&mut bb);
        score += PAWN_PST[flip_sq(sq.0 as usize)]; // white pawns move "up" towards index 63, but PST usually designed from A8 down? Actually A1=0.
        // If A1=0, flip_sq(sq) flips rank. PST is visually oriented with A8 at top. So flip_sq puts A8 to 0.
    }
    
    let mut bb = black_pawns;
    while bb != 0 {
        let sq = crate::board::pop_lsb(&mut bb);
        score -= PAWN_PST[sq.0 as usize];
    }

    let mut bb = white_knights; while bb != 0 { let sq = crate::board::pop_lsb(&mut bb); score += KNIGHT_PST[flip_sq(sq.0 as usize)]; }
    let mut bb = black_knights; while bb != 0 { let sq = crate::board::pop_lsb(&mut bb); score -= KNIGHT_PST[sq.0 as usize]; }

    let mut bb = white_bishops; while bb != 0 { let sq = crate::board::pop_lsb(&mut bb); score += BISHOP_PST[flip_sq(sq.0 as usize)]; }
    let mut bb = black_bishops; while bb != 0 { let sq = crate::board::pop_lsb(&mut bb); score -= BISHOP_PST[sq.0 as usize]; }

    let mut bb = white_rooks; while bb != 0 { let sq = crate::board::pop_lsb(&mut bb); score += ROOK_PST[flip_sq(sq.0 as usize)]; }
    let mut bb = black_rooks; while bb != 0 { let sq = crate::board::pop_lsb(&mut bb); score -= ROOK_PST[sq.0 as usize]; }

    let mut bb = white_queens; while bb != 0 { let sq = crate::board::pop_lsb(&mut bb); score += QUEEN_PST[flip_sq(sq.0 as usize)]; }
    let mut bb = black_queens; while bb != 0 { let sq = crate::board::pop_lsb(&mut bb); score -= QUEEN_PST[sq.0 as usize]; }

    let mut bb = white_king; while bb != 0 { let sq = crate::board::pop_lsb(&mut bb); score += KING_MIDGAME_PST[flip_sq(sq.0 as usize)]; }
    let mut bb = black_king; while bb != 0 { let sq = crate::board::pop_lsb(&mut bb); score -= KING_MIDGAME_PST[sq.0 as usize]; }

    // Perspective (positive score means side to move is winning)
    if board.side_to_move == Color::White {
        score
    } else {
        -score
    }
}
