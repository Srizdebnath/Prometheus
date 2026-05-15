// Runtime-loaded opening book — zero compile cost.
// The binary blob is embedded at compile time via include_bytes!,
// which is essentially instant compared to compiling 100K lines of Rust.
use std::collections::HashMap;
use std::io::{Cursor, Read};

/// Each entry: fixed 5-byte UCI move string + u32 weight.
pub type BookMoves = Vec<([u8; 5], u32)>;

pub fn build_opening_book() -> HashMap<String, BookMoves> {
    let data: &[u8] = include_bytes!("../Openings/book.bin");
    parse_book(data)
}

fn parse_book(data: &[u8]) -> HashMap<String, BookMoves> {
    let mut cur = Cursor::new(data);
    let mut map = HashMap::with_capacity(100_000);

    loop {
        // u16 LE: key length
        let mut len_buf = [0u8; 2];
        if cur.read_exact(&mut len_buf).is_err() {
            break;
        }
        let key_len = u16::from_le_bytes(len_buf) as usize;

        // key bytes (no null terminator)
        let mut key_buf = vec![0u8; key_len];
        if cur.read_exact(&mut key_buf).is_err() {
            break;
        }
        let key = match String::from_utf8(key_buf) {
            Ok(s) => s,
            Err(_) => break,
        };

        // u8: number of moves
        let mut count_buf = [0u8; 1];
        if cur.read_exact(&mut count_buf).is_err() {
            break;
        }
        let count = count_buf[0] as usize;

        // moves: each is a fixed 5-byte UCI string + u32 LE weight
        let mut moves = Vec::with_capacity(count);
        for _ in 0..count {
            let mut mv = [0u8; 5];
            if cur.read_exact(&mut mv).is_err() {
                break;
            }
            let mut wt_buf = [0u8; 4];
            if cur.read_exact(&mut wt_buf).is_err() {
                break;
            }
            moves.push((mv, u32::from_le_bytes(wt_buf)));
        }

        if !moves.is_empty() {
            map.insert(key, moves);
        }
    }

    map
}

/// Extract the UCI move string from the 5-byte fixed array.
pub fn mv_str(mv: &[u8; 5]) -> &str {
    let end = mv.iter().position(|&b| b == 0).unwrap_or(5);
    std::str::from_utf8(&mv[..end]).unwrap_or("")
}

/// Look up the current position by the sequence of moves played so far.
/// Always returns the highest-weight move — the most theoretically sound
/// GM choice. No randomness: we always want the objectively best line.
pub fn probe_book(book: &HashMap<String, BookMoves>, moves: &[String]) -> Option<String> {
    let key = moves.join(" ");
    let entries = book.get(&key)?;
    if entries.is_empty() {
        return None;
    }
    // Pick the move with the single highest weight (top GM choice).
    let best = entries.iter().max_by_key(|(_, w)| *w)?;
    Some(mv_str(&best.0).to_owned())
}
