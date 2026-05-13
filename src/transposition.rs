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
    pub age: u8,            // Search generation for aging
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
    pub num_entries: usize,
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
            num_entries,
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

        // Improved replacement scheme:
        // 1. Always replace empty entries
        // 2. Always replace if same position (we have newer/better info)
        // 3. Replace if new entry is from a newer generation (current is stale)
        // 4. Replace if new entry has greater depth (within same generation)
        // 5. Prefer Exact nodes over bound nodes
        
        let should_replace = current.key == 0
            || current.key == entry.key
            || entry.age != current.age  // Different generation = stale, replace
            || entry.depth + if entry.node_type == NodeType::Exact as u8 { 2 } else { 0 }
               >= current.depth + if current.node_type == NodeType::Exact as u8 { 2 } else { 0 };
        
        if should_replace {
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

    /// Returns hashfull in permill (0-1000) for UCI info
    pub fn hashfull(&self) -> u32 {
        let sample_size = 1000.min(self.num_entries);
        let mut used = 0u32;
        for i in 0..sample_size {
            if self.entries[i].key != 0 {
                used += 1;
            }
        }
        (used * 1000) / sample_size as u32
    }
}
