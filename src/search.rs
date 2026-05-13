use std::time::{Instant, Duration};
use crate::board::{Board, Move, PieceType};
use crate::movegen::{MoveList, generate_moves};
use crate::evaluation::evaluate;
use crate::transposition::{TranspositionTable, TTEntry, NodeType};

pub const MAX_GAME_PLY: usize = 1024;

pub const INFINITY: i32 = 30000;
pub const MATE_SCORE: i32 = 29000;
pub const MAX_PLY: usize = 128;
pub const MAX_HIST: i32 = 16384;

// Logarithmic Late Move Reduction table
static mut LMR_TABLE: [[i32; 64]; 64] = [[0; 64]; 64];

pub fn init_lmr() {
    unsafe {
        for depth in 1..64 {
            for move_num in 1..64 {
                LMR_TABLE[depth][move_num] = 
                    (0.75 + (depth as f64).ln() * (move_num as f64).ln() / 2.25) as i32;
            }
        }
    }
}

#[inline]
fn lmr_reduction(depth: usize, move_num: usize) -> i32 {
    unsafe {
        LMR_TABLE[depth.min(63)][move_num.min(63)]
    }
}

// Piece values for SEE
const SEE_VALUES: [i32; 7] = [100, 300, 320, 500, 900, 20000, 0];

fn piece_value_see(pt: PieceType) -> i32 {
    match pt {
        PieceType::Pawn => SEE_VALUES[0],
        PieceType::Knight => SEE_VALUES[1],
        PieceType::Bishop => SEE_VALUES[2],
        PieceType::Rook => SEE_VALUES[3],
        PieceType::Queen => SEE_VALUES[4],
        PieceType::King => SEE_VALUES[5],
    }
}

/// Static Exchange Evaluation: returns true if the capture on `m` wins 
/// material (or is at least equal). This is used for:
/// - Pruning losing captures in quiescence search
/// - Better move ordering (good captures vs bad captures)
/// - More aggressive LMR on SEE-negative moves
pub fn see_ge(board: &Board, m: Move, threshold: i32) -> bool {
    use crate::board::Color;
    use crate::attacks;
    
    let from = m.from();
    let to = m.to();
    let us = board.side_to_move;
    
    // Get the initial balance: value of captured piece - threshold
    let victim_pt = board.piece_type_on(to, us.opposite());
    let mut balance = if let Some(vpt) = victim_pt {
        piece_value_see(vpt) - threshold
    } else {
        // En passant captures a pawn
        if m.flags() == 5 { 100 - threshold } else { -threshold }
    };
    
    // If even capturing for free doesn't meet threshold, fail
    if balance < 0 { return false; }
    
    // Value of our piece that just captured
    let attacker_pt = board.piece_type_on(from, us).unwrap_or(PieceType::Pawn);
    
    // Worst case: we lose our piece
    balance -= piece_value_see(attacker_pt);
    
    // If even losing our piece still meets threshold, succeed
    if balance >= 0 { return true; }
    
    // Now simulate the swap sequence
    let mut occ = (board.colors[0] | board.colors[1]) ^ (1u64 << from.0) ^ (1u64 << to.0);
    
    // Handle en passant - remove the captured pawn too
    if m.flags() == 5 {
        let ep_pawn_sq = if us == Color::White { to.0 - 8 } else { to.0 + 8 };
        occ ^= 1u64 << ep_pawn_sq;
    }
    
    let mut side_to_move = us.opposite(); // opponent's turn to recapture
    
    loop {
        // Find the least valuable attacker for `side_to_move` to the `to` square
        
        // Pawns
        let pawn_attackers = attacks::pawn_attacks(side_to_move.opposite(), to) 
                           & board.pieces[side_to_move as usize][PieceType::Pawn as usize] & occ;
        if pawn_attackers != 0 {
            balance = -balance - 1 - 100; // Capture and risk losing a pawn
            if balance >= 0 { // Even losing a pawn still above threshold
                // But check: can the other side recapture?
                occ ^= 1u64 << pawn_attackers.trailing_zeros();
                side_to_move = side_to_move.opposite();
                continue;
            }
            return side_to_move != us; // If it's our turn and we can't do better, fail
        }
        
        // Knights
        let knight_attackers = attacks::knight_attacks(to) 
                             & board.pieces[side_to_move as usize][PieceType::Knight as usize] & occ;
        if knight_attackers != 0 {
            balance = -balance - 1 - 300;
            if balance >= 0 {
                occ ^= 1u64 << knight_attackers.trailing_zeros();
                side_to_move = side_to_move.opposite();
                continue;
            }
            return side_to_move != us;
        }
        
        // Bishops
        let bishop_attackers = attacks::bishop_attacks(to, occ) 
                             & board.pieces[side_to_move as usize][PieceType::Bishop as usize] & occ;
        if bishop_attackers != 0 {
            balance = -balance - 1 - 320;
            if balance >= 0 {
                occ ^= 1u64 << bishop_attackers.trailing_zeros();
                side_to_move = side_to_move.opposite();
                continue;
            }
            return side_to_move != us;
        }
        
        // Rooks
        let rook_attackers = attacks::rook_attacks(to, occ) 
                           & board.pieces[side_to_move as usize][PieceType::Rook as usize] & occ;
        if rook_attackers != 0 {
            balance = -balance - 1 - 500;
            if balance >= 0 {
                occ ^= 1u64 << rook_attackers.trailing_zeros();
                side_to_move = side_to_move.opposite();
                continue;
            }
            return side_to_move != us;
        }
        
        // Queens
        let queen_attackers = attacks::queen_attacks(to, occ) 
                            & board.pieces[side_to_move as usize][PieceType::Queen as usize] & occ;
        if queen_attackers != 0 {
            balance = -balance - 1 - 900;
            if balance >= 0 {
                occ ^= 1u64 << queen_attackers.trailing_zeros();
                side_to_move = side_to_move.opposite();
                continue;
            }
            return side_to_move != us;
        }
        
        // King
        let king_attackers = attacks::king_attacks(to) 
                           & board.pieces[side_to_move as usize][PieceType::King as usize] & occ;
        if king_attackers != 0 {
            // King capture: only valid if opponent can't recapture
            return side_to_move != us;
        }
        
        // No more attackers — current side to move loses
        break;
    }
    
    // Side to move has no recapture, other side wins
    side_to_move != us
}

pub struct Search {
    pub nodes: u64,
    pub tt: TranspositionTable,
    pub start_time: Instant,
    pub time_limit: Duration,
    pub abort_search: bool,
    
    // Killer moves: 2 per ply
    pub killers: [[Move; 2]; MAX_PLY],
    
    // History heuristic: [color][from][to]
    pub history: [[[i32; 64]; 64]; 2],
    
    // Countermove heuristic: [prev_piece][prev_to] -> counter
    pub countermoves: [[Move; 64]; 6],
    
    // Search generation for TT aging
    pub search_generation: u8,
    
    // PV tracking
    pub seldepth: u8,
    
    // Position history for repetition detection
    pub position_history: Vec<u64>,
}

impl Search {
    pub fn new() -> Self {
        init_lmr();
        Search { 
            nodes: 0,
            tt: TranspositionTable::new(64), // 64 MB TT
            start_time: Instant::now(),
            time_limit: Duration::from_secs(u64::MAX),
            abort_search: false,
            killers: [[Move(0); 2]; MAX_PLY],
            history: [[[0; 64]; 64]; 2],
            countermoves: [[Move(0); 64]; 6],
            search_generation: 0,
            seldepth: 0,
            position_history: Vec::with_capacity(512),
        }
    }

    fn check_time(&mut self) {
        if self.nodes & 4095 == 0 {
            if self.start_time.elapsed() >= self.time_limit {
                self.abort_search = true;
            }
        }
    }

    // Reset heuristics between searches (not games)
    fn clear_heuristics(&mut self) {
        self.killers = [[Move(0); 2]; MAX_PLY];
        // Age history table (divide by 2 to retain some knowledge)
        for c in 0..2 {
            for f in 0..64 {
                for t in 0..64 {
                    self.history[c][f][t] /= 2;
                }
            }
        }
        self.countermoves = [[Move(0); 64]; 6];
    }

    pub fn search(&mut self, board: &mut Board, depth: u8) -> (i32, Option<Move>) {
        self.nodes = 0;
        self.start_time = Instant::now();
        self.time_limit = Duration::from_secs(u64::MAX);
        self.abort_search = false;
        self.clear_heuristics();
        
        let mut best_move = None;
        let score = self.alpha_beta(board, depth, 0, -INFINITY, INFINITY, &mut best_move, Move(0));
        (score, best_move)
    }

    pub fn iterative_deepening(&mut self, board: &mut Board, max_depth: u8, time_limit: Duration) -> (i32, Option<Move>) {
        self.nodes = 0;
        self.start_time = Instant::now();
        self.time_limit = time_limit;
        self.abort_search = false;
        self.search_generation = self.search_generation.wrapping_add(1);
        self.clear_heuristics();
        
        let mut best_move = None;
        let mut best_score = 0;
        
        for d in 1..=max_depth {
            self.seldepth = 0;
            
            // Aspiration windows for depth >= 4
            let (mut alpha, mut beta) = if d >= 4 {
                (best_score - 25, best_score + 25)
            } else {
                (-INFINITY, INFINITY)
            };
            
            let mut current_best;
            let mut score;
            
            // Aspiration window loop with widening
            loop {
                current_best = None;
                score = self.alpha_beta(board, d, 0, alpha, beta, &mut current_best, Move(0));
                
                if self.abort_search && d > 1 {
                    break;
                }
                
                // Widen window on fail-low or fail-high
                if score <= alpha {
                    alpha = (alpha - 100).max(-INFINITY);
                    beta = (alpha + beta) / 2 + 1; // Re-center slightly
                    continue;
                }
                if score >= beta {
                    beta = (beta + 100).min(INFINITY);
                    continue;
                }
                break;
            }
            
            if self.abort_search && d > 1 {
                break;
            }

            best_score = score;
            if current_best.is_some() {
                best_move = current_best;
            }
            
            // Info output (UCI)
            let elapsed = self.start_time.elapsed();
            let nps = if elapsed.as_millis() > 0 {
                self.nodes as u128 * 1000 / elapsed.as_millis()
            } else {
                0
            };
            
            // Mate score display
            let score_str = if best_score.abs() >= MATE_SCORE - 1000 {
                let mate_in = if best_score > 0 {
                    (MATE_SCORE - best_score + 1) / 2
                } else {
                    -(MATE_SCORE + best_score + 1) / 2
                };
                format!("mate {}", mate_in)
            } else {
                format!("cp {}", best_score)
            };

            println!(
                "info depth {} seldepth {} score {} nodes {} nps {} time {} hashfull {}",
                d, self.seldepth, score_str, self.nodes, nps,
                elapsed.as_millis(), self.tt.hashfull()
            );
            use std::io::Write;
            std::io::stdout().flush().ok();
            
            // If mate found, stop early
            if best_score.abs() >= MATE_SCORE - 1000 {
                break;
            }
            
            // Time management: if we've used > 50% of our time, don't start a new iteration
            if elapsed >= time_limit / 2 {
                break;
            }
        }
        
        if best_move.is_none() {
            let mut fallback_list = MoveList::new();
            generate_moves(board, &mut fallback_list);
            for i in 0..fallback_list.len() {
                let m = fallback_list[i];
                if let Some(undo) = board.make_move(m) {
                    board.unmake_move(m, undo);
                    best_move = Some(m);
                    break;
                }
            }
        }
        
        (best_score, best_move)
    }

    // Move ordering scores
    const TT_MOVE_SCORE: i32 = 10_000_000;
    const GOOD_CAPTURE_BASE: i32 = 8_000_000;
    const BAD_CAPTURE_BASE: i32 = -2_000_000;
    const KILLER1_SCORE: i32 = 6_000_000;
    const KILLER2_SCORE: i32 = 5_900_000;
    const COUNTER_SCORE: i32 = 5_800_000;
    // History moves get scores from 0 to MAX_HIST

    fn score_move(&self, board: &Board, m: Move, ply: usize, tt_move: Option<Move>, prev_move: Move) -> i32 {
        // TT move gets highest priority
        if Some(m) == tt_move {
            return Self::TT_MOVE_SCORE;
        }
        
        if m.is_capture() {
            let attacker = board.piece_type_on(m.from(), board.side_to_move).unwrap_or(PieceType::Pawn);
            let victim = board.piece_type_on(m.to(), board.side_to_move.opposite()).unwrap_or(PieceType::Pawn);
            let mvv_lva = piece_value_see(victim) * 10 - piece_value_see(attacker);
            
            // Use SEE to separate good and bad captures
            if see_ge(board, m, 0) {
                return Self::GOOD_CAPTURE_BASE + mvv_lva;
            } else {
                return Self::BAD_CAPTURE_BASE + mvv_lva;
            }
        }
        
        if m.is_promotion() {
            return Self::GOOD_CAPTURE_BASE + 500;
        }
        
        // Killer moves
        if ply < MAX_PLY {
            if m == self.killers[ply][0] {
                return Self::KILLER1_SCORE;
            }
            if m == self.killers[ply][1] {
                return Self::KILLER2_SCORE;
            }
        }
        
        // Countermove
        if prev_move.0 != 0 {
            let prev_to = prev_move.to().0 as usize;
            if let Some(prev_pt) = board.piece_type_on(prev_move.to(), board.side_to_move) {
                if m == self.countermoves[prev_pt as usize][prev_to] {
                    return Self::COUNTER_SCORE;
                }
            }
        }
        
        // History heuristic
        let side = board.side_to_move as usize;
        self.history[side][m.from().0 as usize][m.to().0 as usize]
    }
    
    // Update killer/history/countermove on a beta cutoff
    fn update_quiet_stats(&mut self, board: &Board, m: Move, depth: u8, ply: usize, prev_move: Move, searched_quiets: &[Move]) {
        let side = board.side_to_move as usize;
        let bonus = (depth as i32 * depth as i32).min(400);
        
        // Update killer moves
        if ply < MAX_PLY && m != self.killers[ply][0] {
            self.killers[ply][1] = self.killers[ply][0];
            self.killers[ply][0] = m;
        }
        
        // Update history for the best move (bonus)
        let from = m.from().0 as usize;
        let to = m.to().0 as usize;
        self.history[side][from][to] += bonus - bonus * self.history[side][from][to] / MAX_HIST;
        
        // Penalize quiet moves that didn't cause cutoff (malus)
        for &q in searched_quiets {
            if q != m {
                let qf = q.from().0 as usize;
                let qt = q.to().0 as usize;
                self.history[side][qf][qt] -= bonus - bonus * self.history[side][qf][qt].abs() / MAX_HIST;
            }
        }
        
        // Countermove update
        if prev_move.0 != 0 {
            if let Some(prev_pt) = board.piece_type_on(prev_move.to(), board.side_to_move) {
                self.countermoves[prev_pt as usize][prev_move.to().0 as usize] = m;
            }
        }
    }

    fn alpha_beta(
        &mut self, 
        board: &mut Board, 
        mut depth: u8, 
        ply: u8, 
        mut alpha: i32, 
        mut beta: i32, 
        best_move_out: &mut Option<Move>,
        prev_move: Move,
    ) -> i32 {
        self.check_time();
        if self.abort_search {
            return 0;
        }

        // Update seldepth
        if ply > self.seldepth {
            self.seldepth = ply;
        }

        // Prevent overflow
        if ply >= MAX_PLY as u8 - 1 {
            return evaluate(board);
        }

        // Draw detection: repetition and 50-move rule
        if ply > 0 {
            // 50-move rule
            if board.halfmove_clock >= 100 {
                return 0;
            }
            // Repetition detection: check if current position appeared before
            let key = board.zobrist_key;
            let hist_len = self.position_history.len();
            let lookback = (board.halfmove_clock as usize).min(hist_len);
            for i in (0..lookback).rev() {
                if self.position_history[hist_len - 1 - i] == key {
                    return 0; // Draw by repetition
                }
            }
        }

        let is_root = ply == 0;
        let in_check = board.is_in_check(board.side_to_move);
        let is_pv = beta - alpha > 1;
        
        // Check extension: extend when in check
        if in_check {
            depth = depth.saturating_add(1);
        }

        let alpha_orig = alpha;

        // Probe TT
        let mut tt_move = None;
        if let Some(entry) = self.tt.probe(board.zobrist_key) {
            tt_move = Some(entry.best_move);
            // Validate TT move: skip if clearly invalid (null move or out of range)
            if entry.best_move.0 == 0 { tt_move = None; }
            
            if !is_pv && entry.depth >= depth {
                let tt_score = entry.score as i32;
                match entry.node_type() {
                    NodeType::Exact => {
                        if is_root { *best_move_out = Some(entry.best_move); }
                        return tt_score;
                    },
                    NodeType::LowerBound => {
                        if tt_score > alpha { alpha = tt_score; }
                    },
                    NodeType::UpperBound => {
                        if tt_score < beta { beta = tt_score; }
                    }
                }
                if alpha >= beta {
                    if is_root { *best_move_out = Some(entry.best_move); }
                    return tt_score;
                }
            }
        }

        // Internal Iterative Deepening (IID)
        // When we have no TT move at a PV node, do a shallow search first
        if is_pv && tt_move.is_none() && depth >= 4 && !in_check {
            let mut dummy = None;
            self.alpha_beta(board, depth - 2, ply, alpha, beta, &mut dummy, prev_move);
            // Now probe TT again for a move hint
            if let Some(entry) = self.tt.probe(board.zobrist_key) {
                if entry.best_move.0 != 0 {
                    tt_move = Some(entry.best_move);
                }
            }
        }

        if depth == 0 {
            return self.quiescence(board, alpha, beta, ply);
        }

        self.nodes += 1;
        
        // Static evaluation for pruning decisions
        let static_eval = if in_check { -INFINITY } else { evaluate(board) };
        
        // Reverse Futility Pruning (Static NMP)
        // If our position is so good that even with a margin we're above beta,
        // we can safely prune this node
        if !is_pv && !in_check && depth <= 8 {
            let rfp_margin = 80 * depth as i32;
            if static_eval - rfp_margin >= beta {
                return static_eval - rfp_margin;
            }
        }

        // Null Move Pruning
        // If skipping our turn still results in a beta cutoff, the position is likely winning
        if !is_pv && !in_check && depth >= 3 && static_eval >= beta {
            // Don't null move in positions with few pieces (zugzwang risk)
            let us = board.side_to_move;
            let our_pieces = board.colors[us as usize];
            let our_pawns = board.pieces[us as usize][PieceType::Pawn as usize];
            let our_king = board.pieces[us as usize][PieceType::King as usize];
            let has_non_pawn_material = (our_pieces & !(our_pawns | our_king)) != 0;
            
            if has_non_pawn_material {
                let r = 3 + depth as i32 / 3 + ((static_eval - beta) / 200).min(3);
                let null_depth = depth.saturating_sub(r as u8);
                
                // Make null move
                let old_ep = board.en_passant;
                let old_key = board.zobrist_key;
                if let Some(ep) = board.en_passant {
                    board.zobrist_key ^= crate::zobrist::ZOBRIST.en_passant[ep.file() as usize];
                }
                board.en_passant = None;
                board.side_to_move = board.side_to_move.opposite();
                board.zobrist_key ^= crate::zobrist::ZOBRIST.side_to_move;
                
                let mut dummy = None;
                let null_score = -self.alpha_beta(board, null_depth, ply + 1, -beta, -beta + 1, &mut dummy, Move(0));
                
                // Unmake null move
                board.side_to_move = board.side_to_move.opposite();
                board.en_passant = old_ep;
                board.zobrist_key = old_key;
                
                if self.abort_search { return 0; }
                
                if null_score >= beta {
                    // Don't return unproven mate scores
                    if null_score >= MATE_SCORE - 1000 {
                        return beta;
                    }
                    return null_score;
                }
            }
        }

        // Razoring: at low depths, if static eval is far below alpha, drop into qsearch
        if !is_pv && !in_check && depth <= 3 {
            let razor_margin = 300 + 200 * depth as i32;
            if static_eval + razor_margin <= alpha {
                let qscore = self.quiescence(board, alpha, beta, ply);
                if qscore <= alpha {
                    return qscore;
                }
            }
        }

        // Generate and score moves
        let mut move_list = MoveList::new();
        generate_moves(board, &mut move_list);

        let mut scores = [0i32; crate::movegen::MAX_MOVES];
        for i in 0..move_list.len() {
            scores[i] = self.score_move(board, move_list[i], ply as usize, tt_move, prev_move);
        }

        let mut legal_moves = 0;
        let mut best_score = -INFINITY;
        let mut best_move = None;
        let mut searched_quiets: Vec<Move> = Vec::new();

        for i in 0..move_list.len() {
            // Selection sort: pick the best-scored move
            let mut best_idx = i;
            for j in (i + 1)..move_list.len() {
                if scores[j] > scores[best_idx] {
                    best_idx = j;
                }
            }
            
            // Swap
            scores.swap(i, best_idx);
            let m = move_list[best_idx];
            move_list[best_idx] = move_list[i];
            move_list[i] = m;

            if let Some(undo) = board.make_move(m) {
                legal_moves += 1;
                
                let gives_check = board.is_in_check(board.side_to_move);
                let is_tactical = m.is_capture() || m.is_promotion();
                let is_quiet = !is_tactical;
                
                // Track searched quiet moves for history malus
                if is_quiet {
                    searched_quiets.push(m);
                }

                // SEE Pruning: skip captures that lose material at low depths
                if !is_pv && !in_check && depth <= 7 && m.is_capture() {
                    if !see_ge(board, m, -20 * depth as i32 * depth as i32) {
                        board.unmake_move(m, undo);
                        continue;
                    }
                }

                // Late Move Pruning (LMP): skip quiet moves at low depths
                if !is_pv && !in_check && is_quiet && depth <= 8 && !gives_check {
                    let lmp_threshold = if depth <= 5 {
                        3 + depth as usize * depth as usize
                    } else {
                        5 + depth as usize * depth as usize / 2
                    };
                    if legal_moves > lmp_threshold {
                        board.unmake_move(m, undo);
                        continue;
                    }
                }

                // Futility Pruning: at low depth, if static eval + margin < alpha, skip quiet
                if !is_pv && !in_check && is_quiet && depth <= 8 && !gives_check {
                    let futility_margin = 120 * depth as i32;
                    if static_eval + futility_margin <= alpha {
                        board.unmake_move(m, undo);
                        // Track the best score we'd get from stand-pat
                        if static_eval + futility_margin > best_score {
                            best_score = static_eval + futility_margin;
                        }
                        continue;
                    }
                }

                let score = if legal_moves == 1 {
                    // First move: search with full window
                    let mut dummy = None;
                    -self.alpha_beta(board, depth - 1, ply + 1, -beta, -alpha, &mut dummy, m)
                } else {
                    let mut s;
                    let mut needs_full_search = true;

                    // Late Move Reduction (LMR)
                    if depth >= 3 && legal_moves >= 3 && !in_check {
                        let mut r = lmr_reduction(depth as usize, legal_moves);
                        
                        // Reduce less for: killers, good captures, checks, PV nodes
                        if m.is_capture() && see_ge(board, m, 0) { r -= 1; }
                        if gives_check { r -= 1; }
                        if is_pv { r -= 1; }
                        
                        // Reduce more for bad captures and moves with bad history
                        if m.is_capture() && !see_ge(board, m, 0) { r += 1; }
                        
                        // Reduce more for: moves with bad history
                        if is_quiet {
                            let hist = self.history[board.side_to_move.opposite() as usize][m.from().0 as usize][m.to().0 as usize];
                            if hist < -100 { r += 1; }
                        }
                        
                        r = r.clamp(0, depth as i32 - 2);
                        
                        let reduced_depth = (depth as i32 - 1 - r) as u8;
                        let mut dummy = None;
                        s = -self.alpha_beta(board, reduced_depth, ply + 1, -alpha - 1, -alpha, &mut dummy, m);
                        
                        needs_full_search = s > alpha;
                    }

                    if needs_full_search {
                        // Zero-window search (PVS)
                        let mut dummy = None;
                        s = -self.alpha_beta(board, depth - 1, ply + 1, -alpha - 1, -alpha, &mut dummy, m);
                        
                        // Re-search with full window if it beats alpha
                        if s > alpha && s < beta {
                            s = -self.alpha_beta(board, depth - 1, ply + 1, -beta, -alpha, &mut dummy, m);
                        }
                    } else {
                        s = alpha; // Will not improve, already set by LMR
                    }
                    s
                };
                
                board.unmake_move(m, undo);
                
                if self.abort_search {
                    return 0;
                }

                if score > best_score {
                    best_score = score;
                    best_move = Some(m);
                    
                    if score > alpha {
                        alpha = score;
                        if alpha >= beta {
                            // Beta cutoff - update killer/history/countermove for quiet moves
                            if is_quiet {
                                self.update_quiet_stats(board, m, depth, ply as usize, prev_move, &searched_quiets);
                            }
                            break;
                        }
                    }
                }
            }
        }

        if legal_moves == 0 {
            if in_check {
                return -MATE_SCORE + ply as i32; // Checkmate
            } else {
                return 0; // Stalemate
            }
        }

        if is_root {
            *best_move_out = best_move;
        }

        // Store in TT
        let node_type = if best_score <= alpha_orig {
            NodeType::UpperBound
        } else if best_score >= beta {
            NodeType::LowerBound
        } else {
            NodeType::Exact
        };

        if let Some(m) = best_move {
            self.tt.store(TTEntry::new(
                board.zobrist_key, m, best_score as i16, depth, node_type, self.search_generation
            ));
        }

        best_score
    }

    fn quiescence(
        &mut self,
        board: &mut Board,
        mut alpha: i32,
        beta: i32,
        ply: u8,
    ) -> i32 {
        self.nodes += 1;
        self.check_time();
        if self.abort_search {
            return 0;
        }

        if ply > self.seldepth {
            self.seldepth = ply;
        }

        if ply >= MAX_PLY as u8 - 1 {
            return evaluate(board);
        }

        let in_check = board.is_in_check(board.side_to_move);
        
        let stand_pat = if in_check { -INFINITY } else { evaluate(board) };
        
        if !in_check {
            if stand_pat >= beta {
                return beta;
            }
            
            // Delta pruning: if even capturing the best piece can't raise us to alpha,
            // we can probably prune this node
            let delta = 1000; // Queen value + margin
            if stand_pat + delta < alpha {
                return alpha;
            }
            
            if alpha < stand_pat {
                alpha = stand_pat;
            }
        }

        let mut move_list = MoveList::new();
        generate_moves(board, &mut move_list);

        // Score captures by MVV-LVA
        let mut scores = [0i32; crate::movegen::MAX_MOVES];
        for i in 0..move_list.len() {
            if move_list[i].is_capture() {
                let attacker = board.piece_type_on(move_list[i].from(), board.side_to_move).unwrap_or(PieceType::Pawn);
                let victim = board.piece_type_on(move_list[i].to(), board.side_to_move.opposite()).unwrap_or(PieceType::Pawn);
                scores[i] = piece_value_see(victim) * 10 - piece_value_see(attacker);
            }
        }

        let mut legal_moves = 0;
        let mut best_score = stand_pat;

        for i in 0..move_list.len() {
            // Selection sort
            let mut best_idx = i;
            for j in (i + 1)..move_list.len() {
                if scores[j] > scores[best_idx] {
                    best_idx = j;
                }
            }
            
            scores.swap(i, best_idx);
            let m = move_list[best_idx];
            move_list[best_idx] = move_list[i];
            move_list[i] = m;

            // In QS, only search tactical moves (captures, promotions), UNLESS in check
            if !in_check && !m.is_capture() && !m.is_promotion() {
                continue;
            }
            
            // SEE pruning: skip captures that lose material
            if !in_check && m.is_capture() && !see_ge(board, m, 0) {
                continue;
            }

            // Delta pruning per-move: skip captures that can't possibly raise alpha
            if !in_check && m.is_capture() {
                let victim = board.piece_type_on(m.to(), board.side_to_move.opposite()).unwrap_or(PieceType::Pawn);
                let gain = piece_value_see(victim) + 200; // 200cp margin
                if stand_pat + gain < alpha {
                    continue;
                }
            }

            if let Some(undo) = board.make_move(m) {
                legal_moves += 1;
                let score = -self.quiescence(board, -beta, -alpha, ply + 1);
                board.unmake_move(m, undo);

                if score > best_score {
                    best_score = score;
                }
                if score >= beta {
                    return beta;
                }
                if score > alpha {
                    alpha = score;
                }
            }
        }

        if in_check && legal_moves == 0 {
            return -MATE_SCORE + ply as i32;
        }

        best_score.max(stand_pat)
    }
}
