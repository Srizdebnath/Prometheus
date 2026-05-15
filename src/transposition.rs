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
    pub key: u64,        // Zobrist hash of the position
    pub best_move: Move, // Best move found in this position
    pub score: i16,      // Evaluation score
    pub depth: u8,       // Search depth at which this entry was created
    pub node_type: u8,   // NodeType representation (Exact, LowerBound, UpperBound)
    pub age: u8,         // Search generation for aging
}

impl TTEntry {
    pub fn new(
        key: u64,
        best_move: Move,
        score: i16,
        depth: u8,
        node_type: NodeType,
        age: u8,
    ) -> Self {
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
    entries: Vec<[TTEntry; 2]>, // Each slot has 2 buckets
    mask: usize,
    pub num_entries: usize,
}

impl TranspositionTable {
    pub fn new(size_mb: usize) -> Self {
        let size_bytes = size_mb * 1024 * 1024;
        let entry_size = std::mem::size_of::<[TTEntry; 2]>();
        let num_entries = (size_bytes / entry_size).next_power_of_two() / 2;
        let empty = TTEntry::new(0, Move(0), 0, 0, NodeType::Exact, 0);
        TranspositionTable {
            entries: vec![[empty; 2]; num_entries],
            mask: num_entries - 1,
            num_entries,
        }
    }

    pub fn clear(&mut self) {
        for slot in self.entries.iter_mut() {
            for entry in slot.iter_mut() {
                entry.key = 0;
                entry.depth = 0;
                entry.age = 0;
            }
        }
    }

    pub fn store(&mut self, entry: TTEntry) {
        let index = (entry.key as usize) & self.mask;
        let slot = &mut self.entries[index];

        // Bucket 0: depth-preferred
        // Bucket 1: always-replace

        // Check if we're updating the same position in either bucket
        if slot[0].key == entry.key {
            // Same position: update if new entry has equal/higher depth or is exact
            if entry.depth >= slot[0].depth
                || entry.node_type == NodeType::Exact as u8
                || entry.age != slot[0].age
            {
                slot[0] = entry;
            }
            return;
        }
        if slot[1].key == entry.key {
            slot[1] = entry; // Always update the always-replace bucket for same position
            return;
        }

        // New position: decide which bucket to use
        // Depth-preferred bucket (0): replace only if new entry is from newer generation or has more depth
        let replace_depth_bucket =
            slot[0].key == 0 || entry.age != slot[0].age || entry.depth >= slot[0].depth;

        if replace_depth_bucket {
            slot[0] = entry;
        } else {
            // Always-replace bucket (1): always overwrite
            slot[1] = entry;
        }
    }

    pub fn probe(&self, key: u64) -> Option<TTEntry> {
        let index = (key as usize) & self.mask;
        let slot = &self.entries[index];
        if slot[0].key == key {
            return Some(slot[0]);
        }
        if slot[1].key == key {
            return Some(slot[1]);
        }
        None
    }

    /// Returns hashfull in permill (0-1000) for UCI info
    pub fn hashfull(&self) -> u32 {
        let sample_size = 500.min(self.num_entries);
        let mut used = 0u32;
        for i in 0..sample_size {
            // Count both buckets; total samples = sample_size * 2
            if self.entries[i][0].key != 0 {
                used += 1;
            }
            if self.entries[i][1].key != 0 {
                used += 1;
            }
        }
        (used * 1000) / (sample_size as u32 * 2)
    }
}
