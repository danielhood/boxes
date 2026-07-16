//! Sparse chunked world storage.

use std::collections::HashMap;

use crate::cell::{Cell, CellFlags, TYPE_EMPTY};
use crate::chunk::Chunk;
use crate::constants::WORLD_SIZE;
use crate::coord::{cell_at, ChunkCoord, WorldPos};

/// Sparse map of allocated chunks. Unallocated regions are implicitly empty.
#[derive(Clone, Debug, Default)]
pub struct ChunkMap {
    chunks: HashMap<ChunkCoord, Chunk>,
}

impl ChunkMap {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn chunk_count(&self) -> usize {
        self.chunks.len()
    }

    #[must_use]
    pub fn contains_chunk(&self, coord: ChunkCoord) -> bool {
        self.chunks.contains_key(&coord)
    }

    #[must_use]
    pub fn get_chunk(&self, coord: ChunkCoord) -> Option<&Chunk> {
        self.chunks.get(&coord)
    }

    #[must_use]
    pub fn get_chunk_mut(&mut self, coord: ChunkCoord) -> Option<&mut Chunk> {
        self.chunks.get_mut(&coord)
    }

    /// Read a cell. Missing chunks and out-of-bounds positions return empty.
    #[must_use]
    pub fn get(&self, pos: WorldPos) -> Cell {
        if !pos.is_in_bounds() {
            return Cell::empty();
        }
        self.chunks
            .get(&pos.chunk_coord())
            .map(|chunk| cell_at(chunk, pos))
            .unwrap_or_else(Cell::empty)
    }

    /// Write a cell. Allocates a chunk on first non-empty write; removes empty chunks.
    pub fn set(&mut self, pos: WorldPos, cell: Cell) -> bool {
        if !pos.is_in_bounds() {
            return false;
        }

        let coord = pos.chunk_coord();
        let index = pos.local_index();

        if cell.is_empty() {
            if let Some(chunk) = self.chunks.get_mut(&coord) {
                chunk.cells[index] = Cell::empty();
                if chunk.is_empty() {
                    self.chunks.remove(&coord);
                }
            }
            return false;
        }

        let chunk = self.chunks.entry(coord).or_default();
        let changed = chunk.cells[index] != cell;
        chunk.cells[index] = cell;
        changed
    }

    pub fn iter_chunks(&self) -> impl Iterator<Item = (&ChunkCoord, &Chunk)> {
        self.chunks.iter()
    }

    pub fn iter_chunks_mut(&mut self) -> impl Iterator<Item = (&ChunkCoord, &mut Chunk)> {
        self.chunks.iter_mut()
    }

    /// Iterate non-empty cells with world positions.
    pub fn iter_non_empty(&self) -> impl Iterator<Item = (WorldPos, Cell)> + '_ {
        self.chunks.iter().flat_map(|(coord, chunk)| {
            chunk.cells.iter().enumerate().filter_map(move |(local, cell)| {
                if cell.is_empty() {
                    None
                } else {
                    Some((
                        crate::coord::world_pos_from_local(*coord, local),
                        *cell,
                    ))
                }
            })
        })
    }
}

/// World-level grid accessor combining bounds checks and chunk routing.
#[derive(Clone, Debug, Default)]
pub struct World {
    pub chunks: ChunkMap,
}

impl World {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn in_bounds(x: u16, y: u16, z: u16) -> bool {
        x < WORLD_SIZE as u16 && y < WORLD_SIZE as u16 && z < WORLD_SIZE as u16
    }

    #[must_use]
    pub fn get(&self, pos: WorldPos) -> Cell {
        self.chunks.get(pos)
    }

    pub fn set(&mut self, pos: WorldPos, cell: Cell) -> bool {
        self.chunks.set(pos, cell)
    }

    /// Set cell type and state, preserving other fields unless resetting to empty.
    pub fn set_typed(&mut self, pos: WorldPos, type_id: u8, state: u16) -> bool {
        if type_id == TYPE_EMPTY {
            return self.set(pos, Cell::empty());
        }
        let mut cell = self.get(pos);
        cell.type_id = type_id;
        cell.state = state;
        self.set(pos, cell)
    }

    pub fn mark_dirty(&mut self, pos: WorldPos) -> bool {
        if !pos.is_in_bounds() {
            return false;
        }
        let mut cell = self.get(pos);
        if cell.is_empty() {
            return false;
        }
        cell.set_flags(CellFlags(cell.flags | CellFlags::DIRTY));
        self.set(pos, cell)
    }

    pub fn clear_dirty_flag(&mut self, pos: WorldPos) {
        if !pos.is_in_bounds() {
            return;
        }
        let coord = pos.chunk_coord();
        let Some(chunk) = self.chunks.get_chunk_mut(coord) else {
            return;
        };
        let cell = &mut chunk.cells[pos.local_index()];
        cell.set_flags(CellFlags(cell.flags & !CellFlags::DIRTY));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sparse_allocation() {
        let mut world = World::new();
        assert_eq!(world.chunks.chunk_count(), 0);

        let pos = WorldPos::new(100, 200, 300);
        assert!(world.get(pos).is_empty());

        world.set_typed(pos, 1, 42);
        assert_eq!(world.chunks.chunk_count(), 1);
        assert_eq!(world.get(pos).type_id, 1);
        assert_eq!(world.get(pos).state, 42);

        world.set(pos, Cell::empty());
        assert_eq!(world.chunks.chunk_count(), 0);
    }

    #[test]
    fn chunk_boundary_neighbors_readable() {
        let mut world = World::new();
        // Chunk boundary at x = 31 / 32.
        let a = WorldPos::new(31, 0, 0);
        let b = WorldPos::new(32, 0, 0);
        world.set_typed(a, 1, 1);
        world.set_typed(b, 2, 2);

        assert_eq!(world.get(a).type_id, 1);
        assert_eq!(world.get(b).type_id, 2);
        assert_ne!(a.chunk_coord(), b.chunk_coord());

        let chunk_a = world.chunks.get_chunk(a.chunk_coord()).unwrap();
        assert_eq!(crate::cell_at(chunk_a, a).type_id, 1);
    }

    #[test]
    fn local_index_round_trip() {
        use crate::coord::local_index;

        let pos = WorldPos::new(33, 65, 97);
        let chunk = pos.chunk_coord();
        let local = local_index(pos.x, pos.y, pos.z);
        let rebuilt = crate::coord::world_pos_from_local(chunk, local);
        assert_eq!(pos, rebuilt);
    }
}
