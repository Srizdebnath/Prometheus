# Prometheus 🦅🔥

Prometheus is a high-performance, UCI-compatible classical chess engine written entirely from scratch in **Rust** 🦀. 

Built with a focus on zero-cost abstractions and blazing-fast tactical calculations, Prometheus combines modern search heuristics with a deeply tuned PeSTO-inspired Tapered Evaluation system. In its debut v1.0 state, it calculates at millions of nodes per second, effortlessly crushing human masters (tested against ~2400 peak Elo), and serves as a highly readable, modular foundation for advanced chess programming.

## Features ✨

### Core Architecture
* **Bitboard Representation**: Blazing fast board state representation using 64-bit integers.
* **Pre-computed Ray Attacks**: Compile-time (`const fn`) generated sliding piece attack tables for instant move generation.
* **Zero-Cost Zobrist Hashing**: Incremental XOR hashing for completely frictionless state updates and Transposition Table (TT) lookups.
* **UCI Protocol Support**: Plug-and-play compatibility with any modern Chess GUI (CuteChess, Arena, PyChess, En Croissant).

### Advanced Search Engine
* **Negamax Alpha-Beta Pruning**: The core backbone of the search tree.
* **Principal Variation Search (PVS)**: A null-window search optimization that drastically shrinks the search tree by assuming the first searched move is the best.
* **Iterative Deepening**: Progressive depth exploration paired with a strict time-management budget (`std::time::Instant`).
* **Quiescence Search (QS)**: Deep tactical resolution at the horizon depth, evaluating captures and checks to eliminate the "Horizon Effect."
* **Late Move Reductions (LMR)**: Aggressive search depth pruning for non-tactical "quiet" moves, enabling exponential depth in forcing tactical lines.
* **Transposition Table (TT)**: Depth-preferred caching of previously evaluated positions to prevent redundant calculations and ensure optimal move ordering.

### Evaluation
* **PeSTO Tapered Evaluation**: Dynamic interpolation between Midgame (MG) and Endgame (EG) game phases.
* **Phase-Aware Piece Square Tables (PSTs)**: Positional understanding that shifts dynamically (e.g., centralizing the King in the endgame, or pushing passed pawns).

## Installation & Usage 🚀

### Prerequisites
You will need the **Rust toolchain** installed on your machine.
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Compiling
To achieve maximum performance (Nodes Per Second), you **must** compile Prometheus in release mode.
```bash
git clone https://github.com/Srizdebnath/Prometheus.git
cd Prometheus
cargo build --release
```
The executable will be located at: `target/release/prometheus`

### Playing against Prometheus
Prometheus runs via the UCI protocol and is meant to be loaded into a Chess GUI. 
For Linux users, we recommend **PyChess** or **CuteChess**.
1. Install a GUI: `sudo apt install pychess`
2. Open the GUI and navigate to Engine Settings.
3. Add a new engine and provide the path to your compiled `/target/release/prometheus` binary.
4. Start a new game and prepare to fight!

## Contributing 🤝
Prometheus is completely open-source and welcomes pull requests! If you are a Rust developer, chess programmer, or AI enthusiast, we are looking for contributors to help push Prometheus into the 3000+ Elo stratosphere.

### High-Priority Features for v2.0:
1. **Magic Bitboards**: Replacing ray-attacks with perfect hashing for sliding pieces.
2. **Null Move Pruning (NMP)**: Aggressive beta-cutoffs for dominant positions.
3. **History Heuristic & Killer Moves**: Sibling-node move ordering optimizations.
4. **NNUE Integration**: Evolving the evaluation from classical PSTs to an Efficiently Updatable Neural Network (HalfKP architecture stub already exists in `src/nn.rs`!).

## License 📄
This project is open-sourced under the MIT License.
