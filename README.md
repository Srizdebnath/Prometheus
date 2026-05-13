# Prometheus Chess Engine 2.0

A high-performance chess engine written in Rust, designed for competitive play via the UCI protocol.

## Features

### Board Representation
- **Bitboard-based** position representation for efficient move generation
- Full Zobrist hashing for transposition table and repetition detection
- Complete FEN parsing and UCI protocol support

### Search & Draw Detection
- **Iterative Deepening** with aspiration windows
- **Alpha-Beta Pruning** with Principal Variation Search (PVS)
- **Null Move Pruning** (NMP) — skip our turn; if still winning, prune
- **Reverse Futility Pruning** (Static NMP) — static eval margin-based pruning
- **Late Move Reductions** (LMR) — logarithmic reduction table
- **Late Move Pruning** (LMP) — skip quiet moves at low depths  
- **Futility Pruning** — skip hopeless quiet moves near leaf nodes
- **Razoring** — drop into quiescence when far below alpha
- **Check Extensions** — extend search when in check
- **Quiescence Search** with delta pruning and SEE-based move pruning
- **Draw Detection** — 50-move rule and position history repetition detection

### Move Ordering
- TT move priority
- MVV-LVA capture ordering
- **Killer Move Heuristic** (2 per ply)
- **History Heuristic** with gravity (bonus/malus)
- **Countermove Heuristic**

### Evaluation
- **Tapered Evaluation** (midgame ↔ endgame interpolation)
- **PeSTO Piece-Square Tables** for all pieces
- **Pawn Structure**: doubled, isolated, and passed pawn detection
- **Bishop Pair** bonus
- **Rook on Open/Semi-Open Files**
- **Piece Mobility** (knight, bishop, rook, queen)
- **King Safety**: comprehensive evaluation (pawn shield, open files near king, enemy attacker counting with quadratic danger scaling)
- **Endgame Knowledge**: connected passed pawns, rook behind passed pawn bonuses
- Tempo bonus

### Opening Book
- Built-in polyglot-style opening book extracted from 1.6M+ master games (Carlsen, Nakamura, Anand, etc.)
- Automatically avoids early blunders and selects GM-approved opening lines.

### Transposition Table
- 64MB default (configurable via UCI `Hash` option)
- Generation-based aging for smarter replacement
- Depth-preferred with Exact node bias

### UCI Protocol
- Full `wtime/btime/winc/binc/movestogo` time management
- `depth`, `movetime`, `infinite` search modes
- Configurable hash size
- Info output with depth, seldepth, score, nodes, nps, hashfull

## Building

```bash
cargo build --release
```

## Running

```bash
./target/release/prometheus
```

Then use any UCI-compatible chess GUI (Arena, CuteChess, etc.) to play.

## Quick Test

```bash
echo -e "uci\nisready\nposition startpos\ngo depth 12\nquit" | ./target/release/prometheus
```

## Architecture

| Module | Purpose |
|--------|---------|
| `board.rs` | Bitboard representation, make/unmake moves |
| `movegen.rs` | Legal move generation |
| `attacks.rs` | Attack tables (pawns, knights, kings, sliding pieces) |
| `search.rs` | Alpha-beta search with all pruning techniques |
| `evaluation.rs` | Tapered PeSTO + structural evaluation |
| `transposition.rs` | Transposition table with aging |
| `zobrist.rs` | Zobrist key generation |
| `uci.rs` | UCI protocol handler |
| `nn.rs` | NNUE architecture (future) |

## Author

**Srizdebnath**
