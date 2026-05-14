#!/usr/bin/env python3
"""
Generate NNUE training data from master PGN games.
For each position, extracts HalfKP features and gets Stockfish evaluation.
Outputs binary training data for the NNUE trainer.
"""

import chess
import chess.pgn
import chess.engine
import struct
import os
import sys
import glob
import time
import random
import asyncio
from pathlib import Path
from concurrent.futures import ProcessPoolExecutor

# HalfKP feature mapping
HIDDEN_SIZE = 256
HALFKP_FEATURES = 40960  # 64 king squares * 10 piece types * 64 piece squares

PIECE_INDEX = {
    chess.PAWN: 0,
    chess.KNIGHT: 1,
    chess.BISHOP: 2,
    chess.ROOK: 3,
    chess.QUEEN: 4,
}

def halfkp_indices(board):
    """Extract active HalfKP feature indices for a position."""
    white_features = []
    black_features = []
    
    wk = board.king(chess.WHITE)
    bk = board.king(chess.BLACK)
    
    if wk is None or bk is None:
        return white_features, black_features
    
    for sq in chess.SQUARES:
        piece = board.piece_at(sq)
        if piece is None or piece.piece_type == chess.KING:
            continue
        
        # White perspective
        if piece.color == chess.WHITE:
            p_idx = PIECE_INDEX[piece.piece_type]
        else:
            p_idx = PIECE_INDEX[piece.piece_type] + 5
        
        f_w = (wk * 10 + p_idx) * 64 + sq
        white_features.append(f_w)
        
        # Black perspective (mirrored)
        mirrored_sq = sq ^ 56
        mirrored_bk = bk ^ 56
        if piece.color == chess.WHITE:
            p_idx_b = PIECE_INDEX[piece.piece_type] + 5  # opponent's piece
        else:
            p_idx_b = PIECE_INDEX[piece.piece_type]
        
        f_b = (mirrored_bk * 10 + p_idx_b) * 64 + mirrored_sq
        black_features.append(f_b)
    
    return white_features, black_features


def result_to_float(result, side_to_move):
    """Convert game result to a float from the side-to-move's perspective."""
    if result == "1-0":
        return 1.0 if side_to_move == chess.WHITE else 0.0
    elif result == "0-1":
        return 0.0 if side_to_move == chess.WHITE else 1.0
    else:
        return 0.5


def process_pgn_file(pgn_path, stockfish_path, output_path, depth=10, max_games=None):
    """Process a single PGN file and generate training data."""
    positions = []
    game_count = 0
    pos_count = 0
    skipped = 0
    
    print(f"Processing {os.path.basename(pgn_path)}...", flush=True)
    
    try:
        engine = chess.engine.SimpleEngine.popen_uci(stockfish_path)
        engine.configure({"Threads": 1, "Hash": 64})
    except Exception as e:
        print(f"  Error starting Stockfish: {e}")
        return 0
    
    try:
        with open(pgn_path) as pgn_file:
            while True:
                game = chess.pgn.read_game(pgn_file)
                if game is None:
                    break
                    
                if max_games and game_count >= max_games:
                    break
                    
                game_count += 1
                result_str = game.headers.get("Result", "*")
                if result_str == "*":
                    continue
                
                board = game.board()
                move_num = 0
                
                for move in game.mainline_moves():
                    board.push(move)
                    move_num += 1
                    
                    # Skip first 8 moves (opening theory)
                    if move_num < 8:
                        continue
                    
                    # Skip positions in check (noisy)
                    if board.is_check():
                        continue
                    
                    # Random sampling: keep ~40% of positions to diversify
                    if random.random() > 0.4:
                        continue
                    
                    # Skip positions with very few pieces (< 4 pieces per side)
                    if len(board.piece_map()) < 6:
                        continue
                    
                    try:
                        info = engine.analyse(board, chess.engine.Limit(depth=depth))
                        score = info["score"].white()
                        
                        if score.is_mate():
                            # Convert mate to large centipawn value
                            mate_in = score.mate()
                            if mate_in > 0:
                                cp = 10000 - mate_in * 100
                            else:
                                cp = -10000 - mate_in * 100
                        else:
                            cp = score.score()
                        
                        # Skip extreme evaluations (likely won/lost - less training value)
                        if abs(cp) > 3000:
                            skipped += 1
                            continue
                        
                        # Get features
                        wf, bf = halfkp_indices(board)
                        
                        # Game result from white's perspective
                        result_val = result_to_float(result_str, chess.WHITE)
                        
                        # Side to move
                        stm = 1 if board.turn == chess.WHITE else 0
                        
                        positions.append((wf, bf, cp, result_val, stm))
                        pos_count += 1
                        
                    except Exception as e:
                        skipped += 1
                        continue
                
                if game_count % 100 == 0:
                    print(f"  {os.path.basename(pgn_path)}: {game_count} games, {pos_count} positions", flush=True)
                    
    except Exception as e:
        print(f"  Error processing {pgn_path}: {e}")
    finally:
        engine.quit()
    
    # Write binary data
    if positions:
        write_training_data(positions, output_path)
    
    print(f"  {os.path.basename(pgn_path)}: Done. {game_count} games -> {pos_count} positions (skipped {skipped})", flush=True)
    return pos_count


def write_training_data(positions, output_path):
    """
    Write training data in binary format.
    Format per entry:
      - num_white_features (u16)
      - white_feature_indices (u16 each)
      - num_black_features (u16)
      - black_feature_indices (u16 each)
      - score (i16, centipawns from white's perspective)
      - result (f32, game result from white's perspective)
      - stm (u8, 1=white, 0=black)
    """
    with open(output_path, 'ab') as f:  # append mode
        for wf, bf, score, result, stm in positions:
            # Clamp score to i16 range
            score = max(-32000, min(32000, score))
            
            # Write white features
            f.write(struct.pack('<H', len(wf)))
            for idx in wf:
                f.write(struct.pack('<H', idx))
            
            # Write black features
            f.write(struct.pack('<H', len(bf)))
            for idx in bf:
                f.write(struct.pack('<H', idx))
            
            # Write score, result, stm
            f.write(struct.pack('<h', score))
            f.write(struct.pack('<f', result))
            f.write(struct.pack('<B', stm))


def main():
    pgn_dir = "/home/ansh/prometheus/games/masters"
    stockfish_path = "/usr/games/stockfish"
    output_dir = "/home/ansh/prometheus/training_data"
    
    os.makedirs(output_dir, exist_ok=True)
    
    depth = 10  # Stockfish depth per position
    
    pgn_files = sorted(glob.glob(os.path.join(pgn_dir, "*.pgn")))
    
    if not pgn_files:
        print("No PGN files found!")
        return
    
    print(f"Found {len(pgn_files)} PGN files")
    print(f"Using Stockfish at depth {depth}")
    print(f"Output: {output_dir}")
    print()
    
    output_path = os.path.join(output_dir, "training.bin")
    
    # Clear existing data
    if os.path.exists(output_path):
        os.remove(output_path)
    
    total_positions = 0
    start_time = time.time()
    
    for pgn_file in pgn_files:
        count = process_pgn_file(pgn_file, stockfish_path, output_path, depth=depth)
        total_positions += count
    
    elapsed = time.time() - start_time
    print(f"\n=== Data Generation Complete ===")
    print(f"Total positions: {total_positions}")
    print(f"Time: {elapsed:.1f}s ({total_positions/max(elapsed,1):.0f} pos/s)")
    
    file_size = os.path.getsize(output_path) if os.path.exists(output_path) else 0
    print(f"File size: {file_size / 1024 / 1024:.1f} MB")


if __name__ == "__main__":
    main()
