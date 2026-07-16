//! Dense 32³ cell storage for one chunk.

use crate::cell::Cell;
use crate::constants::CHUNK_VOLUME;

/// Fixed-size dense chunk array. Empty cells use the default sentinel.
#[derive(Clone, Debug)]
pub struct Chunk {
    pub cells: [Cell; CHUNK_VOLUME],
}

impl Default for Chunk {
    fn default() -> Self {
        Self {
            cells: [Cell::empty(); CHUNK_VOLUME],
        }
    }
}

impl Chunk {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.cells.iter().all(|c| c.is_empty())
    }
}
