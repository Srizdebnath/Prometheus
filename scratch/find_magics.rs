use std::time::Instant;

type Bitboard = u64;

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

fn rook_attacks_slow(sq: usize, occ: Bitboard) -> Bitboard {
    let rank = (sq / 8) as i32;
    let file = (sq % 8) as i32;
    let mut attacks = 0u64;
    for r in (rank + 1)..8 {
        let bit = 1u64 << (r * 8 + file);
        attacks |= bit;
        if occ & bit != 0 { break; }
    }
    for r in (0..rank).rev() {
        let bit = 1u64 << (r * 8 + file);
        attacks |= bit;
        if occ & bit != 0 { break; }
    }
    for f in (file + 1)..8 {
        let bit = 1u64 << (rank * 8 + f);
        attacks |= bit;
        if occ & bit != 0 { break; }
    }
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

fn find_magic(sq: usize, bits: u32, is_rook: bool) -> u64 {
    let mask = if is_rook { rook_mask(sq) } else { bishop_mask(sq) };
    let subsets = enumerate_subsets(mask);
    let n = subsets.len();
    let mut attacks = Vec::with_capacity(n);
    for &occ in &subsets {
        attacks.push(if is_rook { rook_attacks_slow(sq, occ) } else { bishop_attacks_slow(sq, occ) });
    }

    let mut rng = 123456789u64;
    loop {
        // Simple LCG
        rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
        let magic = rng & rng.wrapping_shl(13) & rng.wrapping_shl(27);

        let mut table = vec![0u64; 1 << bits];
        let mut fail = false;
        for i in 0..n {
            let index = (subsets[i].wrapping_mul(magic) >> (64 - bits)) as usize;
            if table[index] != 0 && table[index] != attacks[i] {
                fail = true;
                break;
            }
            table[index] = attacks[i];
        }
        if !fail {
            return magic;
        }
    }
}

fn main() {
    println!("ROOK_MAGICS:");
    for sq in 0..64 {
        let bits = if [0, 7, 56, 63].contains(&sq) { 12 } else { 11 }; // Simplified
        let magic = find_magic(sq, bits as u32, true);
        print!("0x{:016X}, ", magic);
        if (sq + 1) % 4 == 0 { println!(""); }
    }
    println!("\nBISHOP_MAGICS:");
    for sq in 0..64 {
        let bits = 9; // Simplified
        let magic = find_magic(sq, bits as u32, false);
        print!("0x{:016X}, ", magic);
        if (sq + 1) % 4 == 0 { println!(""); }
    }
}
