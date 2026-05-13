#!/usr/bin/env python3
"""
Parse master PGN files → generate a compact Rust opening book.
Uses python-chess for reliable PGN parsing and UCI move conversion.
Output: a Rust hashmap keyed by the UCI move sequence prefix.
"""

import chess
import chess.pgn
import io
import os
import sys
from collections import defaultdict

BOOK_DEPTH = 12  # Max ply depth for book entries
MIN_ELO = 2400   # Minimum ELO to include a game
MIN_COUNT = 4    # Minimum times a move must appear to be included
MAX_POSITIONS = 5000  # Max book entries

def process_pgn_file(filepath, book, game_count_ref):
    """Process a single PGN file and add to the book."""
    with open(filepath, 'r', errors='replace') as f:
        content = f.read()
    
    # Fix common PGN issues (Windows line endings)
    content = content.replace('\r\n', '\n').replace('\r', '\n')
    pgn_io = io.StringIO(content)
    
    while True:
        try:
            game = chess.pgn.read_game(pgn_io)
        except Exception:
            continue
        if game is None:
            break
        
        # Filter by ELO
        try:
            welo = int(game.headers.get("WhiteElo", "0"))
        except ValueError:
            welo = 0
        try:
            belo = int(game.headers.get("BlackElo", "0"))
        except ValueError:
            belo = 0
        
        if welo < MIN_ELO and belo < MIN_ELO:
            continue
        
        result = game.headers.get("Result", "*")
        if result == "*":
            continue
        
        # Weight by result
        w_weight = 3 if result == "1-0" else (1 if result == "1/2-1/2" else 0)
        b_weight = 3 if result == "0-1" else (1 if result == "1/2-1/2" else 0)
        
        # ELO bonus
        elo_bonus = max(0, (max(welo, belo) - 2500) // 100)
        
        game_count_ref[0] += 1
        
        # Walk the main line
        board = game.board()
        uci_moves = []
        
        for i, move in enumerate(game.mainline_moves()):
            if i >= BOOK_DEPTH:
                break
            
            # Record this position's best move
            position_key = " ".join(uci_moves)
            uci_str = move.uci()
            
            is_white = (i % 2 == 0)
            weight = (w_weight if is_white else b_weight) + elo_bonus
            
            book[position_key][uci_str] += weight
            
            uci_moves.append(uci_str)
            board.push(move)

def generate_rust_book(book):
    """Generate the Rust opening book source file."""
    
    # Filter and sort
    filtered = {}
    for position_key, moves in book.items():
        good_moves = {m: c for m, c in moves.items() if c >= MIN_COUNT}
        if good_moves:
            top = sorted(good_moves.items(), key=lambda x: -x[1])[:4]
            total = sum(c for _, c in top)
            filtered[position_key] = [(m, c, total) for m, c in top]
    
    # Sort by key length (shallow positions first), limit size
    sorted_pos = sorted(filtered.items(), key=lambda x: len(x[0]))
    if len(sorted_pos) > MAX_POSITIONS:
        sorted_pos = sorted_pos[:MAX_POSITIONS]
    
    print(f"Final book: {len(sorted_pos)} positions", file=sys.stderr)
    
    lines = []
    lines.append("// Auto-generated opening book from master games")
    lines.append("// DO NOT EDIT — regenerate with: python3 tools/gen_book.py")
    lines.append("//")
    lines.append(f"// Positions: {len(sorted_pos)}")
    lines.append("")
    lines.append("use std::collections::HashMap;")
    lines.append("")
    lines.append("/// (uci_move, weight)")
    lines.append("pub type BookMoves = &'static [(&'static str, u32)];")
    lines.append("")
    lines.append("pub fn build_opening_book() -> HashMap<&'static str, BookMoves> {")
    lines.append(f"    let mut m: HashMap<&'static str, BookMoves> = HashMap::with_capacity({len(sorted_pos)});")
    
    for pos_key, top_moves in sorted_pos:
        entries = ", ".join([f'("{m}", {c})' for m, c, _ in top_moves])
        lines.append(f'    m.insert("{pos_key}", &[{entries}]);')
    
    lines.append("    m")
    lines.append("}")
    lines.append("")
    lines.append("/// Probe the opening book given the UCI move history.")
    lines.append("/// Returns the best UCI move string, or None.")
    lines.append("pub fn probe_book(book: &HashMap<&str, BookMoves>, uci_history: &[String]) -> Option<String> {")
    lines.append("    let key = uci_history.join(\" \");")
    lines.append('    if let Some(entries) = book.get(key.as_str()) {')
    lines.append("        // Weighted random for variety (using simple hash-based selection)")
    lines.append("        if entries.is_empty() { return None; }")
    lines.append("        let total: u32 = entries.iter().map(|(_, w)| w).sum();")
    lines.append("        if total == 0 { return None; }")
    lines.append("        // Use a simple pseudo-random based on total and key length")
    lines.append("        let seed = (key.len() as u32).wrapping_mul(2654435761);")
    lines.append("        let pick = seed % total;")
    lines.append("        let mut cumulative = 0u32;")
    lines.append("        for (m, w) in entries.iter() {")
    lines.append("            cumulative += w;")
    lines.append("            if pick < cumulative {")
    lines.append("                return Some(m.to_string());")
    lines.append("            }")
    lines.append("        }")
    lines.append("        Some(entries[0].0.to_string())")
    lines.append("    } else {")
    lines.append("        None")
    lines.append("    }")
    lines.append("}")
    
    return "\n".join(lines)

if __name__ == "__main__":
    masters_dir = sys.argv[1] if len(sys.argv) > 1 else "/home/ansh/prometheus/games/masters"
    
    book = defaultdict(lambda: defaultdict(int))
    game_count = [0]
    
    for filename in sorted(os.listdir(masters_dir)):
        if not filename.endswith('.pgn'):
            continue
        filepath = os.path.join(masters_dir, filename)
        print(f"Processing {filename}...", file=sys.stderr)
        process_pgn_file(filepath, book, game_count)
    
    print(f"\nTotal games processed: {game_count[0]}", file=sys.stderr)
    print(f"Raw positions: {len(book)}", file=sys.stderr)
    
    rust_code = generate_rust_book(book)
    print(rust_code)
