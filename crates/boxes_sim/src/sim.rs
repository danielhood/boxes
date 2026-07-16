//! Tick scheduler, dirty queues, and P2 behavior hooks.

use std::collections::{HashSet, VecDeque};

use crate::constants::{
    DEFAULT_MAX_CELL_UPDATES_PER_TICK, DEFAULT_MAX_DIRTY_DRAIN_PER_TICK, PHASE_COUNT,
    TICK_RATE_HZ,
};
use crate::coord::{ChunkCoord, WorldPos};
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

/// Hooks for P2 cell-type logic. P1 provides no-op defaults.
pub trait SimHooks {
    /// Process one dirty cell (transformer / aggregator entry point).
    fn on_dirty_cell(&mut self, _sim: &mut Simulation, _pos: WorldPos) {}

    /// Periodic generator scheduling entry point for one phase bucket.
    fn on_phase_tick(&mut self, _sim: &mut Simulation, _tick: u64, _phase: u8) {}
}

/// Default no-op hooks until P2 wires cell behaviors.
#[derive(Clone, Copy, Debug, Default)]
pub struct NullHooks;

impl SimHooks for NullHooks {}

/// Engine-agnostic simulation kernel.
#[derive(Clone, Debug)]
pub struct Simulation {
    pub world: World,
    pub tick: u64,
    pub config: SimConfig,
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

    /// Advance the simulation by `ticks` steps. Returns chunk coords dirtied this call.
    pub fn step(&mut self, ticks: u32) -> Vec<ChunkCoord> {
        self.step_with_hooks(ticks, &mut NullHooks)
    }

    /// Advance with custom P2 hooks.
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

            if updates < max_updates {
                updates += self.propagate_dirty_stub(pos, max_updates - updates);
            }
        }

        // 2. Phase-gated periodic work (generators in P2).
        let tick = self.tick;
        for phase_id in 0..PHASE_COUNT {
            hooks.on_phase_tick(self, tick, phase_id);
        }

        self.tick += 1;
    }

    /// P1 stub: enqueue face neighbors when a dirty cell is processed.
    fn propagate_dirty_stub(&mut self, pos: WorldPos, budget: usize) -> usize {
        let mut propagated = 0usize;
        for neighbor in pos.neighbors_6() {
            if propagated >= budget {
                break;
            }
            if self.world.get(neighbor).is_empty() {
                continue;
            }
            if self.queued_dirty.contains(&neighbor) {
                continue;
            }
            if self.drained_this_tick.contains(&neighbor) {
                continue;
            }
            self.mark_dirty(neighbor);
            propagated += 1;
        }
        propagated
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::cell::Cell;
    use crate::phase;

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
        let _ = sim.step(1);
        assert_eq!(sim.tick, 1);
        let tick_after = sim.tick;
        let _ = sim.step(0);
        assert_eq!(sim.tick, tick_after);
    }

    #[test]
    fn step_invokes_hooks() {
        let mut sim = Simulation::new();
        let pos = WorldPos::new(5, 5, 5);
        sim.world.set_typed(pos, 1, 0);
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
    fn dirty_propagation_stub_enqueues_neighbors() {
        let mut sim = Simulation::new();
        let center = WorldPos::new(10, 10, 10);
        let neighbor = WorldPos::new(11, 10, 10);

        sim.world.set_typed(center, 1, 0);
        sim.world.set_typed(neighbor, 2, 0);
        sim.mark_dirty(center);

        let mut hooks = RecordingHooks {
            dirty_calls: 0,
            phase_calls: 0,
        };
        let _ = sim.step_with_hooks(1, &mut hooks);

        // Center drains first; stub propagates to neighbor, which drains in the same tick.
        assert_eq!(hooks.dirty_calls, 2);
    }

    #[test]
    fn step_returns_dirty_chunks() {
        let mut sim = Simulation::new();
        let pos = WorldPos::new(64, 64, 64);
        sim.world.set(pos, Cell {
            type_id: 1,
            flags: 0,
            state: 1,
            reserved: 0,
        });
        sim.mark_dirty(pos);

        let dirty = sim.step(1);
        assert_eq!(dirty, vec![pos.chunk_coord()]);
    }

    #[test]
    fn benchmark_stub_one_million_writes_and_thousand_ticks() {
        let mut sim = Simulation::new();

        for i in 0..1_000_000u32 {
            let x = (i % 500) as u16;
            let y = ((i / 500) % 500) as u16;
            let z = ((i / 250_000) % 500) as u16;
            sim.world.set_typed(WorldPos::new(x, y, z), 1, (i % 65536) as u16);
        }

        let dirty = sim.step(1000);
        assert_eq!(sim.tick, 1000);
        // Chunks were allocated; stepping may report none without dirty queue input.
        let _ = dirty;
        assert!(sim.world.chunks.chunk_count() > 0);
    }
}
