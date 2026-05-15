#!/usr/bin/env python3
"""
Converts openings.rs into a compact binary book.
Format per entry:
  - 2 bytes: key length
  - N bytes: key (UCI move sequence, e.g. "e2e4 e7e5")
  - 1 byte:  number of moves
  Per move:
    - 5 bytes: move string (padded, e.g. "g1f3\0")
    - 4 bytes: weight (u32 little-endian)
"""
import re, struct, sys

src = open(sys.argv[1] if len(sys.argv) > 1 else "../src/openings.rs").read()

pattern = r'm\.insert\("([^"]*)",\s*&\[([^\]]*)\]\)'
entries = re.findall(pattern, src)

out = open("../Openings/book.bin", "wb")
count = 0
for key, moves_str in entries:
    move_pairs = re.findall(r'\("([^"]+)",\s*(\d+)\)', moves_str)
    key_bytes = key.encode()
    out.write(struct.pack("<H", len(key_bytes)))
    out.write(key_bytes)
    out.write(struct.pack("<B", len(move_pairs)))
    for mv, wt in move_pairs:
        mv_bytes = mv.encode().ljust(5, b'\0')[:5]
        out.write(mv_bytes)
        out.write(struct.pack("<I", int(wt)))
    count += 1

out.close()
print(f"Done: {count} positions written to ../Openings/book.bin")
