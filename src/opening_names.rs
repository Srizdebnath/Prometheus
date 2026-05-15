// Auto-generated opening name lookup
// Maps UCI move sequences to standard opening names
// Uses longest-prefix matching: finds the deepest known position

/// Returns the name of the opening for the given move history (space-joined UCI moves).
/// Walks from the full sequence down to shorter prefixes, returning the deepest match.
pub fn get_opening_name(moves: &str) -> Option<&'static str> {
    let mut key = moves.trim();
    loop {
        if let Some(name) = lookup(key) {
            return Some(name);
        }
        match key.rfind(' ') {
            Some(idx) => key = &key[..idx],
            None => {
                // try single move
                return lookup(key);
            }
        }
    }
}

#[allow(unreachable_patterns)]
fn lookup(key: &str) -> Option<&'static str> {
    match key {
        // ── Single first moves ────────────────────────────────────────────────
        "e2e4" => Some("King's Pawn Opening"),
        "d2d4" => Some("Queen's Pawn Opening"),
        "g1f3" => Some("Réti Opening"),
        "c2c4" => Some("English Opening"),
        "f2f4" => Some("Bird's Opening"),
        "b2b3" => Some("Nimzowitsch-Larsen Attack"),
        "g2g3" => Some("King's Fianchetto Opening"),
        "b2b4" => Some("Polish Opening"),
        "d2d3" => Some("King's Indian Attack"),
        "e2e3" => Some("Van't Kruijs Opening"),
        "c2c3" => Some("Saragossa Opening"),
        "f2f3" => Some("Barnes Opening"),
        "a2a3" => Some("Anderssen's Opening"),
        "a2a4" => Some("Ware Opening"),
        "h2h4" => Some("Kadas Opening"),
        "g2g4" => Some("Grob's Attack"),
        "b1c3" => Some("Dunst Opening"),
        "b1a3" => Some("Sodium Attack"),

        // ── e4 e5 lines ───────────────────────────────────────────────────────
        "e2e4 e7e5" => Some("Open Game"),
        "e2e4 e7e5 g1f3" => Some("King's Knight Opening"),
        "e2e4 e7e5 g1f3 b8c6" => Some("Three Knights / Ruy Lopez"),
        "e2e4 e7e5 g1f3 b8c6 f1b5" => Some("Ruy Lopez"),
        "e2e4 e7e5 g1f3 b8c6 f1b5 a7a6" => Some("Ruy Lopez: Morphy Defence"),
        "e2e4 e7e5 g1f3 b8c6 f1b5 a7a6 b5a4" => Some("Ruy Lopez: Morphy Defence"),
        "e2e4 e7e5 g1f3 b8c6 f1b5 a7a6 b5a4 g8f6" => Some("Ruy Lopez: Open"),
        "e2e4 e7e5 g1f3 b8c6 f1b5 a7a6 b5a4 f8e7" => Some("Ruy Lopez: Closed"),
        "e2e4 e7e5 g1f3 b8c6 f1b5 a7a6 b5a4 f8e7 e1g1" => Some("Ruy Lopez: Closed Main Line"),
        "e2e4 e7e5 g1f3 b8c6 f1b5 a7a6 b5a4 f8e7 e1g1 b7b5" => {
            Some("Ruy Lopez: Closed, Chigorin Defence")
        }
        "e2e4 e7e5 g1f3 b8c6 f1b5 a7a6 b5c6" => Some("Ruy Lopez: Exchange Variation"),
        "e2e4 e7e5 g1f3 b8c6 f1b5 g8f6" => Some("Ruy Lopez: Berlin Defence"),
        "e2e4 e7e5 g1f3 b8c6 f1b5 g8f6 e1g1" => Some("Ruy Lopez: Berlin Defence"),
        "e2e4 e7e5 g1f3 b8c6 f1b5 g8f6 e1g1 f6e4" => Some("Ruy Lopez: Berlin Defence, Rio Gambit"),
        "e2e4 e7e5 g1f3 b8c6 f1b5 f8c5" => Some("Ruy Lopez: Classical Defence"),
        "e2e4 e7e5 g1f3 b8c6 f1b5 d7d6" => Some("Ruy Lopez: Steinitz Defence"),
        "e2e4 e7e5 g1f3 b8c6 f1b5 g7g6" => Some("Ruy Lopez: Smyslov Defence"),
        "e2e4 e7e5 g1f3 b8c6 f1b5 b7b5" => Some("Ruy Lopez: Cozio Defence"),
        "e2e4 e7e5 g1f3 b8c6 f1b5 g8e7" => Some("Ruy Lopez: Cozio Defence"),
        "e2e4 e7e5 g1f3 b8c6 f1b5 a7a6 b5a4 g8f6 e1g1 f6e4 d2d4 b7b5 a4b3 d7d5 d4e5 f8e6" => {
            Some("Ruy Lopez: Open, Main Line")
        }
        "e2e4 e7e5 g1f3 b8c6 f1b5 a7a6 b5a4 g8f6 e1g1 f8e7 f1e1 b7b5 a4b3 d7d6 c2c3 e8g8" => {
            Some("Ruy Lopez: Closed, Main Line")
        }
        "e2e4 e7e5 g1f3 b8c6 f1b5 a7a6 b5a4 g8f6 e1g1 f8e7 f1e1 b7b5 a4b3 e8g8 c2c3 d7d5" => {
            Some("Ruy Lopez: Marshall Attack")
        }
        "e2e4 e7e5 g1f3 b8c6 f1b5 a7a6 b5a4 g8f6 e1g1 f8e7 f1e1 b7b5 a4b3 d7d6 c2c3 e8g8 h2h3 b8b8" => {
            Some("Ruy Lopez: Breyer Defence")
        }
        "e2e4 e7e5 g1f3 b8c6 f1b5 a7a6 b5a4 g8f6 e1g1 f8e7 f1e1 b7b5 a4b3 d7d6 c2c3 e8g8 h2h3 c8b7" => {
            Some("Ruy Lopez: Arkhangelsk Defence")
        }

        // Italian / Giuoco
        "e2e4 e7e5 g1f3 b8c6 f1c4" => Some("Italian Game"),
        "e2e4 e7e5 g1f3 b8c6 f1c4 f8c5" => Some("Giuoco Piano"),
        "e2e4 e7e5 g1f3 b8c6 f1c4 f8c5 c2c3" => Some("Giuoco Piano: Main Line"),
        "e2e4 e7e5 g1f3 b8c6 f1c4 f8c5 c2c3 g8f6" => Some("Giuoco Piano: Pianissimo"),
        "e2e4 e7e5 g1f3 b8c6 f1c4 f8c5 b2b4" => Some("Evans Gambit"),
        "e2e4 e7e5 g1f3 b8c6 f1c4 f8c5 b2b4 c5b4" => Some("Evans Gambit Accepted"),
        "e2e4 e7e5 g1f3 b8c6 f1c4 g8f6" => Some("Two Knights Defence"),
        "e2e4 e7e5 g1f3 b8c6 f1c4 g8f6 f3g5" => Some("Two Knights: Fried Liver Attack"),
        "e2e4 e7e5 g1f3 b8c6 f1c4 g8f6 d2d4" => Some("Two Knights: Max Lange Attack"),
        "e2e4 e7e5 g1f3 b8c6 f1c4 f8e7" => Some("Hungarian Defence"),

        // Scotch
        "e2e4 e7e5 g1f3 b8c6 d2d4" => Some("Scotch Game"),
        "e2e4 e7e5 g1f3 b8c6 d2d4 e5d4" => Some("Scotch Game"),
        "e2e4 e7e5 g1f3 b8c6 d2d4 e5d4 f3d4" => Some("Scotch Game"),
        "e2e4 e7e5 g1f3 b8c6 d2d4 e5d4 f3d4 f8c5" => Some("Scotch Game: Classical"),
        "e2e4 e7e5 g1f3 b8c6 d2d4 e5d4 f3d4 g8f6" => Some("Scotch Game: Schmidt Variation"),
        "e2e4 e7e5 g1f3 b8c6 d2d4 e5d4 f3d4 d8h4" => Some("Scotch Game: Haxo Gambit"),
        "e2e4 e7e5 g1f3 b8c6 d2d4 e5d4 c3d4" => Some("Scotch Gambit"),

        // King's Gambit
        "e2e4 e7e5 f2f4" => Some("King's Gambit"),
        "e2e4 e7e5 f2f4 e5f4" => Some("King's Gambit Accepted"),
        "e2e4 e7e5 f2f4 e5f4 g1f3" => Some("King's Gambit Accepted: Bishop's Gambit"),
        "e2e4 e7e5 f2f4 e5f4 f1c4" => Some("King's Gambit Accepted: Bishop's Gambit"),
        "e2e4 e7e5 f2f4 e5f4 g1f3 g7g5" => Some("King's Gambit Accepted: Fischer Defence"),
        "e2e4 e7e5 f2f4 d7d5" => Some("Falkbeer Counter-Gambit"),
        "e2e4 e7e5 f2f4 f8c5" => Some("King's Gambit Declined: Classical"),

        // Four Knights
        "e2e4 e7e5 g1f3 b8c6 b1c3" => Some("Three Knights Game"),
        "e2e4 e7e5 g1f3 b8c6 b1c3 g8f6" => Some("Four Knights Game"),
        "e2e4 e7e5 g1f3 b8c6 b1c3 g8f6 f1b5" => Some("Four Knights: Spanish"),
        "e2e4 e7e5 g1f3 b8c6 b1c3 g8f6 f1c4" => Some("Four Knights: Italian"),

        // Russian / Petroff
        "e2e4 e7e5 g1f3 g8f6" => Some("Russian Game (Petroff Defence)"),
        "e2e4 e7e5 g1f3 g8f6 f3e5" => Some("Russian Game: Classical Attack"),
        "e2e4 e7e5 g1f3 g8f6 d2d4" => Some("Russian Game: Modern Attack"),
        "e2e4 e7e5 g1f3 g8f6 f3e5 d7d6 e5f3 f6e4 d2d4" => {
            Some("Russian Game: Classical, Stafford Gambit")
        }

        // Latvian / Philidor / others
        "e2e4 e7e5 g1f3 f7f5" => Some("Latvian Gambit"),
        "e2e4 e7e5 g1f3 d7d6" => Some("Philidor Defence"),
        "e2e4 e7e5 g1f3 d7d6 d2d4 g8f6" => Some("Philidor Defence: Hanham Variation"),
        "e2e4 e7e5 b1c3" => Some("Vienna Game"),
        "e2e4 e7e5 b1c3 g8f6" => Some("Vienna Game: Falkbeer Variation"),
        "e2e4 e7e5 b1c3 f8c5" => Some("Vienna Game: Mieses Variation"),
        "e2e4 e7e5 f1c4" => Some("Bishop's Opening"),
        "e2e4 e7e5 f1c4 f8c5" => Some("Bishop's Opening: Classical"),
        "e2e4 e7e5 f1c4 g8f6" => Some("Bishop's Opening: Berlin Defence"),

        // ── Sicilian Defence ──────────────────────────────────────────────────
        "e2e4 c7c5" => Some("Sicilian Defence"),
        "e2e4 c7c5 g1f3" => Some("Sicilian Defence: Open"),
        "e2e4 c7c5 g1f3 d7d6" => Some("Sicilian Defence"),
        "e2e4 c7c5 g1f3 d7d6 d2d4" => Some("Sicilian Defence: Open"),
        "e2e4 c7c5 g1f3 d7d6 d2d4 c5d4 f3d4" => Some("Sicilian Defence: Open"),
        "e2e4 c7c5 g1f3 d7d6 d2d4 c5d4 f3d4 g8f6 b1c3 a7a6" => Some("Sicilian: Najdorf Variation"),
        "e2e4 c7c5 g1f3 d7d6 d2d4 c5d4 f3d4 g8f6 b1c3 a7a6 c1g5" => {
            Some("Sicilian: Najdorf, English Attack")
        }
        "e2e4 c7c5 g1f3 d7d6 d2d4 c5d4 f3d4 g8f6 b1c3 a7a6 c1e3" => {
            Some("Sicilian: Najdorf, English Attack")
        }
        "e2e4 c7c5 g1f3 d7d6 d2d4 c5d4 f3d4 g8f6 b1c3 a7a6 f2f3" => {
            Some("Sicilian: Najdorf, English Attack")
        }
        "e2e4 c7c5 g1f3 d7d6 d2d4 c5d4 f3d4 g8f6 b1c3 a7a6 g2g4" => {
            Some("Sicilian: Najdorf, Keres Attack")
        }
        "e2e4 c7c5 g1f3 d7d6 d2d4 c5d4 f3d4 g8f6 b1c3 a7a6 f1e2" => {
            Some("Sicilian: Najdorf, Classical")
        }
        "e2e4 c7c5 g1f3 d7d6 d2d4 c5d4 f3d4 g8f6 b1c3 a7a6 f2f4" => {
            Some("Sicilian: Najdorf, Four Pawns Attack")
        }
        "e2e4 c7c5 g1f3 d7d6 d2d4 c5d4 f3d4 g8f6 b1c3 g7g6" => Some("Sicilian: Dragon Variation"),
        "e2e4 c7c5 g1f3 d7d6 d2d4 c5d4 f3d4 g8f6 b1c3 g7g6 c1e3" => {
            Some("Sicilian: Dragon, Yugoslav Attack")
        }
        "e2e4 c7c5 g1f3 d7d6 d2d4 c5d4 f3d4 g8f6 b1c3 g7g6 f2f4" => {
            Some("Sicilian: Dragon, Levenfish Attack")
        }
        "e2e4 c7c5 g1f3 d7d6 d2d4 c5d4 f3d4 g8f6 b1c3 g7g6 g2g3" => {
            Some("Sicilian: Dragon, Fianchetto Variation")
        }
        "e2e4 c7c5 g1f3 d7d6 d2d4 c5d4 f3d4 g8f6 b1c3 e7e6" => {
            Some("Sicilian: Scheveningen Variation")
        }
        "e2e4 c7c5 g1f3 d7d6 d2d4 c5d4 f3d4 g8f6 b1c3 e7e6 c1e3" => {
            Some("Sicilian: Scheveningen, English Attack")
        }
        "e2e4 c7c5 g1f3 d7d6 d2d4 c5d4 f3d4 g8f6 b1c3 e7e6 g2g4" => {
            Some("Sicilian: Scheveningen, Keres Attack")
        }
        "e2e4 c7c5 g1f3 d7d6 d2d4 c5d4 f3d4 g8f6 b1c3 b8c6" => {
            Some("Sicilian: Classical Variation")
        }
        "e2e4 c7c5 g1f3 d7d6 d2d4 c5d4 f3d4 g8f6 b1c3 b8c6 c1g5" => Some("Sicilian: Rauzer Attack"),
        "e2e4 c7c5 g1f3 d7d6 d2d4 c5d4 f3d4 g8f6 b1c3 b8c6 f1c4" => Some("Sicilian: Sozin Attack"),
        "e2e4 c7c5 g1f3 b8c6" => Some("Sicilian Defence"),
        "e2e4 c7c5 g1f3 b8c6 d2d4 c5d4 f3d4" => Some("Sicilian: Open, Four Knights"),
        "e2e4 c7c5 g1f3 b8c6 d2d4 c5d4 f3d4 g7g6" => Some("Sicilian: Accelerated Dragon"),
        "e2e4 c7c5 g1f3 b8c6 d2d4 c5d4 f3d4 g7g6 c2c4" => {
            Some("Sicilian: Accelerated Dragon, Maroczy Bind")
        }
        "e2e4 c7c5 g1f3 b8c6 d2d4 c5d4 f3d4 e7e6" => Some("Sicilian: Taimanov Variation"),
        "e2e4 c7c5 g1f3 b8c6 d2d4 c5d4 f3d4 d8c7" => {
            Some("Sicilian: Taimanov, Bastrikov Variation")
        }
        "e2e4 c7c5 g1f3 e7e6" => Some("Sicilian Defence: French Variation"),
        "e2e4 c7c5 g1f3 e7e6 d2d4 c5d4 f3d4" => Some("Sicilian: Kan Variation"),
        "e2e4 c7c5 g1f3 e7e6 d2d4 c5d4 f3d4 a7a6" => Some("Sicilian: Kan Variation"),
        "e2e4 c7c5 g1f3 e7e6 d2d4 c5d4 f3d4 b8c6" => Some("Sicilian: Taimanov Variation"),
        "e2e4 c7c5 g1f3 e7e6 d2d4 c5d4 f3d4 g8f6 b1c3 d7d6" => {
            Some("Sicilian: Scheveningen Variation")
        }
        "e2e4 c7c5 g1f3 e7e6 d2d4 c5d4 f3d4 g8f6 b1c3 f8b4" => {
            Some("Sicilian: Löwenthal Variation")
        }
        "e2e4 c7c5 b1c3" => Some("Sicilian: Closed"),
        "e2e4 c7c5 b1c3 b8c6" => Some("Sicilian: Closed"),
        "e2e4 c7c5 b1c3 b8c6 g2g3" => Some("Sicilian: Closed, Fianchetto"),
        "e2e4 c7c5 c2c3" => Some("Sicilian: Alapin Variation"),
        "e2e4 c7c5 c2c3 d7d5" => Some("Sicilian: Alapin, d5 push"),
        "e2e4 c7c5 c2c3 g8f6" => Some("Sicilian: Alapin, Nimzowitsch-Rubinstein"),
        "e2e4 c7c5 d2d4" => Some("Sicilian: Morra Gambit"),
        "e2e4 c7c5 d2d4 c5d4 c2c3" => Some("Sicilian: Smith-Morra Gambit"),
        "e2e4 c7c5 d2d4 c5d4 c2c3 d4c3 b1c3" => Some("Sicilian: Smith-Morra Gambit Accepted"),
        "e2e4 c7c5 f2f4" => Some("Sicilian: Grand Prix Attack"),
        "e2e4 c7c5 g1f3 d7d6 d2d4 c5d4 f3d4 g8f6 b1c3 c8d7" => Some("Sicilian: English Attack"),
        "e2e4 c7c5 g1f3 d7d6 d2d4 c5d4 f3d4 g8f6 f3d4" => Some("Sicilian: Richter-Rauzer"),

        // ── French Defence ────────────────────────────────────────────────────
        "e2e4 e7e6" => Some("French Defence"),
        "e2e4 e7e6 d2d4" => Some("French Defence"),
        "e2e4 e7e6 d2d4 d7d5" => Some("French Defence"),
        "e2e4 e7e6 d2d4 d7d5 b1c3" => Some("French Defence: Classical"),
        "e2e4 e7e6 d2d4 d7d5 b1c3 g8f6" => Some("French Defence: Classical"),
        "e2e4 e7e6 d2d4 d7d5 b1c3 f8b4" => Some("French Defence: Winawer Variation"),
        "e2e4 e7e6 d2d4 d7d5 b1c3 f8b4 e4e5" => Some("French Defence: Winawer, Advance"),
        "e2e4 e7e6 d2d4 d7d5 b1c3 f8b4 a2a3" => Some("French Defence: Winawer, Poisoned Pawn"),
        "e2e4 e7e6 d2d4 d7d5 b1c3 b8c6" => Some("French Defence: Three Knights"),
        "e2e4 e7e6 d2d4 d7d5 b1d2" => Some("French Defence: Tarrasch Variation"),
        "e2e4 e7e6 d2d4 d7d5 b1d2 g8f6" => Some("French Defence: Tarrasch, Open"),
        "e2e4 e7e6 d2d4 d7d5 b1d2 c7c5" => Some("French Defence: Tarrasch, Guimard"),
        "e2e4 e7e6 d2d4 d7d5 b1d2 b8c6" => Some("French Defence: Tarrasch, Morozevich"),
        "e2e4 e7e6 d2d4 d7d5 e4e5" => Some("French Defence: Advance Variation"),
        "e2e4 e7e6 d2d4 d7d5 e4e5 c7c5" => Some("French Defence: Advance, Main Line"),
        "e2e4 e7e6 d2d4 d7d5 e4e5 c7c5 c2c3" => Some("French Defence: Advance, Milner-Barry"),
        "e2e4 e7e6 d2d4 d7d5 e4d5" => Some("French Defence: Exchange Variation"),
        "e2e4 e7e6 d2d4 d7d5 e4d5 e6d5" => Some("French Defence: Exchange Variation"),
        "e2e4 e7e6 d2d4 d7d5 b1c3 g8f6 c1g5" => Some("French Defence: Classical, Alekhine-Chatard"),
        "e2e4 e7e6 d2d4 d7d5 b1c3 g8f6 e4e5" => Some("French Defence: Steinitz Variation"),
        "e2e4 e7e6 d2d4 d7d5 b1c3 g8f6 e4e5 f6d7 f2f4" => {
            Some("French Defence: Steinitz, Boleslavsky")
        }
        "e2e4 e7e6 d2d4 d7d5 g1f3" => Some("French Defence: Two Knights"),
        "e2e4 e7e6 d2d4 d7d5 g1f3 g8f6 c1d2" => Some("French Defence: Two Knights, Rubinstein"),

        // ── Caro-Kann ─────────────────────────────────────────────────────────
        "e2e4 c7c6" => Some("Caro-Kann Defence"),
        "e2e4 c7c6 d2d4" => Some("Caro-Kann Defence"),
        "e2e4 c7c6 d2d4 d7d5" => Some("Caro-Kann Defence"),
        "e2e4 c7c6 d2d4 d7d5 b1c3" => Some("Caro-Kann: Classical"),
        "e2e4 c7c6 d2d4 d7d5 b1c3 d5e4 c3e4" => Some("Caro-Kann: Classical"),
        "e2e4 c7c6 d2d4 d7d5 b1c3 d5e4 c3e4 c8f5" => Some("Caro-Kann: Classical, Bishop System"),
        "e2e4 c7c6 d2d4 d7d5 b1c3 d5e4 c3e4 g8f6" => Some("Caro-Kann: Classical, Capablanca"),
        "e2e4 c7c6 d2d4 d7d5 b1c3 d5e4 c3e4 b8d7" => Some("Caro-Kann: Karpov Variation"),
        "e2e4 c7c6 d2d4 d7d5 e4e5" => Some("Caro-Kann: Advance Variation"),
        "e2e4 c7c6 d2d4 d7d5 e4e5 c8f5" => Some("Caro-Kann: Advance, Short System"),
        "e2e4 c7c6 d2d4 d7d5 e4e5 c8f5 g2g4" => Some("Caro-Kann: Advance, Bayonet Attack"),
        "e2e4 c7c6 d2d4 d7d5 e4d5" => Some("Caro-Kann: Exchange Variation"),
        "e2e4 c7c6 d2d4 d7d5 e4d5 c6d5" => Some("Caro-Kann: Panov-Botvinnik Attack"),
        "e2e4 c7c6 d2d4 d7d5 e4d5 c6d5 c2c4" => Some("Caro-Kann: Panov-Botvinnik Attack"),
        "e2e4 c7c6 d2d4 d7d5 b1d2" => Some("Caro-Kann: Tartakower Variation"),
        "e2e4 c7c6 d2d4 d7d5 b1d2 d5e4 d2e4" => Some("Caro-Kann: Tartakower, Main Line"),
        "e2e4 c7c6 g1f3" => Some("Caro-Kann: Two Knights"),
        "e2e4 c7c6 b1c3" => Some("Caro-Kann: Hillbilly Attack"),

        // ── Scandinavian ──────────────────────────────────────────────────────
        "e2e4 d7d5" => Some("Scandinavian Defence"),
        "e2e4 d7d5 e4d5" => Some("Scandinavian Defence"),
        "e2e4 d7d5 e4d5 d8d5" => Some("Scandinavian: Main Line"),
        "e2e4 d7d5 e4d5 d8d5 b1c3" => Some("Scandinavian: Main Line"),
        "e2e4 d7d5 e4d5 d8d5 b1c3 d5a5" => Some("Scandinavian: Mieses-Kotroc"),
        "e2e4 d7d5 e4d5 d8d5 b1c3 d5d6" => Some("Scandinavian: Gubinsky-Melts"),
        "e2e4 d7d5 e4d5 g8f6" => Some("Scandinavian: Modern Variation"),
        "e2e4 d7d5 e4d5 g8f6 d2d4" => Some("Scandinavian: Modern, Gipslis"),

        // ── Pirc / Modern ─────────────────────────────────────────────────────
        "e2e4 d7d6" => Some("Pirc Defence"),
        "e2e4 d7d6 d2d4 g8f6" => Some("Pirc Defence"),
        "e2e4 d7d6 d2d4 g8f6 b1c3" => Some("Pirc Defence"),
        "e2e4 d7d6 d2d4 g8f6 b1c3 g7g6" => Some("Pirc Defence: Classical"),
        "e2e4 d7d6 d2d4 g8f6 b1c3 g7g6 f2f4" => Some("Pirc Defence: Austrian Attack"),
        "e2e4 d7d6 d2d4 g8f6 b1c3 g7g6 g1f3" => Some("Pirc Defence: Classical"),
        "e2e4 d7d6 d2d4 g8f6 b1c3 g7g6 c1e3" => Some("Pirc Defence: 150 Attack"),
        "e2e4 g7g6" => Some("Modern Defence"),
        "e2e4 g7g6 d2d4 f8g7" => Some("Modern Defence"),
        "e2e4 g7g6 d2d4 f8g7 b1c3" => Some("Modern Defence: Standard"),

        // ── Alekhine ──────────────────────────────────────────────────────────
        "e2e4 g8f6" => Some("Alekhine's Defence"),
        "e2e4 g8f6 e4e5" => Some("Alekhine's Defence: Advance Variation"),
        "e2e4 g8f6 e4e5 f6d5" => Some("Alekhine's Defence: Advance"),
        "e2e4 g8f6 e4e5 f6d5 d2d4 d7d6" => Some("Alekhine's Defence: Modern"),
        "e2e4 g8f6 e4e5 f6d5 d2d4 d7d6 g1f3" => Some("Alekhine's Defence: Modern, Main Line"),
        "e2e4 g8f6 e4e5 f6d5 c2c4 d5b6 d2d4" => Some("Alekhine's Defence: Four Pawns Attack"),
        "e2e4 g8f6 b1c3" => Some("Alekhine's Defence: Scandinavian"),

        // ── Owen / others ─────────────────────────────────────────────────────
        "e2e4 b7b6" => Some("Owen's Defence"),
        "e2e4 b7b5" => Some("Polish Defence"),
        "e2e4 f7f5" => Some("Fred Defence"),
        "e2e4 f7f6" => Some("Barnes Defence"),
        "e2e4 a7a6" => Some("St. George Defence"),
        "e2e4 g7g5" => Some("Borg Defence"),
        "e2e4 h7h6" => Some("Carr's Defence"),
        "e2e4 b8c6" => Some("Nimzowitsch Defence"),
        "e2e4 b8c6 d2d4 d7d5" => Some("Nimzowitsch Defence: Main Line"),

        // ── d4 openings ───────────────────────────────────────────────────────
        "d2d4 d7d5" => Some("Queen's Pawn: Closed"),
        "d2d4 d7d5 c2c4" => Some("Queen's Gambit"),
        "d2d4 d7d5 c2c4 d5c4" => Some("Queen's Gambit Accepted"),
        "d2d4 d7d5 c2c4 d5c4 g1f3" => Some("Queen's Gambit Accepted: Classical"),
        "d2d4 d7d5 c2c4 d5c4 e2e4" => Some("Queen's Gambit Accepted: Central Variation"),
        "d2d4 d7d5 c2c4 e7e6" => Some("Queen's Gambit Declined"),
        "d2d4 d7d5 c2c4 e7e6 b1c3" => Some("Queen's Gambit Declined"),
        "d2d4 d7d5 c2c4 e7e6 b1c3 g8f6" => Some("Queen's Gambit Declined"),
        "d2d4 d7d5 c2c4 e7e6 b1c3 g8f6 c1g5" => Some("Queen's Gambit Declined: Orthodox"),
        "d2d4 d7d5 c2c4 e7e6 b1c3 g8f6 c1g5 f8e7" => {
            Some("Queen's Gambit Declined: Orthodox, Main Line")
        }
        "d2d4 d7d5 c2c4 e7e6 b1c3 g8f6 c1g5 h7h6" => {
            Some("Queen's Gambit Declined: Orthodox, Lasker")
        }
        "d2d4 d7d5 c2c4 e7e6 b1c3 g8f6 c1g5 b8d7" => {
            Some("Queen's Gambit Declined: Cambridge Springs")
        }
        "d2d4 d7d5 c2c4 e7e6 b1c3 g8f6 c1f4" => Some("Queen's Gambit Declined: Vienna"),
        "d2d4 d7d5 c2c4 e7e6 b1c3 g8f6 g1f3" => Some("Queen's Gambit Declined: Three Knights"),
        "d2d4 d7d5 c2c4 e7e6 b1c3 g8f6 g1f3 f8e7 c1f4" => Some("Queen's Gambit Declined: Vienna"),
        "d2d4 d7d5 c2c4 e7e6 b1c3 g8f6 g1f3 f8b4" => Some("Queen's Gambit Declined: Ragozin"),
        "d2d4 d7d5 c2c4 e7e6 g1f3" => Some("Queen's Gambit Declined: Tartarsch"),
        "d2d4 d7d5 c2c4 e7e6 g1f3 g8f6 b1c3 c7c5" => {
            Some("Queen's Gambit Declined: Tarrasch Defence")
        }
        "d2d4 d7d5 c2c4 c7c6" => Some("Slav Defence"),
        "d2d4 d7d5 c2c4 c7c6 g1f3" => Some("Slav Defence"),
        "d2d4 d7d5 c2c4 c7c6 g1f3 g8f6" => Some("Slav Defence"),
        "d2d4 d7d5 c2c4 c7c6 g1f3 g8f6 b1c3" => Some("Slav Defence: Main Line"),
        "d2d4 d7d5 c2c4 c7c6 g1f3 g8f6 b1c3 d5c4" => Some("Slav Defence: Accepted"),
        "d2d4 d7d5 c2c4 c7c6 g1f3 g8f6 b1c3 e7e6" => Some("Semi-Slav Defence"),
        "d2d4 d7d5 c2c4 c7c6 g1f3 g8f6 b1c3 e7e6 c1g5" => Some("Semi-Slav: Anti-Moscow Gambit"),
        "d2d4 d7d5 c2c4 c7c6 g1f3 g8f6 b1c3 e7e6 e2e3" => Some("Semi-Slav: Meran Variation"),
        "d2d4 d7d5 c2c4 c7c6 g1f3 g8f6 b1c3 e7e6 e2e3 b8d7 f1d3 d5c4 d3c4 b7b5" => {
            Some("Semi-Slav: Meran, Reynolds' Attack")
        }
        "d2d4 d7d5 c2c4 c7c6 g1f3 g8f6 b1c3 e7e6 e2e4" => Some("Semi-Slav: Botvinnik Variation"),
        "d2d4 d7d5 c2c4 c7c6 b1c3 g8f6 g1f3 e7e6 c1g5" => Some("Semi-Slav: Anti-Moscow"),
        "d2d4 d7d5 c2c4 c7c6 e2e4" => Some("Slav: Slav-Indian"),
        "d2d4 d7d5 c2c4 d5c4 g1f3 g8f6 e2e3" => Some("Queen's Gambit Accepted: Normal"),
        "d2d4 d7d5 c2c4 e7e6 g1f3 c7c5" => Some("Tarrasch Defence"),
        "d2d4 d7d5 c2c4 e7e6 g1f3 g8f6 g2g3" => Some("Catalan Opening"),
        "d2d4 d7d5 c2c4 e7e6 g1f3 g8f6 g2g3 f8e7" => Some("Catalan: Closed"),
        "d2d4 d7d5 c2c4 e7e6 g1f3 g8f6 g2g3 d5c4" => Some("Catalan: Open"),
        "d2d4 d7d5 c2c4 e7e6 g1f3 g8f6 g2g3 d5c4 f1g2 b7b5" => Some("Catalan: Open, Main Line"),
        "d2d4 d7d5 c2c4 e7e6 b1c3 c7c5" => Some("Tarrasch Defence: Classical"),
        "d2d4 d7d5 c2c4 e7e6 b1c3 c7c6" => Some("Semi-Slav Defence"),
        "d2d4 d7d5 c2c4 e7e6 b1c3 f8b4" => Some("Nimzo-QGD"),
        "d2d4 d7d5 c1f4" => Some("London System"),
        "d2d4 d7d5 c1f4 g8f6" => Some("London System"),
        "d2d4 d7d5 c1f4 g8f6 e2e3" => Some("London System"),
        "d2d4 d7d5 c1f4 c7c5" => Some("London System: Anti-London"),
        "d2d4 d7d5 g1f3" => Some("Queen's Pawn: Torre Attack"),
        "d2d4 d7d5 g1f3 g8f6 c1g5" => Some("Torre Attack"),
        "d2d4 d7d5 g1f3 g8f6 c1f4" => Some("London System"),
        "d2d4 d7d5 g1f3 g8f6 e2e3" => Some("Colle System"),
        "d2d4 d7d5 g1f3 g8f6 e2e3 e7e6 f1d3" => Some("Colle System: Main Line"),
        "d2d4 d7d5 g1f3 g8f6 c2c4" => Some("Queen's Gambit"),
        "d2d4 d7d5 b1c3" => Some("Veresov Attack"),
        "d2d4 d7d5 b1c3 g8f6 c1g5" => Some("Veresov Attack: Main Line"),
        "d2d4 d7d5 e2e4" => Some("Blackmar-Diemer Gambit"),
        "d2d4 d7d5 e2e4 d5e4 b1c3" => Some("Blackmar-Diemer Gambit"),
        "d2d4 d7d5 e2e4 d5e4 b1c3 g8f6 f2f3 e4f3 d1f3" => Some("Blackmar-Diemer: Euwe Defence"),

        // ── Knight Indian systems ─────────────────────────────────────────────
        "d2d4 g8f6" => Some("Indian Defence"),
        "d2d4 g8f6 c2c4" => Some("Indian Defence"),
        "d2d4 g8f6 c2c4 e7e6" => Some("Indian Defence: East Indian"),
        "d2d4 g8f6 c2c4 e7e6 b1c3 f8b4" => Some("Nimzo-Indian Defence"),
        "d2d4 g8f6 c2c4 e7e6 b1c3 f8b4 e2e3" => Some("Nimzo-Indian: Rubinstein Variation"),
        "d2d4 g8f6 c2c4 e7e6 b1c3 f8b4 e2e3 e8g8 f1d3" => {
            Some("Nimzo-Indian: Rubinstein, Main Line")
        }
        "d2d4 g8f6 c2c4 e7e6 b1c3 f8b4 d1c2" => Some("Nimzo-Indian: Classical Variation"),
        "d2d4 g8f6 c2c4 e7e6 b1c3 f8b4 d1c2 e8g8" => Some("Nimzo-Indian: Classical, Noa Variation"),
        "d2d4 g8f6 c2c4 e7e6 b1c3 f8b4 a2a3" => Some("Nimzo-Indian: Sämisch Variation"),
        "d2d4 g8f6 c2c4 e7e6 b1c3 f8b4 a2a3 f8c3 b2c3" => Some("Nimzo-Indian: Sämisch, Main Line"),
        "d2d4 g8f6 c2c4 e7e6 b1c3 f8b4 g1f3" => Some("Nimzo-Indian: Three Knights"),
        "d2d4 g8f6 c2c4 e7e6 b1c3 f8b4 g1e2" => Some("Nimzo-Indian: Romanishin-Kasparov"),
        "d2d4 g8f6 c2c4 e7e6 b1c3 f8b4 f2f3" => Some("Nimzo-Indian: Leningrad Variation"),
        "d2d4 g8f6 c2c4 e7e6 g1f3 d7d5" => Some("Queen's Gambit Declined"),
        "d2d4 g8f6 c2c4 e7e6 g1f3 f8b4" => Some("Bogo-Indian Defence"),
        "d2d4 g8f6 c2c4 e7e6 g1f3 f8b4 c1d2" => Some("Bogo-Indian: Grünfeld Variation"),
        "d2d4 g8f6 c2c4 e7e6 g1f3 f8b4 b1d2" => Some("Bogo-Indian: Vitolins Variation"),
        "d2d4 g8f6 c2c4 e7e6 g1f3 b7b6" => Some("Queen's Indian Defence"),
        "d2d4 g8f6 c2c4 e7e6 g1f3 b7b6 b1c3" => Some("Queen's Indian: Classical Variation"),
        "d2d4 g8f6 c2c4 e7e6 g1f3 b7b6 g2g3" => Some("Queen's Indian: Fianchetto"),
        "d2d4 g8f6 c2c4 e7e6 g1f3 b7b6 g2g3 c8b7 f1g2 f8e7 e1g1 e8g8" => {
            Some("Queen's Indian: Fianchetto, Main Line")
        }
        "d2d4 g8f6 c2c4 e7e6 g1f3 b7b6 a2a3" => Some("Queen's Indian: Petrosian System"),
        "d2d4 g8f6 c2c4 e7e6 g1f3 b7b6 e2e3" => Some("Queen's Indian: Nimzowitsch"),
        "d2d4 g8f6 c2c4 g7g6" => Some("King's Indian Defence"),
        "d2d4 g8f6 c2c4 g7g6 b1c3" => Some("King's Indian Defence"),
        "d2d4 g8f6 c2c4 g7g6 b1c3 f8g7" => Some("King's Indian Defence"),
        "d2d4 g8f6 c2c4 g7g6 b1c3 f8g7 e2e4" => Some("King's Indian Defence: Main Line"),
        "d2d4 g8f6 c2c4 g7g6 b1c3 f8g7 e2e4 d7d6" => Some("King's Indian Defence: Main Line"),
        "d2d4 g8f6 c2c4 g7g6 b1c3 f8g7 e2e4 d7d6 g1f3" => Some("King's Indian Defence: Main Line"),
        "d2d4 g8f6 c2c4 g7g6 b1c3 f8g7 e2e4 d7d6 g1f3 e8g8" => {
            Some("King's Indian Defence: Classical")
        }
        "d2d4 g8f6 c2c4 g7g6 b1c3 f8g7 e2e4 d7d6 g1f3 e8g8 f1e2" => {
            Some("King's Indian: Classical Variation")
        }
        "d2d4 g8f6 c2c4 g7g6 b1c3 f8g7 e2e4 d7d6 g1f3 e8g8 f1e2 e7e5" => {
            Some("King's Indian: Classical, Main Line")
        }
        "d2d4 g8f6 c2c4 g7g6 b1c3 f8g7 e2e4 d7d6 f2f4" => Some("King's Indian: Four Pawns Attack"),
        "d2d4 g8f6 c2c4 g7g6 b1c3 f8g7 e2e4 d7d6 f2f3" => Some("King's Indian: Sämisch Variation"),
        "d2d4 g8f6 c2c4 g7g6 b1c3 f8g7 e2e4 d7d6 c1e3" => Some("King's Indian: Averbakh Variation"),
        "d2d4 g8f6 c2c4 g7g6 b1c3 f8g7 e2e4 d7d6 c1e3 e8g8 d1d2" => {
            Some("King's Indian: Averbakh, Main Line")
        }
        "d2d4 g8f6 c2c4 g7g6 b1c3 f8g7 e2e4 e7e5" => Some("King's Indian: Four Pawns Attack"),
        "d2d4 g8f6 c2c4 g7g6 g2g3 f8g7 f1g2" => Some("King's Indian: Fianchetto"),
        "d2d4 g8f6 c2c4 g7g6 g2g3 f8g7 f1g2 e8g8" => Some("King's Indian: Fianchetto, Main Line"),
        "d2d4 g8f6 c2c4 g7g6 b1c3 d7d5" => Some("Grünfeld Defence"),
        "d2d4 g8f6 c2c4 g7g6 b1c3 d7d5 c4d5 f6d5" => Some("Grünfeld Defence: Exchange"),
        "d2d4 g8f6 c2c4 g7g6 b1c3 d7d5 c4d5 f6d5 e2e4 d5c3 b2c3 f8g7" => {
            Some("Grünfeld Defence: Exchange Variation")
        }
        "d2d4 g8f6 c2c4 g7g6 b1c3 d7d5 g1f3" => Some("Grünfeld Defence: Three Knights"),
        "d2d4 g8f6 c2c4 g7g6 b1c3 d7d5 d1b3" => Some("Grünfeld Defence: Taimanov"),
        "d2d4 g8f6 c2c4 g7g6 b1c3 d7d5 c1f4" => Some("Grünfeld Defence: Brinckmann"),
        "d2d4 g8f6 c2c4 c7c5" => Some("Benoni Defence"),
        "d2d4 g8f6 c2c4 c7c5 d4d5" => Some("Modern Benoni"),
        "d2d4 g8f6 c2c4 c7c5 d4d5 e7e6" => Some("Modern Benoni"),
        "d2d4 g8f6 c2c4 c7c5 d4d5 e7e6 b1c3 e6d5 c4d5" => Some("Modern Benoni: Main Line"),
        "d2d4 g8f6 c2c4 c7c5 d4d5 b7b5" => Some("Benko Gambit"),
        "d2d4 g8f6 c2c4 c7c5 d4d5 b7b5 c4b5 a7a6" => Some("Benko Gambit Accepted"),

        // ── Dutch Defence ────────────────────────────────────────────────────
        "d2d4 f7f5" => Some("Dutch Defence"),
        "d2d4 f7f5 g2g3" => Some("Dutch Defence: Fianchetto"),
        "d2d4 f7f5 c2c4" => Some("Dutch Defence"),
        "d2d4 f7f5 c2c4 g8f6" => Some("Dutch Defence"),
        "d2d4 f7f5 c2c4 g8f6 g2g3" => Some("Dutch: Leningrad"),
        "d2d4 f7f5 c2c4 g8f6 g2g3 g7g6 f1g2 f8g7" => Some("Dutch: Leningrad, Main Line"),
        "d2d4 f7f5 c2c4 g8f6 b1c3 e7e6" => Some("Dutch: Classical"),
        "d2d4 f7f5 c2c4 g8f6 b1c3 e7e6 g2g3" => Some("Dutch: Classical, Fianchetto"),
        "d2d4 f7f5 c2c4 e7e6 b1c3 g8f6 g2g3" => Some("Dutch: Classical"),
        "d2d4 e7e6 c2c4 f7f5" => Some("Dutch Defence: Reversed"),

        // ── Grünfeld via g1f3 ────────────────────────────────────────────────
        "d2d4 g8f6 g1f3 g7g6 c2c4 f8g7 b1c3 d7d5" => Some("Grünfeld Defence: Neo-Grünfeld"),

        // ── London via g1f3 ──────────────────────────────────────────────────
        "d2d4 g8f6 g1f3 e7e6 c1f4" => Some("London System"),
        "d2d4 g8f6 g1f3 e7e6 e2e3" => Some("Colle System"),
        "d2d4 g8f6 g1f3 g7g6 c2c4" => Some("King's Indian via Réti"),
        "d2d4 g8f6 g1f3 d7d5 c2c4 c7c6 b1c3" => Some("Slav via Réti"),
        "d2d4 g8f6 g1f3 d7d5 c1f4" => Some("London System"),
        "d2d4 g8f6 g1f3 d7d5 e2e3" => Some("Colle System"),

        // ── Torre Attack ─────────────────────────────────────────────────────
        "d2d4 g8f6 g1f3 e7e6 c1g5" => Some("Torre Attack"),
        "d2d4 g8f6 g1f3 g7g6 c1g5" => Some("Torre Attack: Fianchetto"),
        "d2d4 g8f6 c2c4 e7e6 g1f3 d7d5 b1c3 b8d7" => Some("QGD: Manhattan Variation"),

        // ── Old Indian / Benoni ───────────────────────────────────────────────
        "d2d4 g8f6 c2c4 d7d6" => Some("Old Indian Defence"),
        "d2d4 g8f6 c2c4 d7d6 b1c3 e7e5" => Some("Old Indian Defence: Main Line"),
        "d2d4 d7d6" => Some("Pirc Defence"),
        "d2d4 c7c5" => Some("Old Benoni"),
        "d2d4 e7e6" => Some("Indian Game"),
        "d2d4 b8c6" => Some("Nimzowitsch Defence"),
        "d2d4 g7g6" => Some("Modern Defence"),
        "d2d4 b7b6" => Some("English Defence"),

        // ── English Opening ───────────────────────────────────────────────────
        "c2c4" => Some("English Opening"),
        "c2c4 e7e5" => Some("English Opening: King's English"),
        "c2c4 e7e5 b1c3" => Some("English: King's, Three Knights"),
        "c2c4 e7e5 b1c3 g8f6" => Some("English: King's, Three Knights"),
        "c2c4 e7e5 b1c3 g8f6 g1f3 b8c6" => Some("English: Four Knights"),
        "c2c4 e7e5 b1c3 g8f6 g1f3 b8c6 d2d4" => Some("English: Four Knights, Nimzowitsch"),
        "c2c4 e7e5 b1c3 f8c5" => Some("English: King's, Reversed Sicilian"),
        "c2c4 e7e5 b1c3 b8c6" => Some("English: King's, Reversed Sicilian"),
        "c2c4 e7e5 g2g3" => Some("English: King's, Fianchetto"),
        "c2c4 e7e5 g2g3 g8f6 f1g2 f8c5" => Some("English: King's, Reversed Dragon"),
        "c2c4 g8f6" => Some("English Opening"),
        "c2c4 g8f6 b1c3" => Some("English: Anglo-Indian"),
        "c2c4 g8f6 b1c3 e7e5" => Some("English: Reversed Sicilian"),
        "c2c4 g8f6 b1c3 d7d5" => Some("English: Anglo-Indian, QGD"),
        "c2c4 g8f6 b1c3 d7d5 d2d4" => Some("English: Anglo-QGD"),
        "c2c4 g8f6 g1f3 g7g6" => Some("English: Anglo-Indian, Fianchetto"),
        "c2c4 g8f6 g1f3 e7e6 b1c3" => Some("English: Anglo-Indian"),
        "c2c4 g8f6 g1f3 b7b6" => Some("English: Anglo-Indian, Queen's Indian"),
        "c2c4 c7c5" => Some("English: Symmetrical"),
        "c2c4 c7c5 b1c3" => Some("English: Symmetrical"),
        "c2c4 c7c5 b1c3 b8c6" => Some("English: Symmetrical, Four Knights"),
        "c2c4 c7c5 g1f3 g8f6 b1c3" => Some("English: Symmetrical, Main Line"),
        "c2c4 e7e6" => Some("English: Agincourt"),
        "c2c4 e7e6 b1c3 d7d5" => Some("English: Agincourt, Main Line"),
        "c2c4 e7e6 b1c3 g8f6" => Some("English: Agincourt"),
        "c2c4 e7e6 g1f3 d7d5 b1c3 g8f6 d2d4" => Some("English/Catalan Hybrid"),
        "c2c4 c7c6" => Some("English: Caro-Kann Defensive"),
        "c2c4 g7g6" => Some("English: King's Fianchetto"),
        "c2c4 b8c6" => Some("English: Tübingen Variation"),

        // ── Réti Opening ──────────────────────────────────────────────────────
        "g1f3 d7d5" => Some("Réti Opening"),
        "g1f3 d7d5 c2c4" => Some("Réti: Main Line"),
        "g1f3 d7d5 c2c4 d5c4" => Some("Réti: Accepted"),
        "g1f3 d7d5 c2c4 e7e6" => Some("Réti: QGD Formation"),
        "g1f3 d7d5 c2c4 c7c6" => Some("Réti: Slav Formation"),
        "g1f3 d7d5 g2g3" => Some("Réti: Fianchetto"),
        "g1f3 d7d5 g2g3 g8f6 f1g2" => Some("Réti: King's Indian Attack"),
        "g1f3 g8f6" => Some("Réti Opening"),
        "g1f3 g8f6 c2c4 e7e6 g2g3" => Some("Réti: Catalan Formation"),
        "g1f3 g8f6 g2g3" => Some("King's Indian Attack"),
        "g1f3 g8f6 g2g3 g7g6 f1g2 f8g7" => Some("King's Indian Attack: Yugoslav"),
        "g1f3 c7c5" => Some("Réti: Symmetrical"),
        "g1f3 f7f5" => Some("Réti vs Dutch"),
        "g1f3 b8c6" => Some("Réti: Nimzowitsch"),
        "g1f3 e7e6 c2c4" => Some("English/Réti Hybrid"),

        // ── Bird's Opening ────────────────────────────────────────────────────
        "f2f4 d7d5" => Some("Bird's Opening: Dutch Variation"),
        "f2f4 d7d5 g1f3" => Some("Bird's Opening"),
        "f2f4 d7d5 e2e4" => Some("Bird's: From's Gambit Reversed"),
        "f2f4 e7e5" => Some("Bird's Opening: From's Gambit"),
        "f2f4 e7e5 f4e5 d7d6" => Some("Bird's: From's Gambit, Lasker"),
        "f2f4 g8f6" => Some("Bird's Opening"),
        "f2f4 c7c5 g1f3" => Some("Bird's: Schlechter Gambit"),

        // ── Polish Opening ────────────────────────────────────────────────────
        "b2b4 e7e5" => Some("Polish Opening: King's Indian Formation"),
        "b2b4 e7e5 c1b2" => Some("Polish Opening: Classical"),
        "b2b4 d7d5" => Some("Polish Opening: Schuhler Gambit"),
        "b2b4 g8f6" => Some("Polish Opening: Outflank"),
        "b2b4 c7c6" => Some("Polish Opening: Bugayev Attack"),

        // ── Nimzowitsch-Larsen ────────────────────────────────────────────────
        "b2b3 e7e5" => Some("Nimzowitsch-Larsen Attack: Classical"),
        "b2b3 d7d5" => Some("Nimzowitsch-Larsen Attack: Modern"),
        "b2b3 g8f6" => Some("Nimzowitsch-Larsen Attack: Indian"),
        "b2b3 e7e5 c1b2 b8c6 e2e3" => Some("Nimzowitsch-Larsen: Classical Main Line"),

        // ── King's Indian Attack ──────────────────────────────────────────────
        "g1f3 d7d5 g2g3 g8f6 f1g2 c7c5" => Some("King's Indian Attack: Sicilian"),
        "g1f3 d7d5 g2g3 g8f6 f1g2 e7e6 e1g1" => Some("King's Indian Attack"),
        "e2e4 e7e6 d2d3 d7d5 b1d2 g8f6 g1f3" => Some("King's Indian Attack vs French"),
        "e2e4 c7c5 g1f3 e7e6 d2d3" => Some("King's Indian Attack vs Sicilian"),

        // ── Grob / Unusual ────────────────────────────────────────────────────
        "g2g4 d7d5" => Some("Grob's Attack: Alapin Gambit"),
        "g2g4 e7e5" => Some("Grob's Attack"),

        _ => None,
    }
}
