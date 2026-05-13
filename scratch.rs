fn main() {
    let mut board = prometheus::board::Board::default();
    let moves = ["d2d4", "g8f6", "c2c4", "g7g6", "b1c3", "f8g7", "g1f3", "e8g8", "e2e4", "d7d6", "h2h3", "e7e5", "d4d5", "f6h5"];
    for m in moves {
        // Can't use parse_move directly since it's private. Let's just run search and print its output.
    }
}
