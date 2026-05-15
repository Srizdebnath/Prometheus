use crate::board::{Board, Color, Move, PieceType, Square};
use crate::search::Search;
use std::io::{self, BufRead, Write};
use std::time::Duration;

/// Flush stdout - critical for UCI engines piped to GUIs.
/// Without this, output can be buffered and the GUI never sees it.
#[inline]
fn flush_stdout() {
    io::stdout().flush().ok();
}

pub fn uci_loop() {
    let mut board = Board::default();
    let mut search = Search::new();
    let book = crate::openings::build_opening_book();
    let mut uci_history: Vec<String> = Vec::new();

    // Opening behaviour controls
    let mut always_mainline_mode = false;
    let mut mainline_plies: usize = 16;

    let stdin = io::stdin();

    for line in stdin.lock().lines() {
        let line = line.unwrap();
        let tokens: Vec<&str> = line.split_whitespace().collect();
        if tokens.is_empty() {
            continue;
        }

        match tokens[0] {
            "uci" => {
                println!("id name Prometheus v1.0");
                println!("id author Srizdebnath");
                println!("option name Hash type spin default 64 min 1 max 1024");
                println!("option name Contempt type spin default 30 min -100 max 100");
                println!("option name AlwaysMainline type check default false");
                println!("option name MainlinePlies type spin default 16 min 0 max 60");
                println!("option name WeakOpponentMode type check default false");
                println!("option name WeakContemptBonus type spin default 20 min 0 max 150");
                println!("option name WeakEndgameDrawBias type spin default 20 min 0 max 150");
                println!("option name SelfCheckFilter type check default false");
                println!("option name MultiPVFilterLines type spin default 3 min 1 max 8");
                println!("uciok");
                flush_stdout();
            }
            "isready" => {
                println!("readyok");
                flush_stdout();
            }
            "setoption" => {
                if tokens.len() >= 5 && tokens[1] == "name" {
                    // Supports both:
                    //   setoption name X value Y
                    //   setoption name X Y
                    let (name, val) = if tokens.len() >= 5 && tokens[3] == "value" {
                        (tokens[2], tokens[4])
                    } else if tokens.len() >= 4 {
                        (tokens[2], tokens[3])
                    } else {
                        ("", "")
                    };

                    match name {
                        "Hash" => {
                            if let Ok(size) = val.parse::<usize>() {
                                search.tt = crate::transposition::TranspositionTable::new(size);
                            }
                        }
                        "Contempt" => {
                            if let Ok(v) = val.parse::<i32>() {
                                search.contempt = v;
                            }
                        }
                        "AlwaysMainline" => {
                            if let Some(v) = parse_bool_like(val) {
                                always_mainline_mode = v;
                            }
                        }
                        "MainlinePlies" => {
                            if let Ok(v) = val.parse::<usize>() {
                                mainline_plies = v.min(60);
                            }
                        }
                        "WeakOpponentMode" => {
                            if let Some(v) = parse_bool_like(val) {
                                search.weak_opponent_mode = v;
                            }
                        }
                        "WeakContemptBonus" => {
                            if let Ok(v) = val.parse::<i32>() {
                                search.weak_contempt_bonus = v.clamp(0, 150);
                            }
                        }
                        "WeakEndgameDrawBias" => {
                            if let Ok(v) = val.parse::<i32>() {
                                search.weak_endgame_draw_bias = v.clamp(0, 150);
                            }
                        }
                        "SelfCheckFilter" => {
                            if let Some(v) = parse_bool_like(val) {
                                search.self_check_filter = v;
                            }
                        }
                        "MultiPVFilterLines" => {
                            if let Ok(v) = val.parse::<u8>() {
                                search.multipv_lines = v.clamp(1, 8);
                            }
                        }
                        _ => {}
                    }
                }
            }
            "ucinewgame" => {
                board = Board::default();
                search.tt.clear();
                search.position_history.clear();
                uci_history.clear();
            }
            "position" => {
                parse_position(&mut board, &tokens, &mut search.position_history);
                // Track UCI move history for the opening book
                uci_history.clear();
                if tokens.len() >= 2 && tokens[1] == "startpos" {
                    if tokens.len() > 3 && tokens[2] == "moves" {
                        for i in 3..tokens.len() {
                            uci_history.push(tokens[i].to_string());
                        }
                    }
                }
            }
            "go" => {
                // Opening book policy
                let book_depth_limit = if always_mainline_mode {
                    mainline_plies
                } else {
                    20
                };

                // Emit opening name (deepest prefix match) for GUI
                let opening_key = uci_history.join(" ");
                if let Some(name) = crate::opening_names::get_opening_name(opening_key.as_str()) {
                    if !name.is_empty() {
                        println!("info string opening {}", name);
                        flush_stdout();
                    }
                }

                // If we're still in the opening, use the book
                if uci_history.len() < book_depth_limit {
                    if let Some(book_move) = crate::openings::probe_book(&book, &uci_history) {
                        // Validate that the book move is fully legal in the current position
                        if let Some(m) = parse_move(&board, &book_move) {
                            let mut test_board = board.clone();
                            if test_board.make_move(m).is_some() {
                                println!("bestmove {}", book_move);
                                flush_stdout();
                                continue;
                            }
                        }
                        // Book move is illegal for this position — fall through to search
                    }
                }

                let (max_depth, time_limit) = parse_go(&tokens, &board);

                let (_score, best_move) =
                    search.iterative_deepening(&mut board, max_depth, time_limit);
                if let Some(m) = best_move {
                    println!("bestmove {}", move_to_uci(m));
                } else {
                    println!("bestmove 0000");
                }
                flush_stdout();
            }
            "bench" => {
                let bench_positions = [
                    "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
                    "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
                    "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
                    "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
                    "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
                ];
                let mut total_nodes = 0u64;
                let start = std::time::Instant::now();
                for fen in &bench_positions {
                    if let Some(mut b) = crate::board::Board::from_fen(fen) {
                        search.nodes = 0;
                        search.iterative_deepening(&mut b, 12, std::time::Duration::from_secs(60));
                        total_nodes += search.nodes;
                    }
                }
                let elapsed = start.elapsed().as_millis();
                let nps = if elapsed > 0 {
                    total_nodes as u128 * 1000 / elapsed
                } else {
                    0
                };
                println!("{} nodes {} nps", total_nodes, nps);
                flush_stdout();
            }
            "quit" => break,
            "d" | "display" => {
                // Debug: display the board
                display_board(&board);
            }
            _ => {}
        }
    }
}

fn parse_bool_like(s: &str) -> Option<bool> {
    match s.to_ascii_lowercase().as_str() {
        "true" | "1" | "on" | "yes" => Some(true),
        "false" | "0" | "off" | "no" => Some(false),
        _ => None,
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
                    depth = tokens[i + 1].parse().unwrap_or(64);
                    i += 1;
                }
            }
            "movetime" => {
                if i + 1 < tokens.len() {
                    movetime = Some(tokens[i + 1].parse().unwrap_or(1000));
                    i += 1;
                }
            }
            "wtime" => {
                if i + 1 < tokens.len() {
                    wtime = Some(tokens[i + 1].parse().unwrap_or(60000));
                    i += 1;
                }
            }
            "btime" => {
                if i + 1 < tokens.len() {
                    btime = Some(tokens[i + 1].parse().unwrap_or(60000));
                    i += 1;
                }
            }
            "winc" => {
                if i + 1 < tokens.len() {
                    winc = tokens[i + 1].parse().unwrap_or(0);
                    i += 1;
                }
            }
            "binc" => {
                if i + 1 < tokens.len() {
                    binc = tokens[i + 1].parse().unwrap_or(0);
                    i += 1;
                }
            }
            "movestogo" => {
                if i + 1 < tokens.len() {
                    movestogo = Some(tokens[i + 1].parse().unwrap_or(30));
                    i += 1;
                }
            }
            "infinite" => {
                infinite = true;
            }
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
        let moves_left = movestogo.unwrap_or(25) as u64; // Default 25 instead of 30

        // Base time allocation
        let base_time = our_time / moves_left.max(1);

        // Add increment bonus (use most of it)
        let inc_bonus = our_inc * 8 / 10;

        // Soft limit: what we target
        let soft_limit = (base_time + inc_bonus).min(our_time / 4);

        // Hard limit: absolute max (never use more than 1/3 of remaining time for one move)
        // The search will use soft_limit as time_limit; the engine may use up to 2x soft for the last iteration
        let safe_time = soft_limit.saturating_sub(50).max(10);

        time_limit = Duration::from_millis(safe_time);
    }

    (depth, time_limit)
}

fn parse_position(board: &mut Board, tokens: &[&str], history: &mut Vec<u64>) {
    // position startpos moves e2e4 e7e5
    // position fen ... moves ...
    let mut i = 1;
    if i >= tokens.len() {
        return;
    }

    history.clear();

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
            *board = Board::default();
        }
    }

    // Record starting position
    history.push(board.zobrist_key);

    if i < tokens.len() && tokens[i] == "moves" {
        i += 1;
        while i < tokens.len() {
            let uci_move = tokens[i];
            if let Some(m) = parse_move(board, uci_move) {
                board.make_move(m);
                history.push(board.zobrist_key);
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

            for (color, case_fn) in [
                (Color::White, char::to_ascii_uppercase as fn(&char) -> char),
                (Color::Black, char::to_ascii_lowercase as fn(&char) -> char),
            ] {
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
