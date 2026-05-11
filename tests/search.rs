use prometheus::board::{Board, Move, Square, MOVE_FLAG_QUIET, MOVE_FLAG_DOUBLE_PAWN_PUSH};
use prometheus::search::{Search, MATE_SCORE};

#[test]
fn test_fools_mate() {
    let mut board = Board::default();
    
    // 1. f3
    board.make_move(Move::new(Square::new(13), Square::new(21), MOVE_FLAG_QUIET));
    // 1... e5
    board.make_move(Move::new(Square::new(52), Square::new(36), MOVE_FLAG_DOUBLE_PAWN_PUSH));
    // 2. g4
    board.make_move(Move::new(Square::new(14), Square::new(30), MOVE_FLAG_DOUBLE_PAWN_PUSH));

    // Now it's Black's turn. Qh4# should be the best move.
    // Qh4 is from d8 (file 3, rank 7 -> 59) to h4 (file 7, rank 3 -> 31)
    let mut search = Search::new();
    let (score, best_move) = search.search(&mut board, 2);

    assert!(best_move.is_some());
    let m = best_move.unwrap();
    
    assert_eq!(m.from().0, 59); // d8
    assert_eq!(m.to().0, 31);   // h4
    
    // The score should be a mate score for Black, which means -MATE_SCORE + ply
    // Wait, Negamax returns positive score for the side to move if they are winning.
    // So if Black has a forced mate, Negamax will return a large positive score (near MATE_SCORE).
    assert!(score > MATE_SCORE - 100);
}
