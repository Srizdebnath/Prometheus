//! Magic Bitboard attack tables for sliding pieces (bishops and rooks).
//!
//! This replaces the loop-based ray attack computation with a single
//! lookup per piece, giving roughly 10-20x speedup for sliding piece
//! attack generation.
//!
//! The technique: for each square, we mask the "relevant" occupancy bits
//! (the squares a piece could be blocked on), multiply by a "magic number",
//! and shift right to get an index into a precomputed attack table.

use crate::board::{Bitboard, Square};

// ============================================================
// Magic numbers found by trial (well-known sets from the community)
// ============================================================

const ROOK_MAGICS: [u64; 64] = [
    0x0080001020400080, 0x0040001000200040, 0x0080081000200080, 0x0080040800100080,
    0x0080020400080080, 0x0080010200040080, 0x0080008001000200, 0x0080002040800100,
    0x0000800020400080, 0x0000400020005000, 0x0000801000200080, 0x0000800800100080,
    0x0000800400080080, 0x0000800200040080, 0x0000800100020080, 0x0000800040800100,
    0x0000208000400080, 0x0000404000201000, 0x0000808010002000, 0x0000808008001000,
    0x0000808004000800, 0x0000808002000400, 0x0000010100020004, 0x0000020000408104,
    0x0000208080004000, 0x0000200040005000, 0x0000100080200080, 0x0000080080100080,
    0x0000040080080080, 0x0000020080040080, 0x0000010080800200, 0x0000800080004100,
    0x0000204000800080, 0x0000200040401000, 0x0000100080802000, 0x0000080080801000,
    0x0000040080800800, 0x0000020080800400, 0x0000020001010004, 0x0000800040800100,
    0x0000204000808000, 0x0000200040008080, 0x0000100020008080, 0x0000080010008080,
    0x0000040008008080, 0x0000020004008080, 0x0000010002008080, 0x0000004081020004,
    0x0000204000800080, 0x0000200040008080, 0x0000100020008080, 0x0000080010008080,
    0x0000040008008080, 0x0000020004008080, 0x0000800100020080, 0x0000800041000080,
    0x00FFFCDDFCED714A, 0x007FFCDDFCED714A, 0x003FFFCDFFD88096, 0x0000040010000811,
    0x0001FFFF8BFE4933, 0x0000FFFDF40C0009, 0x0000B3C0200102CA, 0x00005FC0100102CA,
];

const BISHOP_MAGICS: [u64; 64] = [
    0x0002020202020200, 0x0002020202020000, 0x0004010202000000, 0x0004040080000000,
    0x0001104000000000, 0x0000821040000000, 0x0000410410400000, 0x0000104104104000,
    0x0000040404040400, 0x0000020202020200, 0x0000040102020000, 0x0000040400800000,
    0x0000011040000000, 0x0000008210400000, 0x0000004104104000, 0x0000002082082000,
    0x0004000808080800, 0x0002000404040400, 0x0001000202020200, 0x0000800802004000,
    0x0000800400A00000, 0x0000200100884000, 0x0000400082082000, 0x0000200041041000,
    0x0002080010101000, 0x0001040008080800, 0x0000208004010400, 0x0000404004010200,
    0x0000840000802000, 0x0000404002011000, 0x0000808001041000, 0x0000404000820800,
    0x0001041000202000, 0x0000820800101000, 0x0000104400080800, 0x0000020080080080,
    0x0000404040040100, 0x0000808100020100, 0x0001010100020800, 0x0000808080010400,
    0x0000820820004000, 0x0000410410002000, 0x0000082088001000, 0x0000002011000800,
    0x0000080100400400, 0x0001010101000200, 0x0002020202000400, 0x0001010101000200,
    0x0000410410400000, 0x0000208208200000, 0x0000002084100000, 0x0000000020880000,
    0x0000001002020000, 0x0000040408020000, 0x0004040404040000, 0x0002020202020000,
    0x0000104104104000, 0x0000002082082000, 0x0000000020841000, 0x0000000000208800,
    0x0000000010020200, 0x0000000404080200, 0x0000040404040400, 0x0002020202020200,
];

// Number of relevant bits for each square (determines table size)
const ROOK_BITS: [u32; 64] = [
    12, 11, 11, 11, 11, 11, 11, 12,
    11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11,
    12, 11, 11, 11, 11, 11, 11, 12,
];

const BISHOP_BITS: [u32; 64] = [
    6, 5, 5, 5, 5, 5, 5, 6,
    5, 5, 5, 5, 5, 5, 5, 5,
    5, 5, 7, 7, 7, 7, 5, 5,
    5, 5, 7, 9, 9, 7, 5, 5,
    5, 5, 7, 9, 9, 7, 5, 5,
    5, 5, 7, 7, 7, 7, 5, 5,
    5, 5, 5, 5, 5, 5, 5, 5,
    6, 5, 5, 5, 5, 5, 5, 6,
];

// ============================================================
// Mask generation: relevant occupancy bits for each square
// ============================================================

fn rook_mask(sq: usize) -> Bitboard {
    let rank = (sq / 8) as i32;
    let file = (sq % 8) as i32;
    let mut mask = 0u64;
    
    for r in (rank + 1)..7 { mask |= 1u64 << (r * 8 + file); }
    for r in 1..rank       { mask |= 1u64 << (r * 8 + file); }
    for f in (file + 1)..7 { mask |= 1u64 << (rank * 8 + f); }
    for f in 1..file       { mask |= 1u64 << (rank * 8 + f); }
    
    mask
}

fn bishop_mask(sq: usize) -> Bitboard {
    let rank = (sq / 8) as i32;
    let file = (sq % 8) as i32;
    let mut mask = 0u64;
    
    let dirs: [(i32, i32); 4] = [(1,1), (1,-1), (-1,1), (-1,-1)];
    for &(dr, df) in &dirs {
        let mut r = rank + dr;
        let mut f = file + df;
        while r > 0 && r < 7 && f > 0 && f < 7 {
            mask |= 1u64 << (r * 8 + f);
            r += dr;
            f += df;
        }
    }
    
    mask
}

// ============================================================
// Slow attack generation (used only during initialization)
// ============================================================

fn rook_attacks_slow(sq: usize, occ: Bitboard) -> Bitboard {
    let rank = (sq / 8) as i32;
    let file = (sq % 8) as i32;
    let mut attacks = 0u64;
    
    // North
    for r in (rank + 1)..8 {
        let bit = 1u64 << (r * 8 + file);
        attacks |= bit;
        if occ & bit != 0 { break; }
    }
    // South
    for r in (0..rank).rev() {
        let bit = 1u64 << (r * 8 + file);
        attacks |= bit;
        if occ & bit != 0 { break; }
    }
    // East
    for f in (file + 1)..8 {
        let bit = 1u64 << (rank * 8 + f);
        attacks |= bit;
        if occ & bit != 0 { break; }
    }
    // West
    for f in (0..file).rev() {
        let bit = 1u64 << (rank * 8 + f);
        attacks |= bit;
        if occ & bit != 0 { break; }
    }
    
    attacks
}

fn bishop_attacks_slow(sq: usize, occ: Bitboard) -> Bitboard {
    let rank = (sq / 8) as i32;
    let file = (sq % 8) as i32;
    let mut attacks = 0u64;
    
    let dirs: [(i32, i32); 4] = [(1,1), (1,-1), (-1,1), (-1,-1)];
    for &(dr, df) in &dirs {
        let mut r = rank + dr;
        let mut f = file + df;
        while r >= 0 && r < 8 && f >= 0 && f < 8 {
            let bit = 1u64 << (r * 8 + f);
            attacks |= bit;
            if occ & bit != 0 { break; }
            r += dr;
            f += df;
        }
    }
    
    attacks
}

// ============================================================
// Enumerate all subsets of a mask (Carry-Rippler)
// ============================================================

fn enumerate_subsets(mask: Bitboard) -> Vec<Bitboard> {
    let mut subsets = Vec::new();
    let mut subset: u64 = 0;
    loop {
        subsets.push(subset);
        subset = subset.wrapping_sub(mask) & mask;
        if subset == 0 { break; }
    }
    subsets
}

// ============================================================
// The global magic tables (initialized at startup)
// ============================================================

pub struct MagicEntry {
    pub mask: Bitboard,
    pub magic: u64,
    pub shift: u32,
    pub offset: usize,  // offset into the shared attack table
}

pub struct MagicTables {
    pub rook_entries: [MagicEntry; 64],
    pub bishop_entries: [MagicEntry; 64],
    pub rook_attacks: Vec<Bitboard>,
    pub bishop_attacks: Vec<Bitboard>,
}

impl MagicTables {
    pub fn init() -> Self {
        // Calculate total table sizes
        let rook_total: usize = (0..64).map(|sq| 1usize << ROOK_BITS[sq]).sum();
        let bishop_total: usize = (0..64).map(|sq| 1usize << BISHOP_BITS[sq]).sum();
        
        let mut rook_attacks_table = vec![0u64; rook_total];
        let mut bishop_attacks_table = vec![0u64; bishop_total];
        
        // Placeholder entries
        let empty_entry = || MagicEntry { mask: 0, magic: 0, shift: 0, offset: 0 };
        let mut rook_entries: [MagicEntry; 64] = std::array::from_fn(|_| empty_entry());
        let mut bishop_entries: [MagicEntry; 64] = std::array::from_fn(|_| empty_entry());
        
        // Initialize rook tables
        let mut offset = 0;
        for sq in 0..64 {
            let mask = rook_mask(sq);
            let bits = ROOK_BITS[sq];
            let magic = ROOK_MAGICS[sq];
            let shift = 64 - bits;
            
            rook_entries[sq] = MagicEntry { mask, magic, shift, offset };
            
            for occ in enumerate_subsets(mask) {
                let index = ((occ.wrapping_mul(magic)) >> shift) as usize;
                rook_attacks_table[offset + index] = rook_attacks_slow(sq, occ);
            }
            
            offset += 1 << bits;
        }
        
        // Initialize bishop tables
        offset = 0;
        for sq in 0..64 {
            let mask = bishop_mask(sq);
            let bits = BISHOP_BITS[sq];
            let magic = BISHOP_MAGICS[sq];
            let shift = 64 - bits;
            
            bishop_entries[sq] = MagicEntry { mask, magic, shift, offset };
            
            for occ in enumerate_subsets(mask) {
                let index = ((occ.wrapping_mul(magic)) >> shift) as usize;
                bishop_attacks_table[offset + index] = bishop_attacks_slow(sq, occ);
            }
            
            offset += 1 << bits;
        }
        
        MagicTables {
            rook_entries,
            bishop_entries,
            rook_attacks: rook_attacks_table,
            bishop_attacks: bishop_attacks_table,
        }
    }
    
    #[inline(always)]
    pub fn rook_attacks(&self, sq: Square, occ: Bitboard) -> Bitboard {
        let entry = &self.rook_entries[sq.0 as usize];
        let index = (((occ & entry.mask).wrapping_mul(entry.magic)) >> entry.shift) as usize;
        unsafe { *self.rook_attacks.get_unchecked(entry.offset + index) }
    }
    
    #[inline(always)]
    pub fn bishop_attacks(&self, sq: Square, occ: Bitboard) -> Bitboard {
        let entry = &self.bishop_entries[sq.0 as usize];
        let index = (((occ & entry.mask).wrapping_mul(entry.magic)) >> entry.shift) as usize;
        unsafe { *self.bishop_attacks.get_unchecked(entry.offset + index) }
    }
    
    #[inline(always)]
    pub fn queen_attacks(&self, sq: Square, occ: Bitboard) -> Bitboard {
        self.rook_attacks(sq, occ) | self.bishop_attacks(sq, occ)
    }
}

// Global singleton - initialized once at startup
use std::sync::OnceLock;

static MAGIC_TABLES: OnceLock<MagicTables> = OnceLock::new();

pub fn init_magics() {
    MAGIC_TABLES.get_or_init(MagicTables::init);
}

pub fn get_magic_tables() -> &'static MagicTables {
    MAGIC_TABLES.get_or_init(MagicTables::init)
}

// ============================================================
// Public API — drop-in replacements for the old ray functions
// ============================================================

#[inline(always)]
pub fn magic_rook_attacks(sq: Square, occ: Bitboard) -> Bitboard {
    get_magic_tables().rook_attacks(sq, occ)
}

#[inline(always)]
pub fn magic_bishop_attacks(sq: Square, occ: Bitboard) -> Bitboard {
    get_magic_tables().bishop_attacks(sq, occ)
}

#[inline(always)]
pub fn magic_queen_attacks(sq: Square, occ: Bitboard) -> Bitboard {
    get_magic_tables().queen_attacks(sq, occ)
}
