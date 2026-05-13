use prometheus::uci::uci_loop;

fn main() {
    // Initialize magic bitboard tables (must be done before any attack lookups)
    prometheus::magics::init_magics();
    uci_loop();
}
