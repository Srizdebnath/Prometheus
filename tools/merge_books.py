#!/usr/bin/env python3
"""
merge_books.py — merge games.bin + openings.bin into a new book.bin.

Strategy:
  - openings.bin  = deep theoretical lines  → weights multiplied by 4
  - games.bin     = broad master-game stats  → weights as-is
  - Merged into one table, top MAX_MOVES moves kept per position
  - Positions sorted by total weight (strongest first), top MAX_POSITIONS kept
  - Output written to Openings/book.bin in the same compact binary format

Binary format (same as before, openings.rs needs NO changes):
  per entry:
    [u16 LE]  key length
    [N bytes] position key  (space-joined UCI moves, e.g. "e2e4 e7e5")
    [u8]      number of moves (M)
    M × { [5 bytes] null-padded UCI move string
          [u32 LE] weight }
"""

import struct
import os
import sys
from collections import defaultdict

# ── tunables ────────────────────────────────────────────────────────────────
MAX_POSITIONS  = 150_000   # how many positions to keep in the final book
MAX_MOVES      = 8         # top moves per position (was 6, more = stronger)
THEORY_BOOST   = 4         # openings.bin weight multiplier
MIN_WEIGHT     = 2         # drop moves below this weight
MAX_DEPTH_PLY  = 40        # ignore positions deeper than this (space-separated moves)
# ────────────────────────────────────────────────────────────────────────────

SCRIPT_DIR  = os.path.dirname(os.path.abspath(__file__))
PROJECT_DIR = os.path.dirname(SCRIPT_DIR)
OPENINGS_DIR = os.path.join(PROJECT_DIR, "Openings")

INPUT_FILES = [
    (os.path.join(OPENINGS_DIR, "openings.bin"), THEORY_BOOST),
    (os.path.join(OPENINGS_DIR, "games.bin"),    1),
]
OUTPUT_FILE = os.path.join(OPENINGS_DIR, "book.bin")


def parse_bin(path):
    """Parse a binary book file. Returns dict: key -> {move: weight}."""
    book = defaultdict(lambda: defaultdict(int))
    with open(path, "rb") as f:
        data = f.read()

    i = 0
    n = len(data)
    entries = 0

    while i + 3 <= n:
        # key length (u16 LE)
        key_len = struct.unpack_from("<H", data, i)[0]
        i += 2
        if i + key_len + 1 > n:
            break

        key = data[i:i + key_len].decode("utf-8", errors="replace")
        i += key_len

        # move count (u8)
        count = data[i]
        i += 1

        for _ in range(count):
            if i + 9 > n:           # 5 bytes move + 4 bytes weight
                break
            mv_bytes = data[i:i + 5]
            # strip null padding to get the UCI string
            null_pos = mv_bytes.find(b'\x00')
            mv = mv_bytes[:null_pos].decode("ascii") if null_pos >= 0 else mv_bytes.decode("ascii")
            i += 5
            weight = struct.unpack_from("<I", data, i)[0]
            i += 4

            if mv:
                book[key][mv] += weight

        entries += 1
        if entries % 50_000 == 0:
            print(f"  parsed {entries} entries…", file=sys.stderr)

    print(f"  → {entries} positions read from {os.path.basename(path)}", file=sys.stderr)
    return book


def write_bin(book, path):
    """Write merged book dict to the compact binary format."""
    out = bytearray()
    written = 0

    for key, moves in book:
        key_bytes = key.encode("utf-8")
        key_len   = len(key_bytes)

        # encode each move as exactly 5 null-padded bytes
        encoded_moves = []
        for mv, w in moves:
            mv_b = mv.encode("ascii")[:5]
            mv_b = mv_b + b'\x00' * (5 - len(mv_b))   # pad to 5
            encoded_moves.append((mv_b, w))

        out += struct.pack("<H", key_len)
        out += key_bytes
        out += struct.pack("B", len(encoded_moves))
        for mv_b, w in encoded_moves:
            out += mv_b
            out += struct.pack("<I", w)

        written += 1

    with open(path, "wb") as f:
        f.write(out)

    size_mb = len(out) / 1_048_576
    print(f"\nWrote {written} positions  ({size_mb:.1f} MB)  →  {path}", file=sys.stderr)


def main():
    # ── 1. load and merge ────────────────────────────────────────────────────
    merged = defaultdict(lambda: defaultdict(int))

    for filepath, boost in INPUT_FILES:
        if not os.path.exists(filepath):
            print(f"[WARN] not found, skipping: {filepath}", file=sys.stderr)
            continue
        print(f"Loading {os.path.basename(filepath)} (boost ×{boost})…", file=sys.stderr)
        raw = parse_bin(filepath)
        for key, moves in raw.items():
            # skip positions deeper than the depth limit
            depth = len(key.split()) if key else 0
            if depth > MAX_DEPTH_PLY:
                continue
            for mv, w in moves.items():
                merged[key][mv] += w * boost

    print(f"\nTotal unique positions after merge: {len(merged)}", file=sys.stderr)

    # ── 2. filter moves and positions ────────────────────────────────────────
    filtered = []
    for key, moves in merged.items():
        # drop weak moves
        good = {mv: w for mv, w in moves.items() if w >= MIN_WEIGHT}
        if not good:
            continue
        # keep top MAX_MOVES by weight
        top = sorted(good.items(), key=lambda x: -x[1])[:MAX_MOVES]
        total_weight = sum(w for _, w in top)
        filtered.append((key, top, total_weight))

    # sort: shallower + heavier positions first (these are the most useful)
    filtered.sort(key=lambda x: (len(x[0].split()) if x[0] else 0, -x[2]))

    # cap at MAX_POSITIONS
    if len(filtered) > MAX_POSITIONS:
        filtered = filtered[:MAX_POSITIONS]

    print(f"Positions after filter/cap: {len(filtered)}", file=sys.stderr)

    # ── 3. write ─────────────────────────────────────────────────────────────
    book_out = [(key, moves) for key, moves, _ in filtered]
    write_bin(book_out, OUTPUT_FILE)


if __name__ == "__main__":
    main()
