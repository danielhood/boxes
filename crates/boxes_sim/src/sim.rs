//! Tick scheduler, dirty queues, and P2 behavior hooks.

use std::collections::{HashSet, VecDeque};

use crate::constants::{
    DEFAULT_MAX_CELL_UPDATES_PER_TICK, DEFAULT_MAX_DIRTY_DRAIN_PER_TICK, PHASE_COUNT,
    TICK_RATE_HZ,
};
use crate::coord::{ChunkCoord, WorldPos};
use crate::engine::CellEngine;
use crate::world::World;

/// Per-tick budget and rate configuration.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SimConfig {
    pub tick_rate_hz: f32,
    pub max_dirty_drain_per_tick: usize,
    pub max_cell_updates_per_tick: usize,
}

impl Default for SimConfig {
    fn default() -> Self {
        Self {
            tick_rate_hz: TICK_RATE_HZ,
            max_dirty_drain_per_tick: DEFAULT_MAX_DIRTY_DRAIN_PER_TICK,
            max_cell_updates_per_tick: DEFAULT_MAX_CELL_UPDATES_PER_TICK,
        }
    }
}

impl SimConfig {
    #[must_use]
    pub const fn dt(self) -> f32 {
        1.0 / self.tick_rate_hz
    }
}

/// Hooks for cell-type logic and extensions.
pub trait SimHooks {
    /// Process one dirty cell (transformer / aggregator entry point).
    fn on_dirty_cell(&mut self, _sim: &mut Simulation, _pos: WorldPos) {}

    /// Periodic generator scheduling entry point for one phase bucket.
    fn on_phase_tick(&mut self, _sim: &mut Simulation, _tick: u64, _phase: u8) {}

    /// Called at the start of each simulation tick.
    fn on_tick_start(&mut self, _sim: &mut Simulation) {}
}

/// Default no-op hooks for tests and raw grid stepping.
#[derive(Clone, Copy, Debug, Default)]
pub struct NullHooks;

impl SimHooks for NullHooks {}

/// Engine-agnostic simulation kernel.
#[derive(Clone, Debug)]
pub struct Simulation {
    pub world: World,
    pub tick: u64,
    pub config: SimConfig,
    pub total_cell_updates: u64,
    dirty_queue: VecDeque<WorldPos>,
    queued_dirty: HashSet<WorldPos>,
    drained_this_tick: HashSet<WorldPos>,
    dirty_chunks: HashSet<ChunkCoord>,
}

impl Default for Simulation {
    fn default() -> Self {
        Self::new()
    }
}

impl Simulation {
    #[must_use]
    pub fn new() -> Self {
        Self::with_config(SimConfig::default())
    }

    #[must_use]
    pub fn with_config(config: SimConfig) -> Self {
        Self {
            world: World::new(),
            tick: 0,
            config,
            total_cell_updates: 0,
            dirty_queue: VecDeque::new(),
            queued_dirty: HashSet::new(),
            drained_this_tick: HashSet::new(),
            dirty_chunks: HashSet::new(),
        }
    }

    #[must_use]
    pub fn dt(&self) -> f32 {
        self.config.dt()
    }

    #[must_use]
    pub fn dirty_queue_len(&self) -> usize {
        self.dirty_queue.len()
    }

    #[must_use]
    pub fn is_queued_dirty(&self, pos: WorldPos) -> bool {
        self.queued_dirty.contains(&pos)
    }

    pub(crate) fn record_cell_update(&mut self) {
        self.total_cell_updates += 1;
    }

    pub(crate) fn mark_chunk_dirty(&mut self, coord: ChunkCoord) {
        self.dirty_chunks.insert(coord);
    }

    /// Enqueue a cell for event-driven re-evaluation.
    pub fn mark_dirty(&mut self, pos: WorldPos) {
        if !pos.is_in_bounds() {
            return;
        }
        let _ = self.world.mark_dirty(pos);
        if self.queued_dirty.insert(pos) {
            self.dirty_queue.push_back(pos);
        }
        self.dirty_chunks.insert(pos.chunk_coord());
    }

    /// Advance the simulation with built-in cell-type behaviors.
    pub fn step(&mut self, ticks: u32) -> Vec<ChunkCoord> {
        self.step_cells(ticks)
    }

    /// Advance the simulation with built-in cell-type behaviors.
    pub fn step_cells(&mut self, ticks: u32) -> Vec<ChunkCoord> {
        let mut engine = CellEngine::new();
        self.step_with_hooks(ticks, &mut engine)
    }

    /// Advance with custom hooks (testing or extensions).
    pub fn step_with_hooks<H: SimHooks>(&mut self, ticks: u32, hooks: &mut H) -> Vec<ChunkCoord> {
        let mut all_dirty = HashSet::new();

        for _ in 0..ticks {
            self.tick_one(hooks);
            all_dirty.extend(self.dirty_chunks.drain());
        }

        let mut coords: Vec<ChunkCoord> = all_dirty.into_iter().collect();
        coords.sort_unstable();
        coords
    }

    fn tick_one<H: SimHooks>(&mut self, hooks: &mut H) {
        let mut updates = 0usize;
        let max_updates = self.config.max_cell_updates_per_tick;
        let max_drain = self
            .config
            .max_dirty_drain_per_tick
            .min(max_updates);

        self.drained_this_tick.clear();
        hooks.on_tick_start(self);

        // 1. Drain dirty queue (event-driven cells).
        while updates < max_drain {
            let Some(pos) = self.dirty_queue.pop_front() else {
                break;
            };
            self.queued_dirty.remove(&pos);
            self.drained_this_tick.insert(pos);

            if updates >= max_updates {
                break;
            }

            self.world.clear_dirty_flag(pos);
            hooks.on_dirty_cell(self, pos);
            self.dirty_chunks.insert(pos.chunk_coord());
            updates += 1;
        }

        // 2. Phase-gated periodic work (generators).
        let tick = self.tick;
        for phase_id in 0..PHASE_COUNT {
            hooks.on_phase_tick(self, tick, phase_id);
        }

        self.tick += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cell::Cell;
    use crate::phase;
    use crate::types::{make_transformer, Direction, TYPE_GENERATOR};

    struct RecordingHooks {
        dirty_calls: u32,
        phase_calls: u32,
    }

    impl SimHooks for RecordingHooks {
        fn on_dirty_cell(&mut self, _sim: &mut Simulation, _pos: WorldPos) {
            self.dirty_calls += 1;
        }

        fn on_phase_tick(&mut self, _sim: &mut Simulation, _tick: u64, _phase: u8) {
            self.phase_calls += 1;
        }
    }

    #[test]
    fn phase_matches_formula() {
        assert_eq!(phase(0, 0, 0), 0);
        assert_eq!(phase(1, 1, 1), 3);
        assert_eq!(phase(7, 7, 7), 5);
        assert_eq!(WorldPos::new(10, 20, 30).phase(), phase(10, 20, 30));
    }

    #[test]
    fn step_advances_tick_deterministically() {
        let mut sim = Simulation::new();
        sim.mark_dirty(WorldPos::new(1, 2, 3));
        let _ = sim.step_with_hooks(1, &mut NullHooks);
        assert_eq!(sim.tick, 1);
        let tick_after = sim.tick;
        let _ = sim.step_with_hooks(0, &mut NullHooks);
        assert_eq!(sim.tick, tick_after);
    }

    #[test]
    fn step_invokes_hooks() {
        let mut sim = Simulation::new();
        let pos = WorldPos::new(5, 5, 5);
        sim.world.set_typed(pos, TYPE_GENERATOR, 0);
        sim.mark_dirty(pos);

        let mut hooks = RecordingHooks {
            dirty_calls: 0,
            phase_calls: 0,
        };
        let dirty = sim.step_with_hooks(1, &mut hooks);

        assert_eq!(hooks.dirty_calls, 1);
        assert_eq!(hooks.phase_calls, u32::from(PHASE_COUNT));
        assert!(dirty.contains(&pos.chunk_coord()));
    }

    #[test]
    fn type_aware_dirty_propagation_enqueues_listeners() {
        let mut sim = Simulation::new();
        let center = WorldPos::new(10, 10, 10);
        let listener = WorldPos::new(11, 10, 10);

        sim.world.set_typed(center, TYPE_GENERATOR, 1);
        sim.world
            .set(listener, make_transformer(Direction::PosX, 0));

        crate::engine::CellEngine::propagate_to_listeners(&mut sim, center);
        assert!(sim.is_queued_dirty(listener));

        let mut hooks = RecordingHooks {
            dirty_calls: 0,
            phase_calls: 0,
        };
        let _ = sim.step_with_hooks(1, &mut hooks);
        assert_eq!(hooks.dirty_calls, 1);
    }

    #[test]
    fn step_returns_dirty_chunks() {
        let mut sim = Simulation::new();
        let pos = WorldPos::new(64, 64, 64);
        sim.world.set(pos, Cell {
            type_id: TYPE_GENERATOR,
            flags: 0,
            state: 1,
            reserved: 20,
        });
        sim.mark_dirty(pos);

        let dirty = sim.step_with_hooks(1, &mut NullHooks);
        assert_eq!(dirty, vec![pos.chunk_coord()]);
    }

    #[test]
    fn benchmark_stub_one_million_writes_and_thousand_ticks() {
        let mut sim = Simulation::new();

        for i in 0..1_000_000u32 {
            let x = (i % 500) as u16;
            let y = ((i / 500) % 500) as u16;
            let z = ((i / 250_000) % 500) as u16;
            sim.world.set_typed(WorldPos::new(x, y, z), TYPE_GENERATOR, 0);
        }

        let dirty = sim.step_with_hooks(1000, &mut NullHooks);
        assert_eq!(sim.tick, 1000);
        let _ = dirty;
        assert!(sim.world.chunks.chunk_count() > 0);
    }
}
