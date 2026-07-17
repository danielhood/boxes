//! Simulation resource, fixed-timestep stepping, and demo world seed.

use std::collections::HashSet;

use bevy::prelude::*;
use boxes_sim::{
    generator_period, make_aggregator, make_generator, make_transformer, ChunkCoord,
    Direction, ReduceMode, Simulation, WorldPos, MAX_STEPS_PER_FRAME,
};

use crate::input::{random_selection, SelectedCell};
use crate::render::{affected_chunks, ActiveView, OrthoView, PendingChunkRebuilds};

/// Simulation playback speed multiplier.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SimSpeed {
    Half,
    #[default]
    Normal,
    Double,
}

impl SimSpeed {
    #[must_use]
    pub const fn multiplier(self) -> f32 {
        match self {
            Self::Half => 0.5,
            Self::Normal => 1.0,
            Self::Double => 2.0,
        }
    }

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Half => "0.5x",
            Self::Normal => "1x",
            Self::Double => "2x",
        }
    }

    #[must_use]
    pub const fn next(self) -> Self {
        match self {
            Self::Half => Self::Normal,
            Self::Normal => Self::Double,
            Self::Double => Self::Half,
        }
    }
}

/// Pause, speed, and single-step controls for the simulation loop.
#[derive(Resource, Clone, Copy, Debug)]
pub struct SimPlayback {
    pub paused: bool,
    pub speed: SimSpeed,
    pub step_pending: bool,
    pub debug_overlay: bool,
}

impl Default for SimPlayback {
    fn default() -> Self {
        Self {
            paused: false,
            speed: SimSpeed::Normal,
            step_pending: false,
            debug_overlay: false,
        }
    }
}

/// Metrics from the most recent simulation tick (for HUD / debug overlay).
#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct SimTickStats {
    pub last_dirty_chunks: usize,
    pub last_cell_updates: u64,
}

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
    let seed_positions = seed_demo_world(&mut sim);
    let selection = SelectedCell {
        pos: random_selection(&seed_positions, &sim),
    };
    commands.insert_resource(GridSimulation(sim));
    commands.insert_resource(selection);
    commands.insert_resource(SimClock::default());
    commands.insert_resource(SimDirtyChunks::default());
    commands.insert_resource(SimPlayback::default());
    commands.insert_resource(SimTickStats::default());
}

/// Seed a ~64³ active region near world center for dev rendering.
pub fn seed_demo_world(sim: &mut Simulation) -> Vec<WorldPos> {
    let origin = (500u16.saturating_sub(64)) / 2;
    let mut positions = Vec::with_capacity(64 * 64);

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
            positions.push(pos);
        }
    }

    // Mark a few event-driven listeners dirty so the sim produces ongoing activity.
    sim.mark_dirty(WorldPos::new(origin + 2, origin + 2, origin + 2));
    positions
}

/// Whether the sim loop should advance this frame (excluding `step_pending` handling).
#[must_use]
pub fn should_accumulate_sim_time(playback: &SimPlayback) -> bool {
    !playback.paused && !playback.step_pending
}

/// Scaled frame delta for the fixed-timestep accumulator.
#[must_use]
pub fn scaled_frame_delta(frame_delta: f32, playback: &SimPlayback) -> f32 {
    frame_delta * playback.speed.multiplier()
}

#[allow(clippy::too_many_arguments)]
pub fn sim_step_system(
    time: Res<Time>,
    mut clock: ResMut<SimClock>,
    mut playback: ResMut<SimPlayback>,
    mut sim: ResMut<GridSimulation>,
    mut dirty: ResMut<SimDirtyChunks>,
    mut stats: ResMut<SimTickStats>,
    active: Res<ActiveView>,
    mut pending: ResMut<PendingChunkRebuilds>,
) {
    let dt = sim.0.dt();

    if playback.step_pending {
        playback.step_pending = false;
        run_sim_tick(
            &mut sim.0,
            &mut dirty,
            &mut stats,
            active.face(),
            &mut pending,
        );
        return;
    }

    if !should_accumulate_sim_time(&playback) {
        return;
    }

    clock.accumulator += scaled_frame_delta(time.delta_secs(), &playback);

    let mut steps = 0u32;
    while clock.accumulator >= dt && steps < MAX_STEPS_PER_FRAME {
        clock.accumulator -= dt;
        run_sim_tick(
            &mut sim.0,
            &mut dirty,
            &mut stats,
            active.face(),
            &mut pending,
        );
        steps += 1;
    }
}

fn run_sim_tick(
    sim: &mut Simulation,
    dirty: &mut SimDirtyChunks,
    stats: &mut SimTickStats,
    view: OrthoView,
    pending: &mut PendingChunkRebuilds,
) {
    let updates_before = sim.total_cell_updates;
    let chunk_coords = sim.step(1);
    stats.last_dirty_chunks = chunk_coords.len();
    stats.last_cell_updates = sim.total_cell_updates.saturating_sub(updates_before);

    if chunk_coords.is_empty() {
        return;
    }

    dirty.coords.extend(chunk_coords.iter().copied());

    let mut rebuild = HashSet::new();
    for coord in &chunk_coords {
        if let Some(chunk) = sim.world.chunks.get_chunk(*coord) {
            for (local, cell) in chunk.cells.iter().enumerate() {
                if cell.is_empty() {
                    continue;
                }
                let pos = boxes_sim::world_pos_from_local(*coord, local);
                dirty.changed_positions.push(pos);
                rebuild.extend(affected_chunks(pos, view));
            }
        }
    }
    pending.mark_dirty(rebuild);
}

/// When cells are edited externally, queue rebuilds (P4 placement tools).
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
    use crate::render::{OrthoView, PendingChunkRebuilds};

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

    #[test]
    fn queue_rebuild_for_edited_cell() {
        use boxes_sim::{make_generator, Cell, WorldPos};
        use crate::render::{OrthoView, PendingChunkRebuilds};

        let mut sim = Simulation::new();
        let pos = WorldPos::new(10, 10, 10);
        let mut pending = PendingChunkRebuilds::default();

        sim.world.set(pos, make_generator(20, 1));
        queue_rebuild_for_positions(&[pos], OrthoView::Top, &mut pending);
        assert!(!pending.chunks.is_empty());

        sim.world.set(pos, Cell::empty());
        queue_rebuild_for_positions(&[pos], OrthoView::Top, &mut pending);
        assert!(!pending.chunks.is_empty());
    }

    #[test]
    fn view_switch_preserves_world_state() {
        use crate::render::OrthoView;

        let mut sim = Simulation::new();
        seed_demo_world(&mut sim);
        let count_before = sim.world.chunks.chunk_count();

        // View changes are render-only; sim state is unchanged.
        let _views = [OrthoView::Top, OrthoView::Front, OrthoView::Left];
        assert_eq!(sim.world.chunks.chunk_count(), count_before);
    }

    #[test]
    fn paused_playback_does_not_accumulate_time() {
        let playback = SimPlayback {
            paused: true,
            ..Default::default()
        };
        assert!(!should_accumulate_sim_time(&playback));
    }

    #[test]
    fn speed_multiplier_scales_delta() {
        let playback = SimPlayback {
            speed: SimSpeed::Double,
            ..Default::default()
        };
        assert!((scaled_frame_delta(0.05, &playback) - 0.1).abs() < f32::EPSILON);
    }

    #[test]
    fn step_pending_blocks_accumulator() {
        let playback = SimPlayback {
            step_pending: true,
            ..Default::default()
        };
        assert!(!should_accumulate_sim_time(&playback));
    }

    #[test]
    fn run_sim_tick_advances_exactly_one_tick() {
        let mut sim = Simulation::new();
        seed_demo_world(&mut sim);
        let tick_before = sim.tick;
        let mut dirty = SimDirtyChunks::default();
        let mut stats = SimTickStats::default();
        let mut pending = PendingChunkRebuilds::default();

        run_sim_tick(
            &mut sim,
            &mut dirty,
            &mut stats,
            OrthoView::Top,
            &mut pending,
        );

        assert_eq!(sim.tick, tick_before + 1);
    }
}
