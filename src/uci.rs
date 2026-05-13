use std::io::{self, BufRead};
use crate::board::{Board, Move, Square, PieceType, Color};
use crate::search::Search;
use std::time::Duration;

pub fn uci_loop() {
    let mut board = Board::default();
    let mut search = Search::new();
    let stdin = io::stdin();
    
    for line in stdin.lock().lines() {
        let line = line.unwrap();
        let tokens: Vec<&str> = line.split_whitespace().collect();
        if tokens.is_empty() { continue; }
        
        match tokens[0] {
            "uci" => {
                println!("id name Prometheus 2.0");
                println!("id author Srizdebnath");
                println!("option name Hash type spin default 64 min 1 max 1024");
                println!("uciok");
            },
            "isready" => {
                println!("readyok");
            },
            "setoption" => {
                // setoption name Hash value 128
                if tokens.len() >= 5 && tokens[1] == "name" && tokens[2] == "Hash" && tokens[3] == "value" {
                    if let Ok(size) = tokens[4].parse::<usize>() {
                        search.tt = crate::transposition::TranspositionTable::new(size);
                    }
                }
            },
            "ucinewgame" => {
                board = Board::default();
                search.tt.clear();
            },
            "position" => {
                parse_position(&mut board, &tokens);
            },
            "go" => {
                let (max_depth, time_limit) = parse_go(&tokens, &board);
                
                let (_score, best_move) = search.iterative_deepening(&mut board, max_depth, time_limit);
                if let Some(m) = best_move {
                    println!("bestmove {}", move_to_uci(m));
                } else {
                    println!("bestmove 0000");
                }
            },
            "quit" => break,
            "d" | "display" => {
                // Debug: display the board
                display_board(&board);
            },
            _ => {}
        }
    }
}

fn parse_go(tokens: &[&str], board: &Board) -> (u8, Duration) {
    let mut depth: u8 = 64; // Max depth (essentially unlimited)
    let mut time_limit = Duration::from_secs(u64::MAX);
    let mut movetime: Option<u64> = None;
    let mut wtime: Option<u64> = None;
    let mut btime: Option<u64> = None;
    let mut winc: u64 = 0;
    let mut binc: u64 = 0;
    let mut movestogo: Option<u64> = None;
    let mut infinite = false;
    
    let mut i = 1;
    while i < tokens.len() {
        match tokens[i] {
            "depth" => {
                if i + 1 < tokens.len() {
                    depth = tokens[i+1].parse().unwrap_or(64);
                    i += 1;
                }
            },
            "movetime" => {
                if i + 1 < tokens.len() {
                    movetime = Some(tokens[i+1].parse().unwrap_or(1000));
                    i += 1;
                }
            },
            "wtime" => {
                if i + 1 < tokens.len() {
                    wtime = Some(tokens[i+1].parse().unwrap_or(60000));
                    i += 1;
                }
            },
            "btime" => {
                if i + 1 < tokens.len() {
                    btime = Some(tokens[i+1].parse().unwrap_or(60000));
                    i += 1;
                }
            },
            "winc" => {
                if i + 1 < tokens.len() {
                    winc = tokens[i+1].parse().unwrap_or(0);
                    i += 1;
                }
            },
            "binc" => {
                if i + 1 < tokens.len() {
                    binc = tokens[i+1].parse().unwrap_or(0);
                    i += 1;
                }
            },
            "movestogo" => {
                if i + 1 < tokens.len() {
                    movestogo = Some(tokens[i+1].parse().unwrap_or(30));
                    i += 1;
                }
            },
            "infinite" => {
                infinite = true;
            },
            _ => {}
        }
        i += 1;
    }
    
    // Time management logic
    if infinite {
        return (depth, Duration::from_secs(u64::MAX));
    }
    
    if let Some(mt) = movetime {
        // Fixed time per move
        return (depth, Duration::from_millis(mt.saturating_sub(50))); // small buffer
    }
    
    // Calculate time from wtime/btime
    let (our_time, our_inc) = match board.side_to_move {
        Color::White => (wtime.unwrap_or(60000), winc),
        Color::Black => (btime.unwrap_or(60000), binc),
    };
    
    if our_time > 0 {
        let moves_left = movestogo.unwrap_or(30) as u64;
        
        // Base time: divide remaining time by expected moves left
        let base_time = our_time / moves_left.max(1);
        
        // Add a portion of increment
        let inc_bonus = our_inc * 3 / 4;
        
        // Target time: base + increment, but never more than 1/3 of remaining time
        let target = (base_time + inc_bonus).min(our_time / 3);
        
        // Safety margin: never use all our time
        let safe_time = target.saturating_sub(50).max(10);
        
        time_limit = Duration::from_millis(safe_time);
    }
    
    (depth, time_limit)
}

fn parse_position(board: &mut Board, tokens: &[&str]) {
    // position startpos moves e2e4 e7e5
    // position fen ... moves ...
    let mut i = 1;
    if i >= tokens.len() { return; }
    
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

fn display_board(board: &Board) {
    eprintln!("  +---+---+---+---+---+---+---+---+");
    for rank in (0..8).rev() {
        eprint!("{} |", rank + 1);
        for file in 0..8 {
            let sq = Square::new(rank * 8 + file);
            let mut piece_char = ' ';
            
            for (color, case_fn) in [(Color::White, char::to_ascii_uppercase as fn(&char) -> char), 
                                      (Color::Black, char::to_ascii_lowercase as fn(&char) -> char)] {
                if let Some(pt) = board.piece_type_on(sq, color) {
                    let base = match pt {
                        PieceType::Pawn => 'p',
                        PieceType::Knight => 'n',
                        PieceType::Bishop => 'b',
                        PieceType::Rook => 'r',
                        PieceType::Queen => 'q',
                        PieceType::King => 'k',
                    };
                    piece_char = case_fn(&base);
                }
            }
            
            eprint!(" {} |", piece_char);
        }
        eprintln!();
        eprintln!("  +---+---+---+---+---+---+---+---+");
    }
    eprintln!("    a   b   c   d   e   f   g   h");
    eprintln!();
    eprintln!("Side to move: {:?}", board.side_to_move);
    eprintln!("Zobrist key: {:016x}", board.zobrist_key);
}
