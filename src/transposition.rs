use crate::board::Move;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum NodeType {
    Exact,
    LowerBound,
    UpperBound,
}

#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct TTEntry {
    pub key: u64,           // Zobrist hash of the position
    pub best_move: Move,    // Best move found in this position
    pub score: i16,         // Evaluation score
    pub depth: u8,          // Search depth at which this entry was created
    pub node_type: u8,      // NodeType representation (Exact, LowerBound, UpperBound)
    pub age: u8,            // Aging mechanism to overwrite old entries
}

impl TTEntry {
    pub fn new(key: u64, best_move: Move, score: i16, depth: u8, node_type: NodeType, age: u8) -> Self {
        TTEntry {
            key,
            best_move,
            score,
            depth,
            node_type: node_type as u8,
            age,
        }
    }

    pub fn node_type(&self) -> NodeType {
        match self.node_type {
            0 => NodeType::Exact,
            1 => NodeType::LowerBound,
            _ => NodeType::UpperBound,
        }
    }
}

pub struct TranspositionTable {
    entries: Vec<TTEntry>,
    mask: usize,
}

impl TranspositionTable {
    pub fn new(size_mb: usize) -> Self {
        let size_bytes = size_mb * 1024 * 1024;
        let num_entries = size_bytes / std::mem::size_of::<TTEntry>();
        // Ensure size is a power of 2 for fast masking
        let num_entries = num_entries.next_power_of_two() / 2;
        
        TranspositionTable {
            entries: vec![TTEntry::new(0, Move(0), 0, 0, NodeType::Exact, 0); num_entries],
            mask: num_entries - 1,
        }
    }

    pub fn clear(&mut self) {
        for entry in self.entries.iter_mut() {
            entry.key = 0;
            entry.depth = 0;
            entry.age = 0;
        }
    }

    pub fn store(&mut self, entry: TTEntry) {
        let index = (entry.key as usize) & self.mask;
        let current = &self.entries[index];

        // Replacement scheme:
        // 1. Always replace if the entry is empty (key == 0)
        // 2. Replace if it's from a different position (key != current.key)
        // 3. Replace if the new entry has greater or equal depth
        // 4. (Optional) Age-based replacement
        
        // Simple depth-preferred replacement
        if current.key == 0 || current.key != entry.key || entry.depth >= current.depth {
            self.entries[index] = entry;
        }
    }

    pub fn probe(&self, key: u64) -> Option<TTEntry> {
        let index = (key as usize) & self.mask;
        let entry = self.entries[index];
        if entry.key == key {
            Some(entry)
        } else {
            None
        }
    }
}
