//! Simulation kernel for Boxes.
//!
//! This crate owns the grid model, cell types, and tick scheduler. It has no
//! Bevy dependency; `boxes_app` reads dirty chunks and drives rendering.

mod cell;
mod chunk;
mod constants;
mod coord;
mod sim;
mod world;

pub use cell::{Cell, CellFlags, TYPE_EMPTY};
pub use chunk::Chunk;
pub use constants::{
    CHUNK_SIZE, CHUNK_VOLUME, CHUNKS_PER_AXIS, DT, MAX_STEPS_PER_FRAME, PHASE_COUNT,
    TICK_RATE_HZ, WORLD_SIZE,
};
pub use coord::{cell_at, phase, world_pos_from_local, ChunkCoord, WorldPos};
pub use sim::{NullHooks, SimConfig, SimHooks, Simulation};
pub use world::{ChunkMap, World};
