//! Simulation kernel for Boxes.
//!
//! This crate owns the grid model, cell types, and tick scheduler. It has no
//! Bevy dependency; `boxes_app` reads dirty chunks and drives rendering.

mod cell;
mod chunk;
mod constants;
mod coord;
mod engine;
mod sim;
mod types;
mod world;

pub use cell::{Cell, CellFlags};
pub use chunk::Chunk;
pub use constants::{
    CHUNK_SIZE, CHUNK_VOLUME, CHUNKS_PER_AXIS, DT, MAX_STEPS_PER_FRAME, PHASE_COUNT,
    TICK_RATE_HZ, WORLD_SIZE,
};
pub use coord::{cell_at, phase, world_pos_from_local, ChunkCoord, WorldPos};
pub use engine::CellEngine;
pub use sim::{NullHooks, SimConfig, SimHooks, Simulation};
pub use types::{
    generator_period, make_aggregator, make_generator, make_transformer, CellTypeInfo,
    CellTypeRegistry, Direction, ReduceMode, TYPE_AGGREGATOR, TYPE_EMPTY, TYPE_GENERATOR,
    TYPE_TRANSFORMER,
};
pub use world::{ChunkMap, World};
