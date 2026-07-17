//! Persistent selected cell — navigation focal point.

use bevy::prelude::*;
use boxes_sim::{Simulation, WorldPos, WORLD_SIZE};

use crate::render::OrthoView;

/// World center fallback when no seeded cells exist.
pub const FALLBACK_SELECTION: WorldPos = WorldPos::new(250, 250, 250);

/// Always-on UI selection anchor.
#[derive(Resource, Clone, Copy, Debug)]
pub struct SelectedCell {
    pub pos: WorldPos,
}

impl Default for SelectedCell {
    fn default() -> Self {
        Self {
            pos: FALLBACK_SELECTION,
        }
    }
}

#[must_use]
pub fn clamp_pos(pos: WorldPos) -> WorldPos {
    let max = WORLD_SIZE as u16 - 1;
    WorldPos::new(
        pos.x.min(max),
        pos.y.min(max),
        pos.z.min(max),
    )
}

/// Update selection, clamping to grid bounds.
pub fn set_selection(cell: &mut SelectedCell, pos: WorldPos) {
    cell.pos = clamp_pos(pos);
}

#[must_use]
pub fn slice_depth(view: OrthoView, selection: &SelectedCell) -> u16 {
    view.slice_depth(selection.pos)
}

/// Pick a random non-empty cell from `candidates`, or world center.
#[must_use]
pub fn random_selection(candidates: &[WorldPos], sim: &Simulation) -> WorldPos {
    let nonempty: Vec<WorldPos> = candidates
        .iter()
        .copied()
        .filter(|pos| !sim.world.get(*pos).is_empty())
        .collect();

    if nonempty.is_empty() {
        return FALLBACK_SELECTION;
    }

    // Deterministic-ish pick from sim tick 0 without adding a rand dependency.
    let idx = nonempty
        .iter()
        .map(|p| u32::from(p.x) + u32::from(p.y) * 17 + u32::from(p.z) * 289)
        .sum::<u32>() as usize
        % nonempty.len();
    nonempty[idx]
}

#[cfg(test)]
mod tests {
    use super::*;
    use boxes_sim::make_generator;

    #[test]
    fn set_selection_clamps_to_world_bounds() {
        let mut cell = SelectedCell::default();
        set_selection(
            &mut cell,
            WorldPos::new(10_000, 10_000, 10_000),
        );
        assert_eq!(cell.pos.x, WORLD_SIZE as u16 - 1);
    }

    #[test]
    fn slice_depth_follows_selection() {
        let selection = SelectedCell {
            pos: WorldPos::new(5, 9, 3),
        };
        assert_eq!(slice_depth(OrthoView::Top, &selection), 9);
        assert_eq!(slice_depth(OrthoView::Front, &selection), 3);
    }

    #[test]
    fn random_selection_uses_nonempty_candidate() {
        let mut sim = Simulation::new();
        let pos = WorldPos::new(10, 11, 12);
        sim.world.set(pos, make_generator(20, 0));
        let picked = random_selection(&[pos], &sim);
        assert_eq!(picked, pos);
    }

    #[test]
    fn random_selection_falls_back_when_empty() {
        let sim = Simulation::new();
        let picked = random_selection(&[WorldPos::new(1, 2, 3)], &sim);
        assert_eq!(picked, FALLBACK_SELECTION);
    }
}
