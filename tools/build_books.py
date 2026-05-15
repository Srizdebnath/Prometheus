#!/usr/bin/env python3
"""
build_books.py — Download all PGN Mentor master games + openings,
                 then produce two binary files:
                   games.bin    — opening book built from master games
                   openings.bin — opening book built from opening files

Binary format (same for both):
  Per position entry:
    [2 bytes] key length (u16 LE)
    [N bytes] key = space-joined UCI moves, e.g. "e2e4 e7e5 g1f3"
    [1 byte]  number of candidate moves (max 255)
    Per candidate:
      [5 bytes] UCI move, null-padded (e.g. b"g1f3\x00")
      [4 bytes] weight / frequency (u32 LE)

Run:
  python3 build_books.py

Requirements: pip install requests chess
"""

import os, sys, struct, zipfile, io, re, time, urllib.request
from collections import defaultdict

try:
    import chess
    import chess.pgn
except ImportError:
    print("Installing python-chess...")
    os.system(f"{sys.executable} -m pip install chess --quiet")
    import chess
    import chess.pgn

try:
    import requests
except ImportError:
    print("Installing requests...")
    os.system(f"{sys.executable} -m pip install requests --quiet")
    import requests

# ── Config ────────────────────────────────────────────────────────────────────
MAX_OPENING_PLY = 20      # only record the first N half-moves per game
MIN_WEIGHT      = 2       # drop moves seen fewer than this many times
MAX_MOVES_PER_POS = 20    # keep at most this many candidates per position
CACHE_DIR       = os.path.expanduser("~/prometheus/pgn_cache")
OUT_DIR         = os.path.expanduser("~/prometheus/Openings")

# ── All player zip URLs ────────────────────────────────────────────────────────
PLAYER_ZIPS = [
    "Abdusattorov","Adams","Akobian","Akopian","Alburt","Alekhine","Alekseev",
    "Almasi","Anand","Anderssen","Andersson","Andreikin","Aronian","Ashley",
    "Averbakh","Azmaiparashvili","Bacrot","Bareev","BecerraRivero","Beliavsky",
    "Benjamin","Benko","Berliner","Bernstein","Bird","Bisguier","Blackburne",
    "Blatny","Bogoljubow","Boleslavsky","Bologan","Botvinnik","Breyer",
    "Bronstein","Browne","Bruzon","Bu","Byrne","Capablanca","Carlsen","Caruana",
    "Chiburdanidze","Chigorin","Christiansen","DeFirmian","DeLaBourdonnais",
    "Denker","Ding","DominguezPerez","Dreev","Duda","Dzindzichashvili",
    "Ehlvest","Eljanov","Erigaisi","Euwe","Evans","Fedorowicz","Fine",
    "Finegold","Firouzja","Fischer","Fishbein","Flohr","Gaprindashvili",
    "Gashimov","Gelfand","Geller","Georgiev","Giri","Gligoric","Goldin",
    "GrandaZuniga","Grischuk","Gukesh","Gulko","Gunsberg","GurevichD",
    "GurevichM","Harikrishna","Hort","Horwitz","Hou","Huebner","Ibragimov",
    "IllescasCordoba","Inarkiev","Ivanchuk","IvanovA","IvanovI","Ivkov",
    "Jakovenko","Janowski","Jobava","Jussupow","Kaidanov","Kamsky","Karjakin",
    "Karpov","Kasimdzhanov","Kasparov","Kavalek","Keres","Keymer","Khalifman",
    "Kholmov","Koneru","Korchnoi","Korobov","Kosteniuk","Kotov","Kramnik",
    "Krasenkow","Krush","Kudrin","Lahno","Larsen","Lasker","Lautier","Le",
    "Leko","Levenfish","Li","Lilienthal","Ljubojevic","Lputian","MacKenzie",
    "Malakhov","Mamedyarov","Maroczy","Marshall","McDonnell","McShane",
    "Mecking","Mikenas","Miles","Milov","Morozevich","Morphy","Motylev",
    "Movsesian","Muzychuk","Najdorf","Najer","Nakamura","Navara","Negi",
    "Nepomniachtchi","Ni","Nielsen","Nikolic","Nimzowitsch","Nisipeanu",
    "Novikov","Nunn","Olafsson","Oll","Onischuk","Pachman","Paehtz","Panno",
    "Paulsen","Petrosian","Philidor","Pillsbury","Pilnik","PolgarJ","PolgarS",
    "PolgarZ","Polugaevsky","Ponomariov","Portisch","Praggnanandhaa","Psakhis",
    "Quinteros","Radjabov","Rapport","Reshevsky","Reti","Ribli","Rohde",
    "Rubinstein","Rublevsky","Saemisch","Sakaev","Salov","Sasikiran",
    "Schlechter","Seirawan","Serper","Shabalov","Shamkovich","Shirov","Short",
    "Shulman","Smirin","Smyslov","So","Sokolov","Soltis","Spassky","Speelman",
    "Spielmann","Stahlberg","Staunton","Stefanova","Stein","Steinitz","Suetin",
    "SultanKhan","Sutovsky","Svidler","Szabo","Taimanov","Tal","Tarrasch",
    "Tartakower","Teichmann","Timman","Tiviakov","Tkachiev","Tomashevsky",
    "Topalov","TorreRepetto","Uhlmann","Unzicker","Ushenina","VachierLagrave",
    "Vaganian","VallejoPons","VanWely","Vitiugov","Volokitin","Waitzkin",
    "Wang","WangH","Wei","Winawer","Wojtaszek","Wojtkiewicz","Wolff","Xie",
    "Xu","Ye","Yermolinsky","Yu","Yudasin","Zhu","Zukertort","Zvjaginsev",
]

# ── All opening zip names ──────────────────────────────────────────────────────
OPENING_ZIPS = [
    "Modern","SemiBenoni","Trompowsky2Ne4","Trompowsky2e6","TrompowskyOther",
    "Torre2e6","Torre2g6","London2e6","London2g6","Catalan3Bb4","Catalan3c5",
    "CatalanOpen","CatalanClosed","BlackKnightTango","BudapestGambit",
    "OldIndian","CzechBenoni","BenkoGambit","ModernBenoni6Nf3","ModernBenoni6e4",
    "DutchLeningrad","DutchClassical","Dutch3Nc3","DutchOther","GrunfeldFianchetto",
    "Grunfeld4Nf3","GrunfeldOther","GrunfeldExchange","Bogo4Bd2","Bogo4Nbd2",
    "QID4a3","QID4Nc3","QID4e3","QIDOther","QID4g3-Ba6","QID4g3Other",
    "Nimzo4Nf3","Nimzo4e3","Nimzo4Qc2","NimzoSamisch","NimzoOther",
    "KID4Nf3","KID4e4","KIDClassical","KIDSamisch","KIDFour","KIDAverbakh",
    "KIDOther","KingIndianAttack","Slav4Nc3","SlavaSlav",
    "SlavExchange","SlavOther","QGD4Bg5","QGD4Nf3","QGDOther","QGDTarrasch",
    "QGA","QGDExchange","QueensGambitAccepted",
    "Symmetrical4d3","Symmetrical4Nf3","SymmetricalOther","SymmetricalDouble",
    "EnglishOther","EnglishSymmetrical",
    "RetiMain","RetiOther",
    "SicilianNajdorf6Bg5","SicilianNajdorf6Be3","SicilianNajdorf6f4",
    "SicilianNajdorfOther","SicilianDragon","SicilianClassical","SicilianScheveningen",
    "SicilianSozin","SicilianOpen","SicilianClosed","SicilianKan","SicilianSmith",
    "SicilianOther","SicilianRauzer","SicilianAccelerated",
    "FrenchTarrasch","FrenchWinawer","FrenchAdvance","FrenchExchange",
    "FrenchClassical","FrenchRubinstein","FrenchOther",
    "CaroKannAdvance","CaroKannClassical","CaroKannExchange",
    "CaroKannPanov","CaroKannOther",
    "PircAustrian","PircOther",
    "SpanishClosedBreyer","SpanishClosedChigorin","SpanishClosedOther",
    "SpanishExchange","SpanishOpen","SpanishOther","SpanishMarshall",
    "ItalianGiuoco","ItalianEvans","ItalianOther","TwoKnights",
    "ScotchGame","Petroff","KingsGambit","FourKnights",
    "ScandinavianMain","ScandinavianOther","AlekhineMain","AlekhineOther",
]

BASE_PLAYER  = "https://www.pgnmentor.com/players/{}.zip"
BASE_OPENING = "https://www.pgnmentor.com/openings/{}.zip"

# ── Helpers ────────────────────────────────────────────────────────────────────
def ensure_dirs():
    os.makedirs(CACHE_DIR + "/players",  exist_ok=True)
    os.makedirs(CACHE_DIR + "/openings", exist_ok=True)
    os.makedirs(OUT_DIR, exist_ok=True)

def download(url, dest):
    if os.path.exists(dest):
        return True
    try:
        r = requests.get(url, timeout=30, headers={"User-Agent": "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 Chrome/120.0.0.0 Safari/537.36"})
        if r.status_code == 200:
            with open(dest, "wb") as f:
                f.write(r.content)
            return True
        return False
    except Exception:
        return False

def pgn_from_zip(path):
    """Yield PGN text blocks from a zip file."""
    try:
        with zipfile.ZipFile(path) as zf:
            for name in zf.namelist():
                if name.lower().endswith(".pgn"):
                    with zf.open(name) as f:
                        yield f.read().decode("latin-1", errors="replace")
    except Exception:
        pass

def board_to_uci_key(board):
    """Return space-joined UCI moves from the start to current position."""
    moves = []
    b = board.copy()
    b.reset()
    for move in board.move_stack:
        moves.append(move.uci())
        b.push(move)
    return " ".join(moves)

def process_pgn_text(text, book, max_ply):
    """Parse PGN text and accumulate move frequencies into book dict."""
    pgn_io = io.StringIO(text)
    while True:
        try:
            game = chess.pgn.read_game(pgn_io)
        except Exception:
            break
        if game is None:
            break
        board = game.board()
        node = game
        ply = 0
        key_parts = []
        while node.variations and ply < max_ply:
            move = node.variations[0].move
            uci  = move.uci()
            key  = " ".join(key_parts)          # position BEFORE this move
            book[key][uci] += 1
            key_parts.append(uci)
            board.push(move)
            node = node.variations[0]
            ply += 1

def write_bin(book, path, min_weight, max_moves):
    """Write the book dict to a binary file."""
    total_pos = 0
    total_moves = 0
    with open(path, "wb") as f:
        for key, moves in book.items():
            # Filter and sort by weight descending
            filtered = sorted(
                [(mv, w) for mv, w in moves.items() if w >= min_weight],
                key=lambda x: -x[1]
            )[:max_moves]
            if not filtered:
                continue
            key_bytes = key.encode("ascii")
            f.write(struct.pack("<H", len(key_bytes)))
            f.write(key_bytes)
            f.write(struct.pack("<B", len(filtered)))
            for mv, weight in filtered:
                mv_bytes = mv.encode("ascii").ljust(5, b"\x00")[:5]
                f.write(mv_bytes)
                f.write(struct.pack("<I", weight))
            total_pos += 1
            total_moves += len(filtered)
    size_mb = os.path.getsize(path) / 1024 / 1024
    print(f"  → {path}")
    print(f"     {total_pos:,} positions, {total_moves:,} candidate moves, {size_mb:.1f} MB")

# ── Main ───────────────────────────────────────────────────────────────────────
def main():
    ensure_dirs()

    # ── 1. Download player zips ──────────────────────────────────────────────
    print(f"\n{'='*60}")
    print(f"Downloading {len(PLAYER_ZIPS)} player files...")
    print(f"{'='*60}")
    ok = fail = skip = 0
    for name in PLAYER_ZIPS:
        dest = f"{CACHE_DIR}/players/{name}.zip"
        if os.path.exists(dest):
            skip += 1
            continue
        url = BASE_PLAYER.format(name)
        if download(url, dest):
            ok += 1
            print(f"  ✓ {name}", flush=True)
        else:
            fail += 1
            print(f"  ✗ {name} (not found — skipped)", flush=True)
        time.sleep(0.15)   # be polite to the server
    print(f"Players: {ok} downloaded, {skip} cached, {fail} missing")

    # ── 2. Download opening zips ─────────────────────────────────────────────
    print(f"\n{'='*60}")
    print(f"Downloading {len(OPENING_ZIPS)} opening files...")
    print(f"{'='*60}")
    ok = fail = skip = 0
    for name in OPENING_ZIPS:
        dest = f"{CACHE_DIR}/openings/{name}.zip"
        if os.path.exists(dest):
            skip += 1
            continue
        url = BASE_OPENING.format(name)
        if download(url, dest):
            ok += 1
            print(f"  ✓ {name}", flush=True)
        else:
            fail += 1
            print(f"  ✗ {name} (not found — skipped)", flush=True)
        time.sleep(0.15)

    print(f"Openings: {ok} downloaded, {skip} cached, {fail} missing")

    # ── 3. Build games book from player PGNs ─────────────────────────────────
    print(f"\n{'='*60}")
    print("Building games.bin from master player games...")
    print(f"{'='*60}")
    games_book = defaultdict(lambda: defaultdict(int))
    n_games = 0
    for name in PLAYER_ZIPS:
        path = f"{CACHE_DIR}/players/{name}.zip"
        if not os.path.exists(path):
            continue
        for pgn_text in pgn_from_zip(path):
            process_pgn_text(pgn_text, games_book, MAX_OPENING_PLY)
            # count games roughly
            n_games += pgn_text.count("[Event ")
        print(f"  Processed {name} — {len(games_book):,} positions so far", flush=True)

    print(f"\nTotal games processed: ~{n_games:,}")
    print(f"Total positions before filtering: {len(games_book):,}")
    write_bin(games_book, f"{OUT_DIR}/games.bin", MIN_WEIGHT, MAX_MOVES_PER_POS)

    # ── 4. Build openings book from opening PGNs ──────────────────────────────
    print(f"\n{'='*60}")
    print("Building openings.bin from opening files...")
    print(f"{'='*60}")
    openings_book = defaultdict(lambda: defaultdict(int))
    for name in OPENING_ZIPS:
        path = f"{CACHE_DIR}/openings/{name}.zip"
        if not os.path.exists(path):
            continue
        for pgn_text in pgn_from_zip(path):
            process_pgn_text(pgn_text, openings_book, MAX_OPENING_PLY)
        print(f"  Processed {name} — {len(openings_book):,} positions so far", flush=True)

    print(f"\nTotal positions before filtering: {len(openings_book):,}")
    write_bin(openings_book, f"{OUT_DIR}/openings.bin", MIN_WEIGHT, MAX_MOVES_PER_POS)

    print(f"\n{'='*60}")
    print("Done! Files written to ~/prometheus/Openings/")
    print("  games.bin    — use for engine opening book")
    print("  openings.bin — use for opening explorer / theory")
    print(f"{'='*60}\n")

if __name__ == "__main__":
    main()
