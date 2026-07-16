//! World and chunk coordinate types.

use crate::cell::Cell;
use crate::chunk::Chunk;
use crate::constants::{CHUNK_SIZE, CHUNK_SIZE_USIZE, PHASE_COUNT, WORLD_SIZE};

/// Addressable world cell position, 0..499 per axis.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WorldPos {
    pub x: u16,
    pub y: u16,
    pub z: u16,
}

impl WorldPos {
    #[must_use]
    pub const fn new(x: u16, y: u16, z: u16) -> Self {
        Self { x, y, z }
    }

    #[must_use]
    pub fn is_in_bounds(self) -> bool {
        let max = WORLD_SIZE as u16;
        self.x < max && self.y < max && self.z < max
    }

    /// Stagger phase bucket: `(x + y + z) % 8`.
    #[must_use]
    pub fn phase(self) -> u8 {
        phase(self.x, self.y, self.z)
    }

    #[must_use]
    pub fn chunk_coord(self) -> ChunkCoord {
        ChunkCoord::from_world(self)
    }

    #[must_use]
    pub fn local_index(self) -> usize {
        local_index(self.x, self.y, self.z)
    }

    /// Six axis-aligned neighbors within world bounds.
    pub fn neighbors_6(self) -> impl Iterator<Item = Self> {
        const DELTAS: [(i32, i32, i32); 6] = [
            (1, 0, 0),
            (-1, 0, 0),
            (0, 1, 0),
            (0, -1, 0),
            (0, 0, 1),
            (0, 0, -1),
        ];

        DELTAS.into_iter().filter_map(move |(dx, dy, dz)| {
            let x = i32::from(self.x) + dx;
            let y = i32::from(self.y) + dy;
            let z = i32::from(self.z) + dz;
            if x < 0 || y < 0 || z < 0 {
                return None;
            }
            let pos = Self::new(x as u16, y as u16, z as u16);
            pos.is_in_bounds().then_some(pos)
        })
    }
}

/// Chunk address in the sparse chunk map.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ChunkCoord {
    pub x: u16,
    pub y: u16,
    pub z: u16,
}

impl ChunkCoord {
    #[must_use]
    pub const fn new(x: u16, y: u16, z: u16) -> Self {
        Self { x, y, z }
    }

    #[must_use]
    pub fn from_world(pos: WorldPos) -> Self {
        Self {
            x: pos.x / CHUNK_SIZE,
            y: pos.y / CHUNK_SIZE,
            z: pos.z / CHUNK_SIZE,
        }
    }

    #[must_use]
    pub fn origin(self) -> WorldPos {
        WorldPos::new(
            self.x * CHUNK_SIZE,
            self.y * CHUNK_SIZE,
            self.z * CHUNK_SIZE,
        )
    }
}

/// Stagger phase for a world position.
#[must_use]
pub fn phase(x: u16, y: u16, z: u16) -> u8 {
    ((u32::from(x) + u32::from(y) + u32::from(z)) % u32::from(PHASE_COUNT)) as u8
}

#[must_use]
pub fn local_index(x: u16, y: u16, z: u16) -> usize {
    let lx = (x % CHUNK_SIZE) as usize;
    let ly = (y % CHUNK_SIZE) as usize;
    let lz = (z % CHUNK_SIZE) as usize;
    lx + ly * CHUNK_SIZE_USIZE + lz * CHUNK_SIZE_USIZE * CHUNK_SIZE_USIZE
}

#[must_use]
pub fn world_pos_from_local(chunk: ChunkCoord, local: usize) -> WorldPos {
    let origin = chunk.origin();
    let lx = local % CHUNK_SIZE_USIZE;
    let ly = (local / CHUNK_SIZE_USIZE) % CHUNK_SIZE_USIZE;
    let lz = local / (CHUNK_SIZE_USIZE * CHUNK_SIZE_USIZE);
    WorldPos::new(
        origin.x + lx as u16,
        origin.y + ly as u16,
        origin.z + lz as u16,
    )
}

/// Read a cell from a chunk slice at a world position within that chunk.
#[must_use]
pub fn cell_at(chunk: &Chunk, pos: WorldPos) -> Cell {
    chunk.cells[local_index(pos.x, pos.y, pos.z)]
}
