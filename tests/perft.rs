use prometheus::board::Board;
use prometheus::movegen::{MoveList, generate_moves};

pub fn perft(board: &mut Board, depth: u8) -> u64 {
    if depth == 0 {
        return 1;
    }

    let mut nodes = 0;
    let mut move_list = MoveList::new();
    generate_moves(board, &mut move_list);

    for m in &move_list {
        if let Some(undo) = board.make_move(*m) {
            nodes += perft(board, depth - 1);
            board.unmake_move(*m, undo);
        }
    }

    nodes
}

#[test]
fn initial_position_perft() {
    let mut board = Board::default();
    
    // Depth 1: 20
    // Depth 2: 400
    // Depth 3: 8902
    // Depth 4: 197281
    // Depth 5: 4865609
    
    assert_eq!(perft(&mut board, 1), 20);
    assert_eq!(perft(&mut board, 2), 400);
    assert_eq!(perft(&mut board, 3), 8902);
}
