pub struct ZobristKeys {
    pub pieces: [[[u64; 64]; 6]; 2], // color, piece type, square
    pub side_to_move: u64,
    pub castling_rights: [u64; 16],
    pub en_passant: [u64; 8], // file
}

const fn xorshift64star(state: &mut u64) -> u64 {
    let mut x = *state;
    x ^= x >> 12;
    x ^= x << 25;
    x ^= x >> 27;
    *state = x;
    x.wrapping_mul(0x2545F4914F6CDD1D)
}

pub const ZOBRIST: ZobristKeys = {
    let mut state = 0x123456789ABCDEF0; // Starting seed
    
    let mut pieces = [[[0; 64]; 6]; 2];
    let mut c = 0;
    while c < 2 {
        let mut pt = 0;
        while pt < 6 {
            let mut sq = 0;
            while sq < 64 {
                pieces[c][pt][sq] = xorshift64star(&mut state);
                sq += 1;
            }
            pt += 1;
        }
        c += 1;
    }
    
    let side_to_move = xorshift64star(&mut state);
    
    let mut castling_rights = [0; 16];
    let mut i = 0;
    while i < 16 {
        castling_rights[i] = xorshift64star(&mut state);
        i += 1;
    }
    
    let mut en_passant = [0; 8];
    let mut i = 0;
    while i < 8 {
        en_passant[i] = xorshift64star(&mut state);
        i += 1;
    }
    
    ZobristKeys {
        pieces,
        side_to_move,
        castling_rights,
        en_passant,
    }
};
