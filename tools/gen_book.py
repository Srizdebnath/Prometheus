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

BOOK_DEPTH = 25  # Max ply depth for book entries
MIN_ELO = 2400      # Minimum ELO to include a game
MIN_COUNT = 1    # Minimum times a move must appear to be included
MAX_POSITIONS = 100000  # Max book entries

def process_pgn_file(filepath, book, game_count_ref):
    """Process a single PGN file and add to the book."""
    print(f"Processing {os.path.basename(filepath)}...", file=sys.stderr)
    with open(filepath, 'r', errors='replace') as pgn_file:
        while True:
            try:
                game = chess.pgn.read_game(pgn_file)
            except Exception as e:
                print(f"Error reading game in {filepath}: {e}", file=sys.stderr)
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
            
            # Weight by result
            result = game.headers.get("Result", "*")
            w_weight = 4 if result == "1-0" else (2 if result in ["1/2-1/2", "*"] else 0)
            b_weight = 4 if result == "0-1" else (2 if result in ["1/2-1/2", "*"] else 0)
            
            # ELO bonus - give more weight to high ELO games
            avg_elo = (welo + belo) / 2
            elo_bonus = max(0, int((avg_elo - 2000) // 100))
            
            game_count_ref[0] += 1
            if game_count_ref[0] % 1000 == 0:
                print(f"  Processed {game_count_ref[0]} games...", file=sys.stderr)
            
            # Walk the main line
            uci_moves = []
            for i, move in enumerate(game.mainline_moves()):
                if i >= BOOK_DEPTH:
                    break
                
                # Record this position's best move
                position_key = " ".join(uci_moves)
                uci_str = move.uci()
                
                is_white = (i % 2 == 0)
                weight = (w_weight if is_white else b_weight) + elo_bonus
                
                if weight > 0:
                    book[position_key][uci_str] += weight
                
                uci_moves.append(uci_str)

def generate_rust_book(book):
    """Generate the Rust opening book source file."""
    
    # Filter and sort
    filtered = {}
    for position_key, moves in book.items():
        good_moves = {m: c for m, c in moves.items() if c >= MIN_COUNT}
        if good_moves:
            # Take top 6 moves for variety
            top = sorted(good_moves.items(), key=lambda x: -x[1])[:6]
            filtered[position_key] = top
    
    # Sort by key length (shallow positions first), then by total weight
    sorted_pos = sorted(filtered.items(), key=lambda x: (len(x[0]), -sum(m[1] for m in x[1])))
    
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
        entries = ", ".join([f'("{m}", {c})' for m, c in top_moves])
        lines.append(f'    m.insert("{pos_key}", &[{entries}]);')
    
    lines.append("    m")
    lines.append("}")
    lines.append("")
    lines.append("/// Probe the opening book given the UCI move history.")
    lines.append("/// Returns the best UCI move string, or None.")
    lines.append("pub fn probe_book(book: &HashMap<&str, BookMoves>, uci_history: &[String]) -> Option<String> {")
    lines.append("    let key = uci_history.join(\" \");")
    lines.append('    if let Some(entries) = book.get(key.as_str()) {')
    lines.append("        if entries.is_empty() { return None; }")
    lines.append("        let total: u32 = entries.iter().map(|(_, w)| w).sum();")
    lines.append("        if total == 0 { return Some(entries[0].0.to_string()); }")
    lines.append("        ")
    lines.append("        // Selection logic: weighted towards higher weight moves but with some variety")
    lines.append("        // Use a deterministic seed for consistency in a given match position")
    lines.append("        let mut seed = 0u32;")
    lines.append("        for b in key.as_bytes() {")
    lines.append("            seed = seed.wrapping_mul(31).wrapping_add(*b as u32);")
    lines.append("        }")
    lines.append("        if seed == 0 { seed = 12345; }")
    lines.append("        ")
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
    sources = [
        "/home/ansh/prometheus/masters",
    ]
    
    book = defaultdict(lambda: defaultdict(int))
    game_count = [0]
    
    for source in sources:
        if not os.path.exists(source):
            print(f"Warning: Source {source} not found.", file=sys.stderr)
            continue
            
        if os.path.isdir(source):
            for filename in sorted(os.listdir(source)):
                if filename.endswith('.pgn'):
                    filepath = os.path.join(source, filename)
                    process_pgn_file(filepath, book, game_count)
        elif source.endswith('.pgn'):
            process_pgn_file(source, book, game_count)
    
    print(f"\nTotal games processed: {game_count[0]}", file=sys.stderr)
    print(f"Raw positions: {len(book)}", file=sys.stderr)
    
    rust_code = generate_rust_book(book)
    
    # Output to src/openings.rs directly instead of stdout if requested, 
    # but for now let's keep it flexible or just write it.
    output_path = "/home/ansh/prometheus/src/openings.rs"
    with open(output_path, "w") as f:
        f.write(rust_code)
    
    print(f"Book written to {output_path}", file=sys.stderr)

