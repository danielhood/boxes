//! Cell-type behaviors wired into the P1 tick scheduler.

use crate::cell::Cell;
use crate::coord::WorldPos;
use crate::sim::{SimHooks, Simulation};
use crate::types::{
    aggregator_mode, generator_period, generator_should_fire, transformer_direction,
    CellTypeRegistry, Direction, ReduceMode, TYPE_AGGREGATOR, TYPE_GENERATOR, TYPE_TRANSFORMER,
};

/// Executes generator, transformer, and aggregator rules.
///
/// **Read/write policy:** neighbor values are read at the start of evaluation;
/// writes happen once at the end. Dirty propagation runs only when `state` or
/// `type_id` actually changes, preventing stable-configuration feedback loops.
#[derive(Clone, Copy, Debug, Default)]
pub struct CellEngine;

impl CellEngine {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Write a cell and propagate dirties to event-driven neighbors when changed.
    fn write_cell(sim: &mut Simulation, pos: WorldPos, cell: Cell) -> bool {
        let before = sim.world.get(pos);
        if before == cell {
            return false;
        }

        sim.world.set(pos, cell);
        sim.mark_chunk_dirty(pos.chunk_coord());

        let after = sim.world.get(pos);
        if before.state != after.state || before.type_id != after.type_id {
            Self::propagate_to_listeners(sim, pos);
            return true;
        }
        false
    }

    /// Mark transformer / aggregator neighbors dirty after a local state change.
    pub(crate) fn propagate_to_listeners(sim: &mut Simulation, pos: WorldPos) {
        for neighbor in pos.neighbors_6() {
            let cell = sim.world.get(neighbor);
            if CellTypeRegistry::listens_to_neighbors(cell.type_id) {
                sim.mark_dirty(neighbor);
            }
        }
    }

    fn neighbor_in_direction(pos: WorldPos, direction: Direction) -> Option<WorldPos> {
        let candidate = match direction {
            Direction::PosX => WorldPos::new(pos.x.saturating_add(1), pos.y, pos.z),
            Direction::NegX => WorldPos::new(pos.x.saturating_sub(1), pos.y, pos.z),
            Direction::PosY => WorldPos::new(pos.x, pos.y.saturating_add(1), pos.z),
            Direction::NegY => WorldPos::new(pos.x, pos.y.saturating_sub(1), pos.z),
            Direction::PosZ => WorldPos::new(pos.x, pos.y, pos.z.saturating_add(1)),
            Direction::NegZ => WorldPos::new(pos.x, pos.y, pos.z.saturating_sub(1)),
        };
        candidate.is_in_bounds().then_some(candidate)
    }

    fn process_transformer(&mut self, sim: &mut Simulation, pos: WorldPos, cell: Cell) {
        let direction = transformer_direction(cell);
        let input_state = Self::neighbor_in_direction(pos, direction)
            .map(|neighbor| sim.world.get(neighbor).state)
            .unwrap_or(0);

        if cell.state == input_state {
            return;
        }

        let mut next = cell;
        next.state = input_state;
        if Self::write_cell(sim, pos, next) {
            sim.record_cell_update();
        }
    }

    fn process_aggregator(&mut self, sim: &mut Simulation, pos: WorldPos, cell: Cell) {
        let mode = aggregator_mode(cell);
        let mut sum = 0u32;
        let mut max = 0u16;

        for neighbor in pos.neighbors_6() {
            let value = sim.world.get(neighbor).state;
            sum = sum.saturating_add(u32::from(value));
            max = max.max(value);
        }

        let next_state = match mode {
            ReduceMode::Sum => sum.min(u32::from(u16::MAX)) as u16,
            ReduceMode::Max => max,
        };

        if cell.state == next_state {
            return;
        }

        let mut next = cell;
        next.state = next_state;
        if Self::write_cell(sim, pos, next) {
            sim.record_cell_update();
        }
    }

    fn fire_generator(&mut self, sim: &mut Simulation, pos: WorldPos, cell: Cell) {
        // Pulse counter in state so tests can observe fires without extra fields.
        let mut next = cell;
        next.state = next.state.saturating_add(1);
        if Self::write_cell(sim, pos, next) {
            sim.record_cell_update();
        }
    }
}

impl SimHooks for CellEngine {
    fn on_dirty_cell(&mut self, sim: &mut Simulation, pos: WorldPos) {
        let cell = sim.world.get(pos);
        match cell.type_id {
            TYPE_TRANSFORMER => self.process_transformer(sim, pos, cell),
            TYPE_AGGREGATOR => self.process_aggregator(sim, pos, cell),
            _ => {}
        }
    }

    fn on_phase_tick(&mut self, sim: &mut Simulation, tick: u64, phase_id: u8) {
        let generators: Vec<(WorldPos, Cell)> = sim
            .world
            .chunks
            .iter_non_empty()
            .filter(|(pos, cell)| cell.type_id == TYPE_GENERATOR && pos.phase() == phase_id)
            .collect();

        for (pos, cell) in generators {
            let period = generator_period(cell);
            if generator_should_fire(tick, pos, period) {
                self.fire_generator(sim, pos, cell);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::generator_period::{FAST, STANDARD};
    use crate::{
        make_aggregator, make_generator, make_transformer, Direction, ReduceMode, Simulation,
        TYPE_GENERATOR, PHASE_COUNT,
    };

    const ORIGIN: WorldPos = WorldPos::new(50, 50, 50);

    fn pos_with_phase(target: u8) -> WorldPos {
        for x in 0..16u16 {
            for y in 0..8u16 {
                let z = (target as u16 + PHASE_COUNT as u16 - ((x + y) % PHASE_COUNT as u16))
                    % PHASE_COUNT as u16;
                let pos = WorldPos::new(x, y, z);
                if pos.phase() == target {
                    return pos;
                }
            }
        }
        panic!("no position for phase {target}");
    }

    #[test]
    fn generator_fires_on_correct_phased_ticks() {
        for phase_id in 0..PHASE_COUNT {
            let pos = pos_with_phase(phase_id);
            let period = 10u32;
            let mut sim = Simulation::new();
            sim.world.set(pos, make_generator(period, 0));

            let mut fires = 0u16;
            for tick in 0..period {
                sim.tick = u64::from(tick);
                let mut engine = CellEngine::new();
                engine.on_phase_tick(&mut sim, u64::from(tick), phase_id);
                let state = sim.world.get(pos).state;
                fires = fires.max(state);
            }

            assert_eq!(fires, 1, "phase {phase_id} should fire exactly once per period");
        }
    }

    #[test]
    fn generator_stagger_spreads_load() {
        let period = STANDARD;
        let mut sim = Simulation::new();
        for phase_id in 0..PHASE_COUNT {
            let pos = pos_with_phase(phase_id);
            sim.world.set(pos, make_generator(period, 0));
        }

        let mut engine = CellEngine::new();
        engine.on_phase_tick(&mut sim, 0, 0);
        let fired_at_t0 = sim
            .world
            .chunks
            .iter_non_empty()
            .filter(|(_, cell)| cell.type_id == TYPE_GENERATOR && cell.state > 0)
            .count();
        assert_eq!(fired_at_t0, 1, "only phase-0 bucket fires on tick 0");
    }

    #[test]
    fn transformer_copies_input_neighbor_after_source_changes() {
        let mut sim = Simulation::new();
        let source = WorldPos::new(ORIGIN.x + 1, ORIGIN.y, ORIGIN.z);
        let sink = ORIGIN;

        sim.world.set(source, make_generator(FAST, 10));
        sim.world.set(sink, make_transformer(Direction::PosX, 0));
        sim.mark_dirty(sink);
        let _ = sim.step_cells(1);
        assert_eq!(sim.world.get(sink).state, 10);

        sim.world.set_typed(source, TYPE_GENERATOR, 42);
        CellEngine::propagate_to_listeners(&mut sim, source);
        let _ = sim.step_cells(1);
        assert_eq!(sim.world.get(sink).state, 42);
    }

    #[test]
    fn transformer_stable_when_input_unchanged() {
        let mut sim = Simulation::new();
        let source = WorldPos::new(ORIGIN.x + 1, ORIGIN.y, ORIGIN.z);
        let sink = ORIGIN;

        sim.world.set(source, make_generator(10_000, 5));
        sim.world.set(sink, make_transformer(Direction::PosX, 0));
        CellEngine::propagate_to_listeners(&mut sim, source);
        let _ = sim.step_cells(1);
        assert_eq!(sim.world.get(sink).state, 5);

        let updates_before = sim.total_cell_updates;
        let _ = sim.step_cells(5);
        assert_eq!(sim.world.get(sink).state, 5);
        assert_eq!(sim.total_cell_updates, updates_before);
    }

    #[test]
    fn aggregator_sums_neighbors() {
        let mut sim = Simulation::new();
        let center = ORIGIN;
        sim.world.set(center, make_aggregator(ReduceMode::Sum, 0));

        let neighbors = [
            WorldPos::new(center.x + 1, center.y, center.z),
            WorldPos::new(center.x - 1, center.y, center.z),
        ];
        for (idx, pos) in neighbors.iter().enumerate() {
            sim.world
                .set(*pos, make_generator(10_000, (idx as u16 + 1) * 10));
        }

        CellEngine::propagate_to_listeners(&mut sim, neighbors[0]);
        let _ = sim.step_cells(1);
        assert_eq!(sim.world.get(center).state, 10 + 20);
    }

    #[test]
    fn aggregator_max_mode() {
        let mut sim = Simulation::new();
        let center = ORIGIN;
        sim.world.set(center, make_aggregator(ReduceMode::Max, 0));

        sim.world.set(
            WorldPos::new(center.x + 1, center.y, center.z),
            make_generator(10_000, 3),
        );
        sim.world.set(
            WorldPos::new(center.x - 1, center.y, center.z),
            make_generator(10_000, 9),
        );

        CellEngine::propagate_to_listeners(
            &mut sim,
            WorldPos::new(center.x + 1, center.y, center.z),
        );
        let _ = sim.step_cells(1);
        assert_eq!(sim.world.get(center).state, 9);
    }

    #[test]
    fn no_infinite_dirty_loop_on_stable_configuration() {
        let mut sim = Simulation::new();
        let source = WorldPos::new(ORIGIN.x + 1, ORIGIN.y, ORIGIN.z);
        let transformer = ORIGIN;

        sim.world.set(source, make_generator(10_000, 4));
        sim.world
            .set(transformer, make_transformer(Direction::PosX, 0));

        CellEngine::propagate_to_listeners(&mut sim, source);
        let _ = sim.step_cells(1);
        assert_eq!(sim.world.get(transformer).state, 4);

        let queue_len = sim.dirty_queue_len();
        let _ = sim.step_cells(50);
        assert_eq!(sim.dirty_queue_len(), queue_len);
        assert_eq!(sim.world.get(transformer).state, 4);
    }

    #[test]
    fn updates_scale_with_activity_not_population() {
        let mut sim = Simulation::new();
        for i in 0..500u16 {
            let pos = WorldPos::new(ORIGIN.x + i % 50, ORIGIN.y + i / 50, ORIGIN.z);
            sim.world.set(pos, make_transformer(Direction::PosX, 0));
        }

        sim.total_cell_updates = 0;
        let _ = sim.step_cells(20);
        assert_eq!(sim.total_cell_updates, 0);

        sim.world.set(
            WorldPos::new(ORIGIN.x, ORIGIN.y + 1, ORIGIN.z),
            make_generator(FAST, 0),
        );
        let before = sim.total_cell_updates;
        let _ = sim.step_cells(20);
        let active = sim.total_cell_updates - before;
        assert!(active > 0);
        assert!(active <= 4);
    }

    #[test]
    fn transformer_chain_in_3x3_fixture() {
        let mut sim = Simulation::new();
        let base = WorldPos::new(10, 10, 10);
        let mid = WorldPos::new(11, 10, 10);
        let end = WorldPos::new(12, 10, 10);

        sim.world.set(base, make_generator(10_000, 0));
        sim.world.set(mid, make_transformer(Direction::NegX, 0));
        sim.world.set(end, make_transformer(Direction::NegX, 0));

        sim.world.set_typed(base, TYPE_GENERATOR, 1);
        CellEngine::propagate_to_listeners(&mut sim, base);
        let _ = sim.step_cells(1);
        assert_eq!(sim.world.get(base).state, 1);
        assert_eq!(sim.world.get(mid).state, 1);
        // Dirty drain can walk the chain within a single tick when budgets allow.
        assert_eq!(sim.world.get(end).state, 1);

        let updates_before = sim.total_cell_updates;
        let _ = sim.step_cells(10);
        assert_eq!(sim.world.get(end).state, 1);
        assert_eq!(sim.total_cell_updates, updates_before);
    }
}
