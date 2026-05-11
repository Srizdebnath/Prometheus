use std::time::{Instant, Duration};
use crate::board::{Board, Move};
use crate::movegen::{MoveList, generate_moves};
use crate::evaluation::evaluate;
use crate::transposition::{TranspositionTable, TTEntry, NodeType};

pub const INFINITY: i32 = 30000;
pub const MATE_SCORE: i32 = 29000;

pub struct Search {
    pub nodes: u64,
    pub tt: TranspositionTable,
    pub start_time: Instant,
    pub time_limit: Duration,
    pub abort_search: bool,
}

impl Search {
    pub fn new() -> Self {
        Search { 
            nodes: 0,
            tt: TranspositionTable::new(16), // 16 MB TT
            start_time: Instant::now(),
            time_limit: Duration::from_secs(u64::MAX),
            abort_search: false,
        }
    }

    fn check_time(&mut self) {
        if self.nodes & 2047 == 0 {
            if self.start_time.elapsed() >= self.time_limit {
                self.abort_search = true;
            }
        }
    }

    pub fn search(&mut self, board: &mut Board, depth: u8) -> (i32, Option<Move>) {
        self.nodes = 0;
        self.start_time = Instant::now();
        self.time_limit = Duration::from_secs(u64::MAX);
        self.abort_search = false;
        
        let mut best_move = None;
        let score = self.alpha_beta(board, depth, 0, -INFINITY, INFINITY, &mut best_move);
        (score, best_move)
    }

    pub fn iterative_deepening(&mut self, board: &mut Board, max_depth: u8, time_limit: Duration) -> (i32, Option<Move>) {
        self.nodes = 0;
        self.start_time = Instant::now();
        self.time_limit = time_limit;
        self.abort_search = false;
        
        let mut best_move = None;
        let mut best_score = 0;
        
        for d in 1..=max_depth {
            let mut current_best = None;
            let score = self.alpha_beta(board, d, 0, -INFINITY, INFINITY, &mut current_best);
            
            if self.abort_search && d > 1 {
                break;
            }

            best_score = score;
            if current_best.is_some() {
                best_move = current_best;
            }
            
            // if mate found, we can break early
            if score >= MATE_SCORE - 1000 || score <= -MATE_SCORE + 1000 {
                break;
            }
        }
        
        (best_score, best_move)
    }

    fn score_move(board: &Board, m: Move) -> i32 {
        let mut score = 0;
        
        if m.is_capture() {
            let attacker = board.piece_type_on(m.from(), board.side_to_move).unwrap_or(crate::board::PieceType::Pawn);
            let victim = board.piece_type_on(m.to(), board.side_to_move.opposite()).unwrap_or(crate::board::PieceType::Pawn);
            
            let attacker_val = match attacker {
                crate::board::PieceType::Pawn => 10,
                crate::board::PieceType::Knight => 30,
                crate::board::PieceType::Bishop => 32,
                crate::board::PieceType::Rook => 50,
                crate::board::PieceType::Queen => 90,
                crate::board::PieceType::King => 900,
            };
            
            let victim_val = match victim {
                crate::board::PieceType::Pawn => 100,
                crate::board::PieceType::Knight => 300,
                crate::board::PieceType::Bishop => 320,
                crate::board::PieceType::Rook => 500,
                crate::board::PieceType::Queen => 900,
                crate::board::PieceType::King => 10000,
            };
            
            score += 10 * victim_val - attacker_val;
        }
        
        if m.is_promotion() {
            score += 800;
        }
        
        score
    }

    fn alpha_beta(
        &mut self, 
        board: &mut Board, 
        depth: u8, 
        ply: u8, 
        mut alpha: i32, 
        beta: i32, 
        best_move_out: &mut Option<Move>
    ) -> i32 {
        self.check_time();
        if self.abort_search {
            return 0;
        }

        let alpha_orig = alpha;
        let in_check = board.is_in_check(board.side_to_move);

        let mut tt_move = None;
        if let Some(entry) = self.tt.probe(board.zobrist_key) {
            if entry.depth >= depth {
                match entry.node_type() {
                    NodeType::Exact => {
                        if ply == 0 { *best_move_out = Some(entry.best_move); }
                        return entry.score as i32;
                    },
                    NodeType::LowerBound => {
                        if entry.score as i32 > alpha { alpha = entry.score as i32; }
                    },
                    NodeType::UpperBound => {
                        let mut b = beta;
                        if (entry.score as i32) < b { b = entry.score as i32; }
                        if alpha >= b { return entry.score as i32; }
                    }
                }
                if alpha >= beta {
                    if ply == 0 { *best_move_out = Some(entry.best_move); }
                    return entry.score as i32;
                }
            }
            tt_move = Some(entry.best_move);
        }

        if depth == 0 {
            return self.quiescence(board, alpha, beta, ply);
        }

        self.nodes += 1;

        let mut move_list = MoveList::new();
        generate_moves(board, &mut move_list);

        let mut scores = [0; crate::movegen::MAX_MOVES];
        for i in 0..move_list.len() {
            if Some(move_list[i]) == tt_move {
                scores[i] = 1000000; // Best move from TT
            } else {
                scores[i] = Self::score_move(board, move_list[i]);
            }
        }

        let mut legal_moves = 0;
        let mut best_score = -INFINITY;
        let mut best_move = None;

        for i in 0..move_list.len() {
            // Selection sort: find the best move among the remaining ones
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
                let is_tactical = m.is_capture() || m.is_promotion() || gives_check;

                let score = if legal_moves == 1 {
                    let mut dummy = None;
                    -self.alpha_beta(board, depth - 1, ply + 1, -beta, -alpha, &mut dummy)
                } else {
                    let mut s = 0;
                    let mut do_full_search = true;

                    // Late Move Reduction (LMR)
                    if depth >= 3 && legal_moves >= 4 && !is_tactical && !in_check {
                        let mut dummy = None;
                        s = -self.alpha_beta(board, depth - 2, ply + 1, -alpha - 1, -alpha, &mut dummy);
                        if s <= alpha {
                            do_full_search = false;
                        }
                    }

                    if do_full_search {
                        let mut dummy = None;
                        s = -self.alpha_beta(board, depth - 1, ply + 1, -alpha - 1, -alpha, &mut dummy);
                        if s > alpha && s < beta {
                            s = -self.alpha_beta(board, depth - 1, ply + 1, -beta, -alpha, &mut dummy);
                        }
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
                            break; // Beta cut-off
                        }
                    }
                }
            }
        }

        if legal_moves == 0 {
            if board.is_in_check(board.side_to_move) {
                return -MATE_SCORE + ply as i32; // Checkmate, prefer shorter mates
            } else {
                return 0; // Stalemate
            }
        }

        if ply == 0 {
            *best_move_out = best_move;
        }

        let node_type = if best_score <= alpha_orig {
            NodeType::UpperBound
        } else if best_score >= beta {
            NodeType::LowerBound
        } else {
            NodeType::Exact
        };

        if let Some(m) = best_move {
            self.tt.store(TTEntry::new(board.zobrist_key, m, best_score as i16, depth, node_type, 0));
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

        let in_check = board.is_in_check(board.side_to_move);
        
        let stand_pat = if in_check { -crate::search::INFINITY } else { evaluate(board) };
        
        if !in_check {
            if stand_pat >= beta {
                return beta;
            }
            if alpha < stand_pat {
                alpha = stand_pat;
            }
        }

        let mut move_list = MoveList::new();
        generate_moves(board, &mut move_list);

        let mut scores = [0; crate::movegen::MAX_MOVES];
        for i in 0..move_list.len() {
            scores[i] = Self::score_move(board, move_list[i]);
        }

        let mut legal_moves = 0;

        for i in 0..move_list.len() {
            // Selection sort for captures
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

            // In QS, we ONLY look at tactical moves, UNLESS we are in check!
            if !in_check && !m.is_capture() && !m.is_promotion() {
                continue;
            }

            if let Some(undo) = board.make_move(m) {
                legal_moves += 1;
                let score = -self.quiescence(board, -beta, -alpha, ply + 1);
                board.unmake_move(m, undo);

                if score >= beta {
                    return beta;
                }
                if score > alpha {
                    alpha = score;
                }
            }
        }

        if in_check && legal_moves == 0 {
            return -crate::search::MATE_SCORE + ply as i32;
        }

        alpha
    }
}
