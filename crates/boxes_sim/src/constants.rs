//! Simulation parameters from planning and P1 spec.

/// Addressable world extent per axis (cells).
pub const WORLD_SIZE: u32 = 500;

/// Cells along each chunk axis.
pub const CHUNK_SIZE: u16 = 32;

pub const CHUNK_SIZE_USIZE: usize = CHUNK_SIZE as usize;

/// `CHUNK_SIZE³`.
pub const CHUNK_VOLUME: usize = CHUNK_SIZE_USIZE * CHUNK_SIZE_USIZE * CHUNK_SIZE_USIZE;

/// Chunks per axis covering the world (ceil(500 / 32) = 16).
pub const CHUNKS_PER_AXIS: u16 = (WORLD_SIZE as u16).div_ceil(CHUNK_SIZE);

/// Fixed simulation rate (Hz).
pub const TICK_RATE_HZ: f32 = 20.0;

/// Seconds per simulation tick (`1 / 20`).
pub const DT: f32 = 1.0 / TICK_RATE_HZ;

/// Stagger phase count for periodic work.
pub const PHASE_COUNT: u8 = 8;

/// Initial per-frame sim step cap (render side; documented for P3).
pub const MAX_STEPS_PER_FRAME: u32 = 2;

/// Default cap on dirty cells processed per tick.
pub const DEFAULT_MAX_DIRTY_DRAIN_PER_TICK: usize = 125_000;

/// Default cap on cell updates per tick.
pub const DEFAULT_MAX_CELL_UPDATES_PER_TICK: usize = 250_000;
