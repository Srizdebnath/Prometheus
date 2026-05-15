use crate::attacks;
use crate::board::{Bitboard, Board, Color, PieceType, popcount};

const PAWN_MG_VAL: i32 = 82;
const PAWN_EG_VAL: i32 = 94;
const KNIGHT_MG_VAL: i32 = 337;
const KNIGHT_EG_VAL: i32 = 281;
const BISHOP_MG_VAL: i32 = 365;
const BISHOP_EG_VAL: i32 = 297;
const ROOK_MG_VAL: i32 = 477;
const ROOK_EG_VAL: i32 = 512;
const QUEEN_MG_VAL: i32 = 1025;
const QUEEN_EG_VAL: i32 = 936;

// Phase weightings for Tapered Evaluation
const PHASE_WEIGHT_KNIGHT: i32 = 1;
const PHASE_WEIGHT_BISHOP: i32 = 1;
const PHASE_WEIGHT_ROOK: i32 = 2;
const PHASE_WEIGHT_QUEEN: i32 = 4;
const TOTAL_PHASE: i32 = 24;

// PeSTO inspired simplified PSTs (A1 = 0)
const PAWN_MG: [i32; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, 98, 134, 61, 95, 68, 126, 34, -11, -6, 7, 26, 31, 65, 56, 25, -20, -14,
    13, 6, 21, 23, 12, 17, -23, -27, -2, -5, 12, 17, 6, 10, -25, -26, -4, -4, -10, 3, 3, 33, -12,
    -35, -1, -20, -23, -15, 24, 38, -22, 0, 0, 0, 0, 0, 0, 0, 0,
];

const PAWN_EG: [i32; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, 178, 173, 158, 134, 147, 132, 165, 187, 94, 100, 85, 67, 56, 53, 82,
    84, 32, 24, 13, 5, -2, 4, 17, 17, 13, 9, -3, -7, -7, -8, 3, -1, 4, 7, -6, 1, 0, -5, -1, -8, 13,
    8, 8, 10, 13, 0, 2, -7, 0, 0, 0, 0, 0, 0, 0, 0,
];

const KNIGHT_MG: [i32; 64] = [
    -167, -89, -34, -49, 61, -97, -15, -107, -73, -41, 72, 36, 23, 62, 7, -17, -47, 60, 37, 65, 84,
    129, 73, 44, -9, 17, 19, 53, 37, 69, 18, 22, -13, 4, 16, 13, 28, 19, 21, -8, -23, -9, 12, 10,
    19, 17, 25, -16, -29, -53, -12, -3, -1, 18, -14, -19, -105, -21, -58, -33, -17, -28, -19, -23,
];

const KNIGHT_EG: [i32; 64] = [
    -58, -38, -13, -28, -31, -27, -63, -99, -25, -8, -25, -2, -9, -25, -24, -52, -24, -20, 10, 9,
    -1, -9, -19, -41, -17, 3, 22, 22, 22, 11, 8, -18, -18, -6, 16, 25, 16, 17, 4, -18, -23, -3, -1,
    15, 10, -3, -20, -22, -42, -20, -10, -5, -2, -20, -23, -44, -29, -51, -23, -15, -22, -18, -50,
    -64,
];

const BISHOP_MG: [i32; 64] = [
    -29, 4, -82, -37, -25, -42, 7, -8, -26, 16, -18, -13, 30, 59, 18, -47, -16, 37, 43, 40, 35, 50,
    37, -2, -4, 5, 19, 50, 37, 37, 7, -2, -6, 13, 13, 26, 34, 12, 10, 4, 0, 15, 15, 15, 14, 27, 18,
    10, 4, 15, 16, 0, 7, 21, 33, 1, -33, -3, -14, -21, -13, -12, -39, -21,
];

const BISHOP_EG: [i32; 64] = [
    -14, -21, -11, -8, -7, -9, -17, -24, -8, -4, 7, -12, -3, -13, -4, -14, 2, -8, 0, -1, -2, 6, 0,
    4, -3, 9, 12, 9, 14, 10, 3, 2, -6, 3, 13, 19, 7, 10, -3, -9, -12, -3, 8, 10, 13, 3, -7, -15,
    -14, -18, -7, -1, 4, -9, -15, -27, -23, -9, -23, -5, -9, -16, -5, -17,
];

const ROOK_MG: [i32; 64] = [
    32, 42, 32, 51, 63, 9, 31, 43, 27, 32, 58, 62, 80, 67, 26, 44, -5, 19, 26, 36, 17, 45, 61, 16,
    -24, -11, 7, 26, 24, 35, -8, -20, -36, -26, -12, -1, 9, -7, 6, -23, -45, -25, -16, -17, 3, 0,
    -5, -33, -44, -16, -20, -9, -1, 11, -6, -71, -19, -13, 1, 17, 16, 7, -37, -26,
];

const ROOK_EG: [i32; 64] = [
    13, 10, 18, 15, 12, 12, 8, 5, 11, 13, 13, 11, -3, 3, 8, 3, 7, 7, 7, 5, 4, -3, -5, -3, 4, 3, 13,
    1, 2, 1, -1, 2, 3, 5, 8, 4, -5, -6, -8, -11, -4, 0, -5, -1, -7, -12, -8, -16, -6, -6, 0, 2, -9,
    -9, -11, -3, -9, 2, 3, -1, -5, -13, 4, -20,
];

const QUEEN_MG: [i32; 64] = [
    -28, 0, 29, 12, 59, 44, 43, 45, -24, -39, -5, 1, -16, 57, 28, 54, -13, -17, 7, 8, 29, 56, 47,
    57, -27, -27, -16, -16, -1, 17, -2, 1, -9, -26, -9, -10, -2, -4, 3, -3, -14, 2, -11, -2, -5, 2,
    14, 5, -35, -8, 11, 2, 8, 15, -3, 1, -1, -18, -9, 10, -15, -25, -31, -50,
];

const QUEEN_EG: [i32; 64] = [
    -9, 22, 22, 27, 27, 19, 10, 20, -17, 20, 32, 41, 58, 25, 30, 0, -20, 6, 9, 49, 47, 35, 19, 9,
    3, 22, 24, 45, 57, 40, 57, 36, -18, 28, 19, 47, 31, 34, 12, 11, 16, 14, 28, 20, 43, 20, 15, 14,
    22, 33, 3, 12, 24, 4, 14, 21, -4, -7, -11, -12, -12, -14, 22, -8,
];

const KING_MG: [i32; 64] = [
    -65, 23, 16, -15, -56, -34, 2, 13, 29, -1, -20, -7, -8, -4, -38, -29, -9, 24, 2, -16, -20, 6,
    22, -22, -17, -20, -12, -27, -30, -25, -14, -36, -49, -1, -27, -39, -46, -44, -33, -51, -14,
    -14, -22, -46, -44, -30, -15, -27, 1, 7, -8, -64, -43, -16, 9, 8, -15, 36, 12, -54, 8, -28, 24,
    14,
];

const KING_EG: [i32; 64] = [
    -74, -35, -18, -18, -11, 15, 4, -17, -12, 17, 14, 17, 17, 38, 23, 11, 10, 17, 23, 15, 20, 45,
    44, 13, -8, 22, 24, 27, 26, 33, 26, 3, -18, -4, 21, 24, 27, 23, 9, -11, -19, -3, 11, 21, 23,
    16, 7, -9, -27, -11, 4, 13, 14, 4, -5, -17, -53, -34, -21, -11, -28, -14, -24, -43,
];

fn flip_sq(sq: usize, color: Color) -> usize {
    if color == Color::White { sq ^ 56 } else { sq }
}

// File masks for pawn structure evaluation
const FILE_MASK: [Bitboard; 8] = [
    0x0101010101010101,
    0x0202020202020202,
    0x0404040404040404,
    0x0808080808080808,
    0x1010101010101010,
    0x2020202020202020,
    0x4040404040404040,
    0x8080808080808080,
];

const ADJACENT_FILE_MASK: [Bitboard; 8] = [
    FILE_MASK[1],
    FILE_MASK[0] | FILE_MASK[2],
    FILE_MASK[1] | FILE_MASK[3],
    FILE_MASK[2] | FILE_MASK[4],
    FILE_MASK[3] | FILE_MASK[5],
    FILE_MASK[4] | FILE_MASK[6],
    FILE_MASK[5] | FILE_MASK[7],
    FILE_MASK[6],
];

// Rank masks
const RANK_MASK: [Bitboard; 8] = [
    0x00000000000000FF,
    0x000000000000FF00,
    0x0000000000FF0000,
    0x00000000FF000000,
    0x000000FF00000000,
    0x0000FF0000000000,
    0x00FF000000000000,
    0xFF00000000000000,
];

// Passed pawn masks: squares in front of a pawn on adjacent + same files
fn passed_pawn_mask(sq: usize, color: Color) -> Bitboard {
    let file = sq % 8;
    let rank = sq / 8;
    let file_mask = FILE_MASK[file] | ADJACENT_FILE_MASK[file];

    match color {
        Color::White => {
            // All ranks above the pawn
            let mut mask = 0u64;
            for r in (rank + 1)..8 {
                mask |= RANK_MASK[r];
            }
            file_mask & mask
        }
        Color::Black => {
            let mut mask = 0u64;
            for r in 0..rank {
                mask |= RANK_MASK[r];
            }
            file_mask & mask
        }
    }
}

// Bonus/penalty values for structural features
const DOUBLED_PAWN_PENALTY_MG: i32 = -10;
const DOUBLED_PAWN_PENALTY_EG: i32 = -20;
const ISOLATED_PAWN_PENALTY_MG: i32 = -15;
const ISOLATED_PAWN_PENALTY_EG: i32 = -20;
const PASSED_PAWN_BONUS_MG: [i32; 8] = [0, 5, 10, 20, 35, 60, 100, 0];
const PASSED_PAWN_BONUS_EG: [i32; 8] = [0, 10, 20, 40, 70, 120, 200, 0];

// Bishop pair bonus
const BISHOP_PAIR_BONUS_MG: i32 = 30;
const BISHOP_PAIR_BONUS_EG: i32 = 50;

// Rook on open/semi-open file
const ROOK_OPEN_FILE_MG: i32 = 20;
const ROOK_OPEN_FILE_EG: i32 = 10;
const ROOK_SEMI_OPEN_FILE_MG: i32 = 10;
const ROOK_SEMI_OPEN_FILE_EG: i32 = 5;

// Mobility bonus per square (knight, bishop, rook, queen)
const KNIGHT_MOBILITY_MG: i32 = 4;
const KNIGHT_MOBILITY_EG: i32 = 4;
const BISHOP_MOBILITY_MG: i32 = 5;
const BISHOP_MOBILITY_EG: i32 = 5;
const ROOK_MOBILITY_MG: i32 = 2;
const ROOK_MOBILITY_EG: i32 = 3;
const QUEEN_MOBILITY_MG: i32 = 1;
const QUEEN_MOBILITY_EG: i32 = 2;

// Outpost bonuses
const KNIGHT_OUTPOST_BONUS_MG: i32 = 25;
const KNIGHT_OUTPOST_BONUS_EG: i32 = 15;
const BISHOP_OUTPOST_BONUS_MG: i32 = 15;
const BISHOP_OUTPOST_BONUS_EG: i32 = 10;

// Backward pawn penalties
const BACKWARD_PAWN_PENALTY_MG: i32 = -12;
const BACKWARD_PAWN_PENALTY_EG: i32 = -8;

// King safety: penalty for open files near king, bonus for pawn shield

// Tempo bonus for the side to move
const TEMPO_BONUS: i32 = 15;

// ---------------------------------------------------------------------------
// Sub-evaluators (each #[inline(never)] so LLVM compiles them independently,
// keeping peak memory low during --release builds)
// ---------------------------------------------------------------------------

#[inline(never)]
fn eval_psqt(board: &Board, mg: &mut i32, eg: &mut i32, phase: &mut i32) {
    for c in [Color::White, Color::Black] {
        let sign = if c == Color::White { 1 } else { -1 };
        let pts: &[(PieceType, i32, i32, &[i32; 64], &[i32; 64], i32)] = &[
            (
                PieceType::Pawn,
                PAWN_MG_VAL,
                PAWN_EG_VAL,
                &PAWN_MG,
                &PAWN_EG,
                0,
            ),
            (
                PieceType::Knight,
                KNIGHT_MG_VAL,
                KNIGHT_EG_VAL,
                &KNIGHT_MG,
                &KNIGHT_EG,
                PHASE_WEIGHT_KNIGHT,
            ),
            (
                PieceType::Bishop,
                BISHOP_MG_VAL,
                BISHOP_EG_VAL,
                &BISHOP_MG,
                &BISHOP_EG,
                PHASE_WEIGHT_BISHOP,
            ),
            (
                PieceType::Rook,
                ROOK_MG_VAL,
                ROOK_EG_VAL,
                &ROOK_MG,
                &ROOK_EG,
                PHASE_WEIGHT_ROOK,
            ),
            (
                PieceType::Queen,
                QUEEN_MG_VAL,
                QUEEN_EG_VAL,
                &QUEEN_MG,
                &QUEEN_EG,
                PHASE_WEIGHT_QUEEN,
            ),
            (PieceType::King, 0, 0, &KING_MG, &KING_EG, 0),
        ];
        for &(pt, mg_val, eg_val, mg_table, eg_table, pw) in pts {
            let mut bb = board.pieces[c as usize][pt as usize];
            while bb != 0 {
                let sq = crate::board::pop_lsb(&mut bb);
                let idx = flip_sq(sq.0 as usize, c);
                *mg += sign * (mg_val + mg_table[idx]);
                *eg += sign * (eg_val + eg_table[idx]);
                *phase += pw;
            }
        }
    }
}

#[inline(never)]
fn eval_pawns(board: &Board, mg: &mut i32, eg: &mut i32) {
    for c in [Color::White, Color::Black] {
        let our_pawns = board.pieces[c as usize][PieceType::Pawn as usize];
        let their_pawns = board.pieces[c.opposite() as usize][PieceType::Pawn as usize];
        let sign = if c == Color::White { 1 } else { -1 };
        let mut bb = our_pawns;
        while bb != 0 {
            let sq = crate::board::pop_lsb(&mut bb);
            let file = sq.file() as usize;
            let rank = sq.rank() as usize;
            // Doubled
            if popcount(our_pawns & FILE_MASK[file]) > 1 {
                *mg += sign * DOUBLED_PAWN_PENALTY_MG;
                *eg += sign * DOUBLED_PAWN_PENALTY_EG;
            }
            // Isolated
            let is_isolated = (our_pawns & ADJACENT_FILE_MASK[file]) == 0;
            if is_isolated {
                *mg += sign * ISOLATED_PAWN_PENALTY_MG;
                *eg += sign * ISOLATED_PAWN_PENALTY_EG;
            }
            // Passed
            let pp_mask = passed_pawn_mask(sq.0 as usize, c);
            let is_passed = (their_pawns & pp_mask) == 0;
            if is_passed {
                let rel = if c == Color::White { rank } else { 7 - rank };
                *mg += sign * PASSED_PAWN_BONUS_MG[rel];
                *eg += sign * PASSED_PAWN_BONUS_EG[rel];
            }
            // Backward
            if !is_isolated {
                if !((c == Color::White && rank == 7) || (c == Color::Black && rank == 0)) {
                    let stop_sq = if c == Color::White {
                        crate::board::Square::new(sq.0 + 8)
                    } else {
                        crate::board::Square::new(sq.0 - 8)
                    };
                    let stop_attacked =
                        (crate::attacks::pawn_attacks(c, stop_sq) & their_pawns) != 0;
                    let mut behind = 0u64;
                    if c == Color::White {
                        for r in 0..=rank {
                            behind |= RANK_MASK[r];
                        }
                    } else {
                        for r in rank..8 {
                            behind |= RANK_MASK[r];
                        }
                    }
                    let has_support = (our_pawns & ADJACENT_FILE_MASK[file] & behind) != 0;
                    if !is_passed && !has_support && stop_attacked {
                        *mg += sign * BACKWARD_PAWN_PENALTY_MG;
                        *eg += sign * BACKWARD_PAWN_PENALTY_EG;
                    }
                }
            }
        }
    }
}

#[inline(never)]
fn eval_pieces(board: &Board, all_occ: u64, mg: &mut i32, eg: &mut i32) {
    for c in [Color::White, Color::Black] {
        let sign = if c == Color::White { 1 } else { -1 };
        let friendly = board.colors[c as usize];
        // Bishop pair
        if popcount(board.pieces[c as usize][PieceType::Bishop as usize]) >= 2 {
            *mg += sign * BISHOP_PAIR_BONUS_MG;
            *eg += sign * BISHOP_PAIR_BONUS_EG;
        }
        let our_pawns = board.pieces[c as usize][PieceType::Pawn as usize];
        let their_pawns = board.pieces[c.opposite() as usize][PieceType::Pawn as usize];
        // Rook open/semi-open files
        let mut rooks = board.pieces[c as usize][PieceType::Rook as usize];
        while rooks != 0 {
            let sq = crate::board::pop_lsb(&mut rooks);
            let file = sq.file() as usize;
            let fb = FILE_MASK[file];
            if (our_pawns & fb) == 0 {
                if (their_pawns & fb) == 0 {
                    *mg += sign * ROOK_OPEN_FILE_MG;
                    *eg += sign * ROOK_OPEN_FILE_EG;
                } else {
                    *mg += sign * ROOK_SEMI_OPEN_FILE_MG;
                    *eg += sign * ROOK_SEMI_OPEN_FILE_EG;
                }
            }
        }
        // Mobility (exclude squares attacked by enemy pawns)
        let enemy_pawn_atk = {
            let mut atk = 0u64;
            let mut ep = board.pieces[c.opposite() as usize][PieceType::Pawn as usize];
            while ep != 0 {
                let s = crate::board::pop_lsb(&mut ep);
                atk |= crate::attacks::pawn_attacks(c.opposite(), s);
            }
            atk
        };
        let safe = !friendly & !enemy_pawn_atk;
        let mut kn = board.pieces[c as usize][PieceType::Knight as usize];
        while kn != 0 {
            let sq = crate::board::pop_lsb(&mut kn);
            let m = popcount(attacks::knight_attacks(sq) & safe) as i32;
            *mg += sign * (m - 4) * KNIGHT_MOBILITY_MG;
            *eg += sign * (m - 4) * KNIGHT_MOBILITY_EG;
        }
        let mut bi = board.pieces[c as usize][PieceType::Bishop as usize];
        while bi != 0 {
            let sq = crate::board::pop_lsb(&mut bi);
            let m = popcount(attacks::bishop_attacks(sq, all_occ) & safe) as i32;
            *mg += sign * (m - 6) * BISHOP_MOBILITY_MG;
            *eg += sign * (m - 6) * BISHOP_MOBILITY_EG;
        }
        let mut ro = board.pieces[c as usize][PieceType::Rook as usize];
        while ro != 0 {
            let sq = crate::board::pop_lsb(&mut ro);
            let m = popcount(attacks::rook_attacks(sq, all_occ) & safe) as i32;
            *mg += sign * (m - 7) * ROOK_MOBILITY_MG;
            *eg += sign * (m - 7) * ROOK_MOBILITY_EG;
        }
        let mut qu = board.pieces[c as usize][PieceType::Queen as usize];
        while qu != 0 {
            let sq = crate::board::pop_lsb(&mut qu);
            let m = popcount(attacks::queen_attacks(sq, all_occ) & safe) as i32;
            *mg += sign * (m - 14) * QUEEN_MOBILITY_MG;
            *eg += sign * (m - 14) * QUEEN_MOBILITY_EG;
        }
        // Outposts
        let mut kn2 = board.pieces[c as usize][PieceType::Knight as usize];
        while kn2 != 0 {
            let sq = crate::board::pop_lsb(&mut kn2);
            let rank = sq.rank() as usize;
            let rel = if c == Color::White { rank } else { 7 - rank };
            if rel >= 4 {
                let supported = (crate::attacks::pawn_attacks(c.opposite(), sq) & our_pawns) != 0;
                let file = sq.file() as usize;
                let mut ahead = 0u64;
                if c == Color::White {
                    for r in rank..8 {
                        ahead |= RANK_MASK[r];
                    }
                } else {
                    for r in 0..=rank {
                        ahead |= RANK_MASK[r];
                    }
                }
                let not_cont = (their_pawns & ADJACENT_FILE_MASK[file] & ahead) == 0;
                if supported && not_cont {
                    *mg += sign * KNIGHT_OUTPOST_BONUS_MG;
                    *eg += sign * KNIGHT_OUTPOST_BONUS_EG;
                }
            }
        }
        let mut bi2 = board.pieces[c as usize][PieceType::Bishop as usize];
        while bi2 != 0 {
            let sq = crate::board::pop_lsb(&mut bi2);
            let rank = sq.rank() as usize;
            let rel = if c == Color::White { rank } else { 7 - rank };
            if rel >= 4 {
                let supported = (crate::attacks::pawn_attacks(c.opposite(), sq) & our_pawns) != 0;
                let file = sq.file() as usize;
                let mut ahead = 0u64;
                if c == Color::White {
                    for r in rank..8 {
                        ahead |= RANK_MASK[r];
                    }
                } else {
                    for r in 0..=rank {
                        ahead |= RANK_MASK[r];
                    }
                }
                let not_cont = (their_pawns & ADJACENT_FILE_MASK[file] & ahead) == 0;
                if supported && not_cont {
                    *mg += sign * BISHOP_OUTPOST_BONUS_MG;
                    *eg += sign * BISHOP_OUTPOST_BONUS_EG;
                }
            }
        }
    }
}

#[inline(never)]
fn eval_king_safety(board: &Board, all_occ: u64, mg: &mut i32) {
    for c in [Color::White, Color::Black] {
        let sign = if c == Color::White { 1 } else { -1 };
        let them = c.opposite();
        let king_bb = board.pieces[c as usize][PieceType::King as usize];
        if king_bb == 0 {
            continue;
        }
        let king_sq = crate::board::lsb(king_bb);
        let king_file = king_sq.file() as usize;
        let our_pawns = board.pieces[c as usize][PieceType::Pawn as usize];
        let their_pawns = board.pieces[them as usize][PieceType::Pawn as usize];
        // Pawn shield
        let shield_files = if king_file == 0 {
            FILE_MASK[0] | FILE_MASK[1]
        } else if king_file == 7 {
            FILE_MASK[6] | FILE_MASK[7]
        } else {
            FILE_MASK[king_file - 1] | FILE_MASK[king_file] | FILE_MASK[king_file + 1]
        };
        let shield_zone = match c {
            Color::White => shield_files & (RANK_MASK[1] | RANK_MASK[2]),
            Color::Black => shield_files & (RANK_MASK[5] | RANK_MASK[6]),
        };
        let sc = popcount(our_pawns & shield_zone) as i32;
        *mg += sign * sc * 12;
        *mg += sign * (sc - 3).min(0) * 15;
        // Open files near king
        let kf0 = if king_file > 0 { king_file - 1 } else { 0 };
        let kf1 = if king_file < 7 { king_file + 1 } else { 7 };
        for f in kf0..=kf1 {
            let fb = FILE_MASK[f];
            if (our_pawns & fb) == 0 {
                if (their_pawns & fb) == 0 {
                    *mg -= sign * 25;
                } else {
                    *mg -= sign * 12;
                }
            }
        }
        // Attacker count + weight
        let king_zone = attacks::king_attacks(king_sq) | (1u64 << king_sq.0);
        let mut ac = 0i32;
        let mut aw = 0i32;
        let occ_nk = all_occ ^ king_bb;
        let mut en = board.pieces[them as usize][PieceType::Knight as usize];
        while en != 0 {
            let s = crate::board::pop_lsb(&mut en);
            if (attacks::knight_attacks(s) & king_zone) != 0 {
                ac += 1;
                aw += 2;
            }
        }
        let mut eb = board.pieces[them as usize][PieceType::Bishop as usize];
        while eb != 0 {
            let s = crate::board::pop_lsb(&mut eb);
            if (attacks::bishop_attacks(s, all_occ) & king_zone) != 0 {
                ac += 1;
                aw += 2;
            }
        }
        let mut er = board.pieces[them as usize][PieceType::Rook as usize];
        while er != 0 {
            let s = crate::board::pop_lsb(&mut er);
            if (attacks::rook_attacks(s, all_occ) & king_zone) != 0 {
                ac += 1;
                aw += 3;
            }
        }
        let mut eq = board.pieces[them as usize][PieceType::Queen as usize];
        while eq != 0 {
            let s = crate::board::pop_lsb(&mut eq);
            if (attacks::queen_attacks(s, all_occ) & king_zone) != 0 {
                ac += 1;
                aw += 5;
            }
        }
        // X-ray threats through the king
        let mut xb = board.pieces[them as usize][PieceType::Bishop as usize];
        while xb != 0 {
            let s = crate::board::pop_lsb(&mut xb);
            if (attacks::bishop_attacks(s, occ_nk) & king_zone) != 0
                && (attacks::bishop_attacks(s, all_occ) & king_zone) == 0
            {
                aw += 1;
            }
        }
        let mut xr = board.pieces[them as usize][PieceType::Rook as usize];
        while xr != 0 {
            let s = crate::board::pop_lsb(&mut xr);
            if (attacks::rook_attacks(s, occ_nk) & king_zone) != 0
                && (attacks::rook_attacks(s, all_occ) & king_zone) == 0
            {
                aw += 2;
            }
        }
        let mut xq = board.pieces[them as usize][PieceType::Queen as usize];
        while xq != 0 {
            let s = crate::board::pop_lsb(&mut xq);
            if (attacks::queen_attacks(s, occ_nk) & king_zone) != 0
                && (attacks::queen_attacks(s, all_occ) & king_zone) == 0
            {
                aw += 3;
            }
        }
        if ac >= 2 {
            *mg -= sign * aw * aw / 4;
        }
    }
}

#[inline(never)]
fn eval_endgame(board: &Board, eg: &mut i32) {
    for c in [Color::White, Color::Black] {
        let sign = if c == Color::White { 1 } else { -1 };
        let our_pawns = board.pieces[c as usize][PieceType::Pawn as usize];
        let their_pawns = board.pieces[c.opposite() as usize][PieceType::Pawn as usize];
        let our_rooks = board.pieces[c as usize][PieceType::Rook as usize];
        let mut bb = our_pawns;
        while bb != 0 {
            let sq = crate::board::pop_lsb(&mut bb);
            let file = sq.file() as usize;
            let rank = sq.rank() as usize;
            let pp = passed_pawn_mask(sq.0 as usize, c);
            if (their_pawns & pp) == 0 {
                let rel = if c == Color::White { rank } else { 7 - rank };
                if file > 0 && (our_pawns & FILE_MASK[file - 1] & RANK_MASK[rank]) != 0 {
                    *eg += sign * 15 * rel as i32;
                }
                if file < 7 && (our_pawns & FILE_MASK[file + 1] & RANK_MASK[rank]) != 0 {
                    *eg += sign * 15 * rel as i32;
                }
                let behind = match c {
                    Color::White => {
                        let mut m = 0u64;
                        for r in 0..rank {
                            m |= RANK_MASK[r];
                        }
                        FILE_MASK[file] & m
                    }
                    Color::Black => {
                        let mut m = 0u64;
                        for r in (rank + 1)..8 {
                            m |= RANK_MASK[r];
                        }
                        FILE_MASK[file] & m
                    }
                };
                if (our_rooks & behind) != 0 {
                    *eg += sign * 25;
                }
            }
        }
    }
}

#[inline(never)]
pub fn evaluate(board: &Board) -> i32 {
    let mut mg_score = 0i32;
    let mut eg_score = 0i32;
    let mut phase = 0i32;
    let all_occ = board.colors[Color::White as usize] | board.colors[Color::Black as usize];

    eval_psqt(board, &mut mg_score, &mut eg_score, &mut phase);
    eval_pawns(board, &mut mg_score, &mut eg_score);
    eval_pieces(board, all_occ, &mut mg_score, &mut eg_score);
    eval_king_safety(board, all_occ, &mut mg_score);
    eval_endgame(board, &mut eg_score);

    if phase > TOTAL_PHASE {
        phase = TOTAL_PHASE;
    }
    let score = (mg_score * phase + eg_score * (TOTAL_PHASE - phase)) / TOTAL_PHASE;
    if board.side_to_move == Color::White {
        score + TEMPO_BONUS
    } else {
        -score + TEMPO_BONUS
    }
}

