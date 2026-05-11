use crate::board::{Color, PieceType, Square};

// HalfKP features: King Square (64) * Piece Type (10) * Piece Square (64) = 40960
pub const HALFKP_FEATURES: usize = 40960;
pub const HIDDEN_SIZE: usize = 256;
pub const L1_SIZE: usize = 32;

#[repr(C)]
pub struct FeatureTransformer {
    pub weights: [i16; HALFKP_FEATURES * HIDDEN_SIZE],
    pub biases: [i16; HIDDEN_SIZE],
}

#[repr(C)]
pub struct Network {
    pub feature_transformer: FeatureTransformer,
    pub layer1_weights: [i8; 2 * HIDDEN_SIZE * L1_SIZE],
    pub layer1_biases: [i32; L1_SIZE],
    pub output_weights: [i8; L1_SIZE],
    pub output_bias: i32,
}

#[derive(Clone)]
pub struct Accumulator {
    pub white: [i16; HIDDEN_SIZE],
    pub black: [i16; HIDDEN_SIZE],
}

impl Accumulator {
    pub fn new(net: &Network) -> Self {
        Self {
            white: net.feature_transformer.biases,
            black: net.feature_transformer.biases,
        }
    }
    
    pub fn update_feature(&mut self, net: &Network, feature_idx_w: usize, feature_idx_b: usize, add: bool) {
        let offset_w = feature_idx_w * HIDDEN_SIZE;
        let offset_b = feature_idx_b * HIDDEN_SIZE;
        
        for i in 0..HIDDEN_SIZE {
            let w_w = net.feature_transformer.weights[offset_w + i];
            let w_b = net.feature_transformer.weights[offset_b + i];
            
            if add {
                self.white[i] += w_w;
                self.black[i] += w_b;
            } else {
                self.white[i] -= w_w;
                self.black[i] -= w_b;
            }
        }
    }
}

// Convert a piece to a HalfKP index (0..9)
// White pieces: P=0, N=1, B=2, R=3, Q=4
// Black pieces: P=5, N=6, B=7, R=8, Q=9
// Note: Kings are not encoded as pieces in HalfKP, their position determines the "bucket"
fn halfkp_piece_index(pt: PieceType, color: Color) -> usize {
    let mut idx = match pt {
        PieceType::Pawn => 0,
        PieceType::Knight => 1,
        PieceType::Bishop => 2,
        PieceType::Rook => 3,
        PieceType::Queen => 4,
        PieceType::King => return 0, // Should never be called for King
    };
    if color == Color::Black {
        idx += 5;
    }
    idx
}

pub fn get_halfkp_indices(
    king_sq_w: Square,
    king_sq_b: Square,
    sq: Square,
    pt: PieceType,
    color: Color
) -> (usize, usize) {
    let p_idx_w = halfkp_piece_index(pt, color);
    
    // Black's perspective is mirrored (both ranks and colors)
    let mirrored_sq = Square::new(sq.0 ^ 56);
    let mirrored_king_sq_b = Square::new(king_sq_b.0 ^ 56);
    let p_idx_b = halfkp_piece_index(pt, color.opposite());
    
    let f_w = (king_sq_w.0 as usize * 10 + p_idx_w) * 64 + sq.0 as usize;
    let f_b = (mirrored_king_sq_b.0 as usize * 10 + p_idx_b) * 64 + mirrored_sq.0 as usize;
    
    (f_w, f_b)
}

// Inference
pub fn evaluate_nnue(net: &Network, acc: &Accumulator, side_to_move: Color) -> i32 {
    let (us, them) = if side_to_move == Color::White {
        (&acc.white, &acc.black)
    } else {
        (&acc.black, &acc.white)
    };
    
    // Hidden Layer 1 (Clipped ReLU)
    let mut l1 = [0i32; L1_SIZE];
    for i in 0..L1_SIZE {
        l1[i] = net.layer1_biases[i];
        
        let mut sum = 0;
        for j in 0..HIDDEN_SIZE {
            // Clipped ReLU max(0, min(127, x))
            let u_val = us[j].clamp(0, 127) as i32;
            let t_val = them[j].clamp(0, 127) as i32;
            
            sum += u_val * (net.layer1_weights[i * 2 * HIDDEN_SIZE + j] as i32);
            sum += t_val * (net.layer1_weights[i * 2 * HIDDEN_SIZE + HIDDEN_SIZE + j] as i32);
        }
        // SCALED_QA (commonly 64)
        l1[i] += sum / 64; 
    }
    
    // Output Layer
    let mut output = net.output_bias;
    for i in 0..L1_SIZE {
        let val = l1[i].clamp(0, 127) as i32;
        output += val * (net.output_weights[i] as i32);
    }
    
    output / 16 // Scale down to centipawns
}
