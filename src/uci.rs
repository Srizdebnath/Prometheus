use std::io::{self, BufRead};
use crate::board::{Board, Move, Square, PieceType};
use crate::search::Search;
use std::time::Duration;

pub fn uci_loop() {
    let mut board = Board::default();
    let mut search = Search::new();
    let stdin = io::stdin();
    
    println!("Prometheus Chess Engine initialized.");
    
    for line in stdin.lock().lines() {
        let line = line.unwrap();
        let tokens: Vec<&str> = line.split_whitespace().collect();
        if tokens.is_empty() { continue; }
        
        match tokens[0] {
            "uci" => {
                println!("id name Prometheus");
                println!("id author Srizdebnath");
                println!("uciok");
            },
            "isready" => {
                println!("readyok");
            },
            "ucinewgame" => {
                board = Board::default();
                search.tt.clear();
            },
            "position" => {
                parse_position(&mut board, &tokens);
            },
            "go" => {
                // simple go command processing
                // usually: go wtime 10000 btime 10000 ... or go depth 6
                let mut depth = 6;
                let mut time_limit = Duration::from_secs(u64::MAX);
                
                let mut i = 1;
                while i < tokens.len() {
                    match tokens[i] {
                        "depth" => if i + 1 < tokens.len() { depth = tokens[i+1].parse().unwrap_or(6); },
                        "movetime" => if i + 1 < tokens.len() { time_limit = Duration::from_millis(tokens[i+1].parse().unwrap_or(1000)); },
                        _ => {}
                    }
                    i += 1;
                }
                
                let (_score, best_move) = search.iterative_deepening(&mut board, depth, time_limit);
                if let Some(m) = best_move {
                    println!("bestmove {}", move_to_uci(m));
                } else {
                    println!("bestmove 0000"); // should never happen
                }
            },
            "quit" => break,
            _ => {}
        }
    }
}

fn parse_position(board: &mut Board, tokens: &[&str]) {
    // position startpos moves e2e4 e7e5
    // position fen ... moves ...
    let mut i = 1;
    if tokens[i] == "startpos" {
        *board = Board::default();
        i += 1;
    } else if tokens[i] == "fen" {
        i += 1;
        let mut fen_parts = Vec::new();
        while i < tokens.len() && tokens[i] != "moves" {
            fen_parts.push(tokens[i]);
            i += 1;
        }
        let fen = fen_parts.join(" ");
        if let Some(b) = Board::from_fen(&fen) {
            *board = b;
        } else {
            // fallback if FEN is invalid
            *board = Board::default();
        }
    }
    
    if i < tokens.len() && tokens[i] == "moves" {
        i += 1;
        while i < tokens.len() {
            let uci_move = tokens[i];
            if let Some(m) = parse_move(board, uci_move) {
                board.make_move(m); // we assume UCI sends valid legal moves
            }
            i += 1;
        }
    }
}

fn move_to_uci(m: Move) -> String {
    let from = square_to_algebraic(m.from());
    let to = square_to_algebraic(m.to());
    let promo = match m.promotion_piece_type() {
        Some(PieceType::Queen) => "q",
        Some(PieceType::Rook) => "r",
        Some(PieceType::Bishop) => "b",
        Some(PieceType::Knight) => "n",
        _ => "",
    };
    format!("{}{}{}", from, to, promo)
}

fn parse_move(board: &Board, uci: &str) -> Option<Move> {
    // Very basic and slow legal move parsing matching the exact uci string
    let mut move_list = crate::movegen::MoveList::new();
    crate::movegen::generate_moves(board, &mut move_list);
    
    for i in 0..move_list.len() {
        let m = move_list[i];
        if move_to_uci(m) == uci {
            return Some(m);
        }
    }
    None
}

fn square_to_algebraic(sq: Square) -> String {
    let file = (sq.file() + b'a') as char;
    let rank = (sq.rank() + b'1') as char;
    format!("{}{}", file, rank)
}
