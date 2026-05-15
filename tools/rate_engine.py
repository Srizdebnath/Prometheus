import chess
import chess.engine
import asyncio
import os
import math

async def run_match(prometheus_path, stockfish_path, stockfish_elo, num_games, time_limit):
    results = {"1-0": 0, "0-1": 0, "1/2-1/2": 0}
    prometheus_score = 0
    
    print(f"Running {num_games} games against Stockfish at {stockfish_elo} Elo...")
    
    for i in range(num_games):
        _, prometheus_engine = await chess.engine.popen_uci(prometheus_path)
        _, stockfish_engine = await chess.engine.popen_uci(stockfish_path)
        
        # Stockfish 16 UCI_Elo max is 3190
        clamped_elo = min(stockfish_elo, 3190)
        await stockfish_engine.configure({"UCI_LimitStrength": True, "UCI_Elo": clamped_elo})
        
        board = chess.Board()
        
        # Alternate sides
        if i % 2 == 0:
            white, black = prometheus_engine, stockfish_engine
            prometheus_side = chess.WHITE
        else:
            white, black = stockfish_engine, prometheus_engine
            prometheus_side = chess.BLACK
            
        try:
            while not board.is_game_over():
                if board.turn == chess.WHITE:
                    result = await white.play(board, chess.engine.Limit(time=time_limit))
                else:
                    result = await black.play(board, chess.engine.Limit(time=time_limit))
                board.push(result.move)
        except Exception as e:
            print(f"Error in game {i+1}: {e}")
            continue
        finally:
            await prometheus_engine.quit()
            await stockfish_engine.quit()
            
        res = board.result()
        results[res] += 1
        
        if res == "1-0":
            prometheus_score += 1 if prometheus_side == chess.WHITE else 0
        elif res == "0-1":
            prometheus_score += 1 if prometheus_side == chess.BLACK else 0
        else:
            prometheus_score += 0.5
            
        print(f"  Game {i+1}: {res} (Prometheus score: {prometheus_score})")
        
    return prometheus_score

async def main():
    prometheus_path = "/home/ansh/prometheus/target/release/prometheus"
    stockfish_path = "/usr/games/stockfish"
    
    if not os.path.exists(prometheus_path):
        print("Prometheus binary not found. Please build it first.")
        return

    # Levels to test
    elo_levels = [1500, 1800, 2100, 2400, 2700, 3000, 3300]
    games_per_level = 6
    time_limit = 0.2 # 200ms per move
    
    estimates = []
    
    print("--- Prometheus Rating Benchmark ---")
    
    for elo in elo_levels:
        score = await run_match(prometheus_path, stockfish_path, elo, games_per_level, time_limit)
        percentage = score / games_per_level
        
        if percentage == 1:
            est = elo + 400
        elif percentage == 0:
            est = elo - 400
        else:
            est = elo + 400 * math.log10(percentage / (1 - percentage))
        
        estimates.append(est)
        print(f"Estimate at {elo} level: {est:.0f} Elo\n")
        
    final_elo = sum(estimates) / len(estimates)
    print(f"--- FINAL RESULTS ---")
    print(f"Estimated Prometheus Rating: {final_elo:.0f} Elo")
    print(f"Based on {len(elo_levels) * games_per_level} games against Stockfish 16.")

if __name__ == "__main__":
    asyncio.run(main())
