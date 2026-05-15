use crate::board::{Board, Move, PieceType, Square};
use crate::evaluation::evaluate;
use crate::movegen::{MoveList, generate_moves};
use crate::transposition::{NodeType, TTEntry, TranspositionTable};
use std::time::{Duration, Instant};

pub const MAX_GAME_PLY: usize = 1024;

pub const INFINITY: i32 = 30000;
pub const MATE_SCORE: i32 = 29000;
pub const MAX_PLY: usize = 128;
pub const MAX_HIST: i32 = 16384;

// ---------------------------------------------------------------------------
// LMR table — computed once during Search::new(), stored in the struct.
// ---------------------------------------------------------------------------

fn build_lmr_table() -> [[i32; 64]; 64] {
    let mut table = [[0i32; 64]; 64];
    for depth in 1..64 {
        for move_num in 1..64 {
            table[depth][move_num] =
                (0.75 + (depth as f64).ln() * (move_num as f64).ln() / 2.25) as i32;
        }
    }
    table
}

#[inline]
fn lmr_reduction(table: &[[i32; 64]; 64], depth: usize, move_num: usize) -> i32 {
    table[depth.min(63)][move_num.min(63)]
}

// ---------------------------------------------------------------------------
// Piece values for SEE
// ---------------------------------------------------------------------------
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
#[inline(never)]
pub fn see_ge(board: &Board, m: Move, threshold: i32) -> bool {
    use crate::attacks;
    use crate::board::Color;

    let from = m.from();
    let to = m.to();
    let us = board.side_to_move;

    // Get the initial balance: value of captured piece - threshold
    let victim_pt = board.piece_type_on(to, us.opposite());
    let mut balance = if let Some(vpt) = victim_pt {
        piece_value_see(vpt) - threshold
    } else {
        // En passant captures a pawn
        if m.flags() == 5 {
            100 - threshold
        } else {
            -threshold
        }
    };

    // If even capturing for free doesn't meet threshold, fail
    if balance < 0 {
        return false;
    }

    // Value of our piece that just captured
    let attacker_pt = board.piece_type_on(from, us).unwrap_or(PieceType::Pawn);

    // Worst case: we lose our piece
    balance -= piece_value_see(attacker_pt);

    // If even losing our piece still meets threshold, succeed
    if balance >= 0 {
        return true;
    }

    // Now simulate the swap sequence
    let mut occ = (board.colors[0] | board.colors[1]) ^ (1u64 << from.0) ^ (1u64 << to.0);

    // Handle en passant - remove the captured pawn too
    if m.flags() == 5 {
        let ep_pawn_sq = if us == Color::White {
            to.0 - 8
        } else {
            to.0 + 8
        };
        occ ^= 1u64 << ep_pawn_sq;
    }

    let mut side_to_move = us.opposite(); // opponent's turn to recapture

    loop {
        // Find the least valuable attacker for `side_to_move` to the `to` square

        // Pawns
        let pawn_attackers = attacks::pawn_attacks(side_to_move.opposite(), to)
            & board.pieces[side_to_move as usize][PieceType::Pawn as usize]
            & occ;
        if pawn_attackers != 0 {
            balance = -balance - 1 - 100;
            if balance >= 0 {
                occ ^= 1u64 << pawn_attackers.trailing_zeros();
                side_to_move = side_to_move.opposite();
                continue;
            }
            return side_to_move != us;
        }

        // Knights
        let knight_attackers = attacks::knight_attacks(to)
            & board.pieces[side_to_move as usize][PieceType::Knight as usize]
            & occ;
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
            & board.pieces[side_to_move as usize][PieceType::Bishop as usize]
            & occ;
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
            & board.pieces[side_to_move as usize][PieceType::Rook as usize]
            & occ;
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
            & board.pieces[side_to_move as usize][PieceType::Queen as usize]
            & occ;
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
            & board.pieces[side_to_move as usize][PieceType::King as usize]
            & occ;
        if king_attackers != 0 {
            return side_to_move != us;
        }

        // No more attackers — current side to move loses
        break;
    }

    side_to_move != us
}

// ---------------------------------------------------------------------------
// Search struct
// ---------------------------------------------------------------------------

pub struct Search {
    pub nodes: u64,
    pub tt: TranspositionTable,
    pub start_time: Instant,
    pub time_limit: Duration,
    pub abort_search: bool,

    /// Killer moves: 2 per ply
    pub killers: Box<[[Move; 2]; MAX_PLY]>,

    /// History heuristic: [color][from][to]
    pub history: Box<[[[i32; 64]; 64]; 2]>,

    /// Countermove heuristic: [prev_piece][prev_to] -> counter
    pub countermoves: [[Move; 64]; 6],

    /// Search generation for TT aging
    pub search_generation: u8,

    /// PV tracking (seldepth)
    pub seldepth: u8,

    /// Game position history for repetition detection (filled by UCI layer)
    pub position_history: Vec<u64>,

    /// LMR reduction table computed at init time
    lmr_table: Box<[[i32; 64]; 64]>,

    /// PV table and lengths (heap-allocated to avoid stack pressure)
    pub pv_table: Box<[[Move; MAX_PLY]; MAX_PLY]>,
    pub pv_length: Box<[usize; MAX_PLY]>,

    /// Continuation history: [prev_pt][prev_to][cur_pt][cur_to]
    pub cont_hist: Box<[[[[i32; 64]; 6]; 64]; 6]>,

    /// Best move from previous iteration (for time management)
    last_best_move: Option<Move>,

    /// Contempt factor in centipawns (positive = avoid draws, play for the win).
    /// A value of 30 means the engine treats a draw as worth -30 cp,
    /// so it keeps pressing even in equal positions.
    pub contempt: i32,

    /// The side the engine is playing as (set at the start of each search).
    root_color: crate::board::Color,

    /// Weak-opponent mode: push harder for wins, especially in endgames.
    pub weak_opponent_mode: bool,
    pub weak_contempt_bonus: i32,
    pub weak_endgame_draw_bias: i32,

    /// Root safety filter: re-check top-N root lines and keep the safest.
    pub self_check_filter: bool,
    pub multipv_lines: u8,
}

impl Search {
    pub fn new() -> Self {
        Search {
            nodes: 0,
            tt: TranspositionTable::new(64), // 64 MB TT
            start_time: Instant::now(),
            time_limit: Duration::from_secs(u64::MAX),
            abort_search: false,
            killers: Box::new([[Move(0); 2]; MAX_PLY]),
            history: unsafe {
                let mut b: Box<[[[i32; 64]; 64]; 2]> = Box::new_uninit().assume_init();
                for color in b.iter_mut() {
                    for row in color.iter_mut() {
                        for cell in row.iter_mut() {
                            *cell = 0;
                        }
                    }
                }
                b
            },
            countermoves: [[Move(0); 64]; 6],
            search_generation: 0,
            seldepth: 0,
            position_history: Vec::with_capacity(512),
            lmr_table: Box::new(build_lmr_table()),
            pv_table: unsafe {
                let mut b: Box<[[Move; MAX_PLY]; MAX_PLY]> = Box::new_uninit().assume_init();
                for row in b.iter_mut() {
                    for cell in row.iter_mut() {
                        *cell = Move(0);
                    }
                }
                b
            },
            pv_length: Box::new([0usize; MAX_PLY]),
            cont_hist: unsafe {
                let mut b: Box<[[[[i32; 64]; 6]; 64]; 6]> = Box::new_uninit().assume_init();
                for a in b.iter_mut() {
                    for bb in a.iter_mut() {
                        for c in bb.iter_mut() {
                            for d in c.iter_mut() {
                                *d = 0;
                            }
                        }
                    }
                }
                b
            },
            last_best_move: None,
            contempt: 30,
            root_color: crate::board::Color::White,
            weak_opponent_mode: false,
            weak_contempt_bonus: 20,
            weak_endgame_draw_bias: 20,
            self_check_filter: false,
            multipv_lines: 3,
        }
    }

    #[inline]
    fn is_endgame(&self, board: &Board) -> bool {
        let mut npm = 0i32;
        for c in 0..2 {
            npm += board.pieces[c][PieceType::Knight as usize].count_ones() as i32 * 300;
            npm += board.pieces[c][PieceType::Bishop as usize].count_ones() as i32 * 320;
            npm += board.pieces[c][PieceType::Rook as usize].count_ones() as i32 * 500;
            npm += board.pieces[c][PieceType::Queen as usize].count_ones() as i32 * 900;
        }
        npm <= 2600
    }

    #[inline]
    fn effective_contempt(&self, board: &Board) -> i32 {
        let mut c = self.contempt;
        if self.weak_opponent_mode {
            c += self.weak_contempt_bonus;
            if self.is_endgame(board) {
                c += self.weak_endgame_draw_bias;
            }
        }
        c.clamp(-200, 200)
    }

    /// Returns the score for a drawn position, accounting for contempt.
    /// If the engine is the side to move it dislikes the draw (-contempt),
    /// so it keeps pressing for a win rather than repeating.
    #[inline]
    fn draw_score(&self, board: &Board) -> i32 {
        let contempt = self.effective_contempt(board);
        if board.side_to_move == self.root_color {
            -contempt
        } else {
            contempt
        }
    }

    /// MultiPV root safety filter: verify top-N candidates at reduced depth
    /// and pick the move with the best verified score.
    #[inline(never)]
    fn root_multipv_filter(
        &mut self,
        board: &mut Board,
        depth: u8,
        current_best: Option<Move>,
    ) -> Option<Move> {
        if !self.self_check_filter || depth < 4 {
            return current_best;
        }

        let mut move_list = MoveList::new();
        generate_moves(board, &mut move_list);

        // Keep only legal root moves.
        let mut legal: Vec<Move> = Vec::with_capacity(move_list.len());
        for i in 0..move_list.len() {
            let m = move_list[i];
            if let Some(undo) = board.make_move(m) {
                board.unmake_move(m, undo);
                legal.push(m);
            }
        }
        if legal.is_empty() {
            return current_best;
        }

        let tt_move = self.tt.probe(board.zobrist_key).map(|e| e.best_move);
        let mut ranked: Vec<(i32, Move)> = legal
            .iter()
            .map(|&m| (self.score_move(board, m, 0, tt_move, Move(0)), m))
            .collect();
        ranked.sort_by(|a, b| b.0.cmp(&a.0));

        let mut candidates: Vec<Move> = ranked
            .iter()
            .take(self.multipv_lines.clamp(1, 8) as usize)
            .map(|&(_, m)| m)
            .collect();

        if let Some(best) = current_best {
            if !candidates.contains(&best) {
                candidates.push(best);
            }
        }

        let verify_depth = depth.saturating_sub(2).max(1);
        let mut best_move = current_best.or_else(|| candidates.first().copied());
        let mut best_score = -INFINITY;

        for m in candidates {
            if self.start_time.elapsed() >= self.time_limit {
                break;
            }

            if let Some(undo) = board.make_move(m) {
                self.position_history.push(board.zobrist_key);
                let mut dummy = None;
                let score =
                    -self.alpha_beta(board, verify_depth, 1, -INFINITY, INFINITY, &mut dummy, m);
                board.unmake_move(m, undo);
                self.position_history.pop();

                if self.abort_search {
                    break;
                }

                if score > best_score {
                    best_score = score;
                    best_move = Some(m);
                }
            }
        }

        best_move
    }

    fn check_time(&mut self) {
        if self.nodes & 4095 == 0 {
            if self.start_time.elapsed() >= self.time_limit {
                self.abort_search = true;
            }
        }
    }

    /// Reset heuristics between searches (not games)
    fn clear_heuristics(&mut self) {
        for row in self.killers.iter_mut() {
            *row = [Move(0); 2];
        }
        // Age history table (divide by 2 to retain some knowledge)
        for c in 0..2 {
            for f in 0..64 {
                for t in 0..64 {
                    self.history[c][f][t] /= 2;
                }
            }
        }
        self.countermoves = [[Move(0); 64]; 6];
        // Age continuation history
        for a in self.cont_hist.iter_mut() {
            for b in a.iter_mut() {
                for c in b.iter_mut() {
                    for d in c.iter_mut() {
                        *d /= 2;
                    }
                }
            }
        }
    }

    pub fn search(&mut self, board: &mut Board, depth: u8) -> (i32, Option<Move>) {
        self.nodes = 0;
        self.start_time = Instant::now();
        self.time_limit = Duration::from_secs(u64::MAX);
        self.abort_search = false;
        self.root_color = board.side_to_move;
        self.clear_heuristics();

        let mut best_move = None;
        let score = self.alpha_beta(
            board,
            depth,
            0,
            -INFINITY,
            INFINITY,
            &mut best_move,
            Move(0),
        );
        (score, best_move)
    }

    #[inline(never)]
    pub fn iterative_deepening(
        &mut self,
        board: &mut Board,
        max_depth: u8,
        time_limit: Duration,
    ) -> (i32, Option<Move>) {
        self.nodes = 0;
        self.start_time = Instant::now();
        self.time_limit = time_limit;
        self.abort_search = false;
        self.search_generation = self.search_generation.wrapping_add(1);
        self.root_color = board.side_to_move;
        self.clear_heuristics();

        let mut best_move: Option<Move> = None;
        let mut best_score = 0i32;
        let mut last_completed_depth = 0u8;

        for d in 1..=max_depth {
            self.seldepth = 0;

            // Aspiration windows for depth >= 4
            let mut delta = 25i32;
            let mut alpha = if d >= 4 {
                best_score - delta
            } else {
                -INFINITY
            };
            let mut beta = if d >= 4 { best_score + delta } else { INFINITY };

            let mut current_best: Option<Move>;
            let mut score: i32;

            // Aspiration window loop with widening
            loop {
                current_best = None;
                score = self.alpha_beta(board, d, 0, alpha, beta, &mut current_best, Move(0));

                if self.abort_search && d > 1 {
                    break;
                }

                if score <= alpha {
                    // Fail-low: widen down
                    beta = (alpha + beta) / 2;
                    alpha = (score - delta).max(-INFINITY);
                    delta = (delta * 2).min(INFINITY);
                    continue;
                }
                if score >= beta {
                    // Fail-high: widen up
                    beta = (score + delta).min(INFINITY);
                    delta = (delta * 2).min(INFINITY);
                    continue;
                }
                break;
            }

            if self.abort_search && d > 1 {
                break;
            }

            // Determine whether the best move changed (for time management)
            let move_changed = current_best.is_some() && current_best != best_move;

            best_score = score;
            if current_best.is_some() {
                best_move = current_best;
            }
            last_completed_depth = d;

            // Build PV string
            let pv_str: String = (0..self.pv_length[0])
                .map(|i| Self::move_to_uci_static(self.pv_table[0][i]))
                .collect::<Vec<_>>()
                .join(" ");

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
                "info depth {} seldepth {} score {} nodes {} nps {} time {} hashfull {} pv {}",
                d,
                self.seldepth,
                score_str,
                self.nodes,
                nps,
                elapsed.as_millis(),
                self.tt.hashfull(),
                pv_str
            );
            use std::io::Write;
            std::io::stdout().flush().ok();

            // If mate found, stop early
            if best_score.abs() >= MATE_SCORE - 1000 {
                break;
            }

            // Time management: allow up to 75% of time if best move changed,
            // otherwise stop at 50%.
            let stop_fraction = if move_changed { 3u32 } else { 2u32 };
            if elapsed >= time_limit / stop_fraction {
                break;
            }

            self.last_best_move = best_move;
        }

        // Optional safety pass: verify top-N root candidates and choose the
        // safest move (helps avoid practical traps in sharp positions).
        if self.self_check_filter && !self.abort_search {
            if self.start_time.elapsed() < self.time_limit {
                best_move = self.root_multipv_filter(board, last_completed_depth, best_move);
            }
        }

        // Fallback: return any legal move if none found
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

    /// Convert a Move to its UCI string representation.
    fn move_to_uci_static(m: Move) -> String {
        fn sq_str(sq: Square) -> String {
            let file = (sq.file() + b'a') as char;
            let rank = (sq.rank() + b'1') as char;
            format!("{}{}", file, rank)
        }
        let promo = match m.promotion_piece_type() {
            Some(PieceType::Queen) => "q",
            Some(PieceType::Rook) => "r",
            Some(PieceType::Bishop) => "b",
            Some(PieceType::Knight) => "n",
            _ => "",
        };
        format!("{}{}{}", sq_str(m.from()), sq_str(m.to()), promo)
    }

    // ---------------------------------------------------------------------------
    // Move ordering scores
    // ---------------------------------------------------------------------------
    const TT_MOVE_SCORE: i32 = 10_000_000;
    const GOOD_CAPTURE_BASE: i32 = 8_000_000;
    const BAD_CAPTURE_BASE: i32 = -2_000_000;
    const KILLER1_SCORE: i32 = 6_000_000;
    const KILLER2_SCORE: i32 = 5_900_000;
    const COUNTER_SCORE: i32 = 5_800_000;
    // History moves get scores from 0 to MAX_HIST

    #[inline(never)]
    fn score_move(
        &self,
        board: &Board,
        m: Move,
        ply: usize,
        tt_move: Option<Move>,
        prev_move: Move,
    ) -> i32 {
        // TT move gets highest priority
        if Some(m) == tt_move {
            return Self::TT_MOVE_SCORE;
        }

        if m.is_capture() {
            let attacker = board
                .piece_type_on(m.from(), board.side_to_move)
                .unwrap_or(PieceType::Pawn);
            let victim = board
                .piece_type_on(m.to(), board.side_to_move.opposite())
                .unwrap_or(PieceType::Pawn);
            let mvv_lva = piece_value_see(victim) * 10 - piece_value_see(attacker);

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
        let mut score = self.history[side][m.from().0 as usize][m.to().0 as usize];

        // Continuation history (1-ply)
        if prev_move.0 != 0 && !m.is_capture() {
            if let Some(prev_pt) =
                board.piece_type_on(prev_move.to(), board.side_to_move.opposite())
            {
                if let Some(cur_pt) = board.piece_type_on(m.from(), board.side_to_move) {
                    score += self.cont_hist[prev_pt as usize][prev_move.to().0 as usize]
                        [cur_pt as usize][m.to().0 as usize];
                }
            }
        }

        score
    }

    /// Update killer/history/countermove/cont_hist on a beta cutoff
    #[inline(never)]
    fn update_quiet_stats(
        &mut self,
        board: &Board,
        m: Move,
        depth: u8,
        ply: usize,
        prev_move: Move,
        searched_quiets: &[Move],
    ) {
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
        self.history[side][from][to] +=
            bonus - bonus * self.history[side][from][to].abs() / MAX_HIST;

        // Penalize quiet moves that didn't cause cutoff (malus)
        for &q in searched_quiets {
            if q != m {
                let qf = q.from().0 as usize;
                let qt = q.to().0 as usize;
                self.history[side][qf][qt] -=
                    bonus - bonus * self.history[side][qf][qt].abs() / MAX_HIST;
            }
        }

        // Countermove update
        if prev_move.0 != 0 {
            if let Some(prev_pt) = board.piece_type_on(prev_move.to(), board.side_to_move) {
                self.countermoves[prev_pt as usize][prev_move.to().0 as usize] = m;
            }
        }

        // Continuation history update
        if prev_move.0 != 0 {
            if let Some(prev_pt) = board.piece_type_on(prev_move.to(), board.side_to_move) {
                // Bonus for the move that caused cutoff
                if let Some(cur_pt) = board.piece_type_on(m.from(), board.side_to_move) {
                    let entry = &mut self.cont_hist[prev_pt as usize][prev_move.to().0 as usize]
                        [cur_pt as usize][m.to().0 as usize];
                    *entry += bonus - bonus * entry.abs() / MAX_HIST;
                }
                // Penalise quiets that didn't cut off
                for &q in searched_quiets {
                    if q != m {
                        if let Some(cur_pt) = board.piece_type_on(q.from(), board.side_to_move) {
                            let entry = &mut self.cont_hist[prev_pt as usize]
                                [prev_move.to().0 as usize][cur_pt as usize]
                                [q.to().0 as usize];
                            *entry -= bonus - bonus * entry.abs() / MAX_HIST;
                        }
                    }
                }
            }
        }
    }

    // ---------------------------------------------------------------------------
    // alpha_beta
    // ---------------------------------------------------------------------------
    #[inline(never)]
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
        // Reset PV length for this ply
        self.pv_length[ply as usize] = 0;

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
                return self.draw_score(board);
            }
            // Repetition detection: check if current position appeared before
            let key = board.zobrist_key;
            let hist_len = self.position_history.len();
            let lookback = (board.halfmove_clock as usize).min(hist_len);
            for i in (0..lookback).rev() {
                if self.position_history[hist_len - 1 - i] == key {
                    return self.draw_score(board); // Draw by repetition
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
            if entry.best_move.0 == 0 {
                tt_move = None;
            }

            if !is_pv && entry.depth >= depth {
                let tt_score = entry.score as i32;
                match entry.node_type() {
                    NodeType::Exact => {
                        if is_root {
                            *best_move_out = Some(entry.best_move);
                        }
                        return tt_score;
                    }
                    NodeType::LowerBound => {
                        if tt_score > alpha {
                            alpha = tt_score;
                        }
                    }
                    NodeType::UpperBound => {
                        if tt_score < beta {
                            beta = tt_score;
                        }
                    }
                }
                if alpha >= beta {
                    if is_root {
                        *best_move_out = Some(entry.best_move);
                    }
                    return tt_score;
                }
            }
        }

        // ---------------------------------------------------------------------------
        // Singular Extensions
        // ---------------------------------------------------------------------------
        let mut singular_extension = false;
        if !is_root && depth >= 8 && tt_move.is_some() {
            if let Some(entry) = self.tt.probe(board.zobrist_key) {
                if entry.node_type() == NodeType::LowerBound
                    && entry.depth as i32 >= depth as i32 - 3
                    && (entry.score as i32).abs() < MATE_SCORE - 1000
                {
                    let s_beta = entry.score as i32 - depth as i32 * 2;
                    let s_depth = (depth - 1) / 2;
                    let mut dummy = None;
                    let s_score = self.alpha_beta(
                        board,
                        s_depth,
                        ply,
                        s_beta - 1,
                        s_beta,
                        &mut dummy,
                        prev_move,
                    );
                    if s_score < s_beta {
                        singular_extension = true;
                    }
                }
            }
        }

        // ---------------------------------------------------------------------------
        // IIR (Internal Iterative Reduction): reduce depth when no TT move
        // ---------------------------------------------------------------------------
        if tt_move.is_none() && depth >= 4 && !in_check {
            depth -= 1;
        }

        if depth == 0 {
            return self.quiescence(board, alpha, beta, ply);
        }

        self.nodes += 1;

        // Static evaluation for pruning decisions
        let static_eval = if in_check { -INFINITY } else { evaluate(board) };

        // Reverse Futility Pruning (Static NMP)
        if !is_pv && !in_check && depth <= 8 {
            let rfp_margin = 80 * depth as i32;
            if static_eval - rfp_margin >= beta {
                return static_eval - rfp_margin;
            }
        }

        // Null Move Pruning
        if !is_pv && !in_check && depth >= 3 && static_eval >= beta {
            let us = board.side_to_move;
            let our_pieces = board.colors[us as usize];
            let our_pawns = board.pieces[us as usize][PieceType::Pawn as usize];
            let our_king = board.pieces[us as usize][PieceType::King as usize];
            let has_non_pawn_material = (our_pieces & !(our_pawns | our_king)) != 0;

            if has_non_pawn_material {
                let r = 3 + depth as i32 / 3 + ((static_eval - beta) / 200).min(3);
                let null_depth = depth.saturating_sub(r as u8);

                let old_ep = board.en_passant;
                let old_key = board.zobrist_key;
                if let Some(ep) = board.en_passant {
                    board.zobrist_key ^= crate::zobrist::ZOBRIST.en_passant[ep.file() as usize];
                }
                board.en_passant = None;
                board.side_to_move = board.side_to_move.opposite();
                board.zobrist_key ^= crate::zobrist::ZOBRIST.side_to_move;

                let mut dummy = None;
                let null_score = -self.alpha_beta(
                    board,
                    null_depth,
                    ply + 1,
                    -beta,
                    -beta + 1,
                    &mut dummy,
                    Move(0),
                );

                board.side_to_move = board.side_to_move.opposite();
                board.en_passant = old_ep;
                board.zobrist_key = old_key;

                if self.abort_search {
                    return 0;
                }

                if null_score >= beta {
                    if null_score >= MATE_SCORE - 1000 {
                        return beta;
                    }
                    return null_score;
                }
            }
        }

        // Razoring
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

        let mut legal_moves = 0usize;
        let mut best_score = -INFINITY;
        let mut best_move = None;

        // Stack-allocated searched-quiets array (avoid heap)
        let mut searched_quiets = [Move(0u16); 64];
        let mut searched_quiets_count = 0usize;

        for i in 0..move_list.len() {
            // Selection sort: pick the best-scored move
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

            // Push board key BEFORE make_move so repetition detection covers
            // in-tree positions correctly.
            self.position_history.push(board.zobrist_key);

            if let Some(undo) = board.make_move(m) {
                legal_moves += 1;

                let gives_check = board.is_in_check(board.side_to_move);
                let is_tactical = m.is_capture() || m.is_promotion();
                let is_quiet = !is_tactical;

                // Track searched quiet moves for history malus
                if is_quiet && searched_quiets_count < 64 {
                    searched_quiets[searched_quiets_count] = m;
                    searched_quiets_count += 1;
                }

                // SEE Pruning: skip captures that lose material at low depths
                if !is_pv && !in_check && depth <= 7 && m.is_capture() {
                    if !see_ge(board, m, -20 * depth as i32 * depth as i32) {
                        board.unmake_move(m, undo);
                        self.position_history.pop();
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
                        self.position_history.pop();
                        continue;
                    }
                }

                // Futility Pruning
                if !is_pv && !in_check && is_quiet && depth <= 8 && !gives_check {
                    let futility_margin = 120 * depth as i32;
                    if static_eval + futility_margin <= alpha {
                        board.unmake_move(m, undo);
                        self.position_history.pop();
                        if static_eval + futility_margin > best_score {
                            best_score = static_eval + futility_margin;
                        }
                        continue;
                    }
                }

                // Determine extension for this move
                let is_tt_move = tt_move == Some(m);
                let extension = if is_tt_move && singular_extension {
                    1u8
                } else {
                    0u8
                };

                let score = if legal_moves == 1 {
                    // First move: full window
                    let mut dummy = None;
                    -self.alpha_beta(
                        board,
                        depth - 1 + extension,
                        ply + 1,
                        -beta,
                        -alpha,
                        &mut dummy,
                        m,
                    )
                } else {
                    let mut s;
                    let mut needs_full_search = true;

                    // Late Move Reduction (LMR)
                    if depth >= 3 && legal_moves >= 3 && !in_check {
                        let mut r = lmr_reduction(&self.lmr_table, depth as usize, legal_moves);

                        if m.is_capture() && see_ge(board, m, 0) {
                            r -= 1;
                        }
                        if gives_check {
                            r -= 1;
                        }
                        if is_pv {
                            r -= 1;
                        }
                        if m.is_capture() && !see_ge(board, m, 0) {
                            r += 1;
                        }
                        if is_quiet {
                            let hist = self.history[board.side_to_move.opposite() as usize]
                                [m.from().0 as usize][m.to().0 as usize];
                            if hist < -100 {
                                r += 1;
                            }
                        }

                        r = r.clamp(0, depth as i32 - 2);

                        let reduced_depth = (depth as i32 - 1 - r + extension as i32) as u8;
                        let mut dummy = None;
                        s = -self.alpha_beta(
                            board,
                            reduced_depth,
                            ply + 1,
                            -alpha - 1,
                            -alpha,
                            &mut dummy,
                            m,
                        );

                        needs_full_search = s > alpha;
                    }

                    if needs_full_search {
                        let mut dummy = None;
                        s = -self.alpha_beta(
                            board,
                            depth - 1 + extension,
                            ply + 1,
                            -alpha - 1,
                            -alpha,
                            &mut dummy,
                            m,
                        );
                        if s > alpha && s < beta {
                            s = -self.alpha_beta(
                                board,
                                depth - 1 + extension,
                                ply + 1,
                                -beta,
                                -alpha,
                                &mut dummy,
                                m,
                            );
                        }
                    } else {
                        s = alpha; // Won't improve; set by LMR
                    }
                    s
                };

                board.unmake_move(m, undo);
                self.position_history.pop();

                if self.abort_search {
                    return 0;
                }

                if score > best_score {
                    best_score = score;
                    best_move = Some(m);

                    if score > alpha {
                        alpha = score;

                        // Update PV table at PV nodes
                        if is_pv {
                            let p = ply as usize;
                            self.pv_table[p][0] = m;
                            let next_ply = p + 1;
                            let next_len = self.pv_length[next_ply];
                            for k in 0..next_len {
                                self.pv_table[p][k + 1] = self.pv_table[next_ply][k];
                            }
                            self.pv_length[p] = next_len + 1;
                        }

                        if alpha >= beta {
                            // Beta cutoff
                            if is_quiet {
                                self.update_quiet_stats(
                                    board,
                                    m,
                                    depth,
                                    ply as usize,
                                    prev_move,
                                    &searched_quiets[..searched_quiets_count],
                                );
                            }
                            break;
                        }
                    }
                }
            } else {
                // Illegal move — pop the key we pushed
                self.position_history.pop();
            }
        }

        if legal_moves == 0 {
            if in_check {
                return -MATE_SCORE + ply as i32; // Checkmate
            } else {
                return self.draw_score(board); // Stalemate
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
                board.zobrist_key,
                m,
                best_score as i16,
                depth,
                node_type,
                self.search_generation,
            ));
        }

        best_score
    }

    // ---------------------------------------------------------------------------
    // Quiescence search
    // ---------------------------------------------------------------------------
    #[inline(never)]
    fn quiescence(&mut self, board: &mut Board, mut alpha: i32, beta: i32, ply: u8) -> i32 {
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

            // Delta pruning: if even capturing the best piece can't raise us to alpha
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
                let attacker = board
                    .piece_type_on(move_list[i].from(), board.side_to_move)
                    .unwrap_or(PieceType::Pawn);
                let victim = board
                    .piece_type_on(move_list[i].to(), board.side_to_move.opposite())
                    .unwrap_or(PieceType::Pawn);
                scores[i] = piece_value_see(victim) * 10 - piece_value_see(attacker);
            }
        }

        let mut legal_moves = 0usize;
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

            // In QS, only search tactical moves, UNLESS in check
            if !in_check && !m.is_capture() && !m.is_promotion() {
                continue;
            }

            // SEE pruning: skip captures that lose material
            if !in_check && m.is_capture() && !see_ge(board, m, 0) {
                continue;
            }

            // Delta pruning per-move: skip captures that can't possibly raise alpha
            if !in_check && m.is_capture() {
                let victim = board
                    .piece_type_on(m.to(), board.side_to_move.opposite())
                    .unwrap_or(PieceType::Pawn);
                let gain = piece_value_see(victim) + 200;
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

    // ---------------------------------------------------------------------------
    // Bench
    // ---------------------------------------------------------------------------
    /// Run a benchmark search (depth 12) and return (nodes_searched, elapsed_ms).
    pub fn bench(&mut self, board: &mut Board) -> (u64, u64) {
        let start = std::time::Instant::now();
        self.nodes = 0;
        self.iterative_deepening(board, 12, std::time::Duration::from_secs(60));
        let elapsed = start.elapsed().as_millis() as u64;
        (self.nodes, elapsed)
    }
}
