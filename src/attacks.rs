use crate::board::{Bitboard, Color, Square};

pub const NOT_A_FILE: Bitboard = 0xfefefefefefefefe;
pub const NOT_H_FILE: Bitboard = 0x7f7f7f7f7f7f7f7f;
pub const NOT_AB_FILE: Bitboard = 0xfcfcfcfcfcfcfcfc;
pub const NOT_GH_FILE: Bitboard = 0x3f3f3f3f3f3f3f3f;

const fn compute_pawn_attacks(color: usize, sq: usize) -> Bitboard {
    let bitboard = 1u64 << sq;
    if color == 0 { // White
        ((bitboard << 7) & NOT_H_FILE) | ((bitboard << 9) & NOT_A_FILE)
    } else { // Black
        ((bitboard >> 9) & NOT_H_FILE) | ((bitboard >> 7) & NOT_A_FILE)
    }
}

const fn init_pawn_attacks() -> [[Bitboard; 64]; 2] {
    let mut attacks = [[0; 64]; 2];
    let mut sq = 0;
    while sq < 64 {
        attacks[0][sq] = compute_pawn_attacks(0, sq);
        attacks[1][sq] = compute_pawn_attacks(1, sq);
        sq += 1;
    }
    attacks
}

pub const PAWN_ATTACKS: [[Bitboard; 64]; 2] = init_pawn_attacks();

const fn compute_knight_attacks(sq: usize) -> Bitboard {
    let bitboard = 1u64 << sq;
    let mut knight = 0;
    knight |= (bitboard << 17) & NOT_A_FILE;
    knight |= (bitboard << 10) & NOT_AB_FILE;
    knight |= (bitboard >> 6)  & NOT_AB_FILE;
    knight |= (bitboard >> 15) & NOT_A_FILE;
    knight |= (bitboard << 15) & NOT_H_FILE;
    knight |= (bitboard << 6)  & NOT_GH_FILE;
    knight |= (bitboard >> 10) & NOT_GH_FILE;
    knight |= (bitboard >> 17) & NOT_H_FILE;
    knight
}

const fn init_knight_attacks() -> [Bitboard; 64] {
    let mut attacks = [0; 64];
    let mut sq = 0;
    while sq < 64 {
        attacks[sq] = compute_knight_attacks(sq);
        sq += 1;
    }
    attacks
}

pub const KNIGHT_ATTACKS: [Bitboard; 64] = init_knight_attacks();

const fn compute_king_attacks(sq: usize) -> Bitboard {
    let bitboard = 1u64 << sq;
    let mut king = 0;
    king |= bitboard << 8;
    king |= bitboard >> 8;
    king |= (bitboard << 1) & NOT_A_FILE;
    king |= (bitboard >> 1) & NOT_H_FILE;
    king |= (bitboard << 9) & NOT_A_FILE;
    king |= (bitboard >> 9) & NOT_H_FILE;
    king |= (bitboard << 7) & NOT_H_FILE;
    king |= (bitboard >> 7) & NOT_A_FILE;
    king
}

const fn init_king_attacks() -> [Bitboard; 64] {
    let mut attacks = [0; 64];
    let mut sq = 0;
    while sq < 64 {
        attacks[sq] = compute_king_attacks(sq);
        sq += 1;
    }
    attacks
}

pub const KING_ATTACKS: [Bitboard; 64] = init_king_attacks();

#[inline]
pub fn pawn_attacks(color: Color, sq: Square) -> Bitboard {
    PAWN_ATTACKS[color as usize][sq.0 as usize]
}

#[inline]
pub fn knight_attacks(sq: Square) -> Bitboard {
    KNIGHT_ATTACKS[sq.0 as usize]
}

#[inline]
pub fn king_attacks(sq: Square) -> Bitboard {
    KING_ATTACKS[sq.0 as usize]
}

// Ray generation for sliding pieces (fallback before magic bitboards)
const fn compute_ray(sq: usize, delta_rank: i8, delta_file: i8) -> Bitboard {
    let mut ray = 0;
    let mut r = (sq / 8) as i8 + delta_rank;
    let mut f = (sq % 8) as i8 + delta_file;
    
    while r >= 0 && r < 8 && f >= 0 && f < 8 {
        ray |= 1u64 << (r * 8 + f);
        r += delta_rank;
        f += delta_file;
    }
    ray
}

const fn init_rays() -> [[Bitboard; 64]; 8] {
    let mut rays = [[0; 64]; 8];
    let deltas = [
        (1, 0), (-1, 0), (0, 1), (0, -1),   // Rook directions: N, S, E, W
        (1, 1), (1, -1), (-1, 1), (-1, -1)  // Bishop directions: NE, NW, SE, SW
    ];
    
    let mut d = 0;
    while d < 8 {
        let mut sq = 0;
        while sq < 64 {
            rays[d][sq] = compute_ray(sq, deltas[d].0, deltas[d].1);
            sq += 1;
        }
        d += 1;
    }
    rays
}

pub const RAYS: [[Bitboard; 64]; 8] = init_rays();

pub fn get_ray_attacks(sq: Square, occ: Bitboard, dir: usize) -> Bitboard {
    let ray = RAYS[dir][sq.0 as usize];
    let blockers = ray & occ;
    if blockers == 0 {
        return ray;
    }

    let blocker_sq = if dir == 0 || dir == 2 || dir == 4 || dir == 5 {
        // Positive ray: find LSB
        blockers.trailing_zeros() as usize
    } else {
        // Negative ray: find MSB
        63 - blockers.leading_zeros() as usize
    };

    ray ^ RAYS[dir][blocker_sq]
}

#[inline(always)]
pub fn bishop_attacks(sq: Square, occ: Bitboard) -> Bitboard {
    get_ray_attacks(sq, occ, 4) | get_ray_attacks(sq, occ, 5) |
    get_ray_attacks(sq, occ, 6) | get_ray_attacks(sq, occ, 7)
}

#[inline(always)]
pub fn rook_attacks(sq: Square, occ: Bitboard) -> Bitboard {
    get_ray_attacks(sq, occ, 0) | get_ray_attacks(sq, occ, 1) |
    get_ray_attacks(sq, occ, 2) | get_ray_attacks(sq, occ, 3)
}

#[inline(always)]
pub fn queen_attacks(sq: Square, occ: Bitboard) -> Bitboard {
    rook_attacks(sq, occ) | bishop_attacks(sq, occ)
}
