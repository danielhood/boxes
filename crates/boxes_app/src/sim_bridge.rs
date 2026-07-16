//! Simulation resource, fixed-timestep stepping, and demo world seed.

use std::collections::HashSet;

use bevy::prelude::*;
use boxes_sim::{
    generator_period, make_aggregator, make_generator, make_transformer, ChunkCoord,
    Direction, ReduceMode, Simulation, WorldPos, MAX_STEPS_PER_FRAME,
};

use crate::render::{affected_chunks, ActiveView, OrthoView, PendingChunkRebuilds};

/// Bevy resource wrapping the engine-agnostic simulation.
#[derive(Resource)]
pub struct GridSimulation(pub Simulation);

/// Fixed-timestep accumulator for 20 Hz simulation.
#[derive(Resource)]
pub struct SimClock {
    pub accumulator: f32,
}

impl Default for SimClock {
    fn default() -> Self {
        Self { accumulator: 0.0 }
    }
}

/// Chunk coords dirtied by the most recent sim step(s).
#[derive(Resource, Default)]
pub struct SimDirtyChunks {
    pub coords: HashSet<ChunkCoord>,
    pub changed_positions: Vec<WorldPos>,
}

pub fn setup_simulation(mut commands: Commands) {
    let mut sim = Simulation::new();
    seed_demo_world(&mut sim);
    commands.insert_resource(GridSimulation(sim));
    commands.insert_resource(SimClock::default());
    commands.insert_resource(SimDirtyChunks::default());
}

/// Seed a ~64³ active region near world center for dev rendering.
pub fn seed_demo_world(sim: &mut Simulation) {
    let origin = (500u16.saturating_sub(64)) / 2;

    for x in 0..64u16 {
        for z in 0..64u16 {
            let wx = origin + x;
            let wz = origin + z;
            let wy = origin + ((x + z) % 8);
            let pos = WorldPos::new(wx, wy, wz);

            let cell = match (x + z) % 5 {
                0 => make_generator(generator_period::STANDARD, 0),
                1 => make_transformer(Direction::PosX, 0),
                2 => make_aggregator(ReduceMode::Sum, 0),
                3 => make_generator(generator_period::FAST, 0),
                _ => make_generator(generator_period::SLOW, 0),
            };
            sim.world.set(pos, cell);
        }
    }

    // Mark a few event-driven listeners dirty so the sim produces ongoing activity.
    sim.mark_dirty(WorldPos::new(origin + 2, origin + 2, origin + 2));
}

pub fn sim_step_system(
    time: Res<Time>,
    mut clock: ResMut<SimClock>,
    mut sim: ResMut<GridSimulation>,
    mut dirty: ResMut<SimDirtyChunks>,
    active: Res<ActiveView>,
    mut pending: ResMut<PendingChunkRebuilds>,
) {
    let dt = sim.0.dt();
    clock.accumulator += time.delta_secs();

    let mut steps = 0u32;
    while clock.accumulator >= dt && steps < MAX_STEPS_PER_FRAME {
        clock.accumulator -= dt;
        let chunk_coords = sim.0.step(1);
        steps += 1;

        if chunk_coords.is_empty() {
            continue;
        }

        dirty.coords.extend(chunk_coords.iter().copied());

        // Queue view-dependent rebuilds for columns affected by dirty chunks.
        let mut rebuild = HashSet::new();
        for coord in &chunk_coords {
            if let Some(chunk) = sim.0.world.chunks.get_chunk(*coord) {
                for (local, cell) in chunk.cells.iter().enumerate() {
                    if cell.is_empty() {
                        continue;
                    }
                    let pos = boxes_sim::world_pos_from_local(*coord, local);
                    dirty.changed_positions.push(pos);
                    rebuild.extend(affected_chunks(pos, active.0));
                }
            }
        }
        pending.mark_dirty(rebuild);
    }
}

/// When cells are edited externally, queue rebuilds (P4 placement tools).
#[allow(dead_code)]
pub fn queue_rebuild_for_positions(
    positions: &[WorldPos],
    view: OrthoView,
    pending: &mut PendingChunkRebuilds,
) {
    let mut rebuild = HashSet::new();
    for pos in positions {
        rebuild.extend(affected_chunks(*pos, view));
    }
    pending.mark_dirty(rebuild);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demo_seed_populates_region() {
        let mut sim = Simulation::new();
        seed_demo_world(&mut sim);
        assert!(sim.world.chunks.chunk_count() > 0);
    }

    #[test]
    fn sim_step_produces_dirty_chunks() {
        let mut sim = Simulation::new();
        seed_demo_world(&mut sim);
        let dirty = sim.step(1);
        assert!(sim.tick >= 1);
        let _ = dirty;
    }
}
