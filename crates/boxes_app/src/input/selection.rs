//! Persistent selected cell and orbit anchor — navigation focal points.

use bevy::prelude::*;
use boxes_sim::{Simulation, WorldPos, WORLD_SIZE};

use crate::input::pick::{
    anchor_uv_to_include_selection, cell_at_uv_depth, cell_to_uv_f, pan_anchor_on_slice,
};
use crate::render::{ViewCameraState, ViewPose};

/// World center fallback when no seeded cells exist.
pub const FALLBACK_SELECTION: WorldPos = WorldPos::new(250, 250, 250);

/// Always-on UI selection anchor.
#[derive(Resource, Clone, Copy, Debug)]
pub struct SelectedCell {
    pub pos: WorldPos,
}

/// Camera look-at, rotation pivot, and zoom center.
#[derive(Resource, Clone, Copy, Debug)]
pub struct OrbitAnchor {
    pub pos: WorldPos,
}

impl Default for SelectedCell {
    fn default() -> Self {
        Self {
            pos: FALLBACK_SELECTION,
        }
    }
}

impl Default for OrbitAnchor {
    fn default() -> Self {
        Self {
            pos: FALLBACK_SELECTION,
        }
    }
}

/// Set when the most recent selection UV change came from arrow keys (not LMB).
#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct LastSelectionMove {
    pub keyboard_nav: bool,
}

/// Middle-mouse pan drag state (sub-cell offset until release).
#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct MmbPanState {
    pub dragging: bool,
    pub anchor_uv_start: (f32, f32),
    pub uv_offset: (f32, f32),
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

/// Update orbit anchor, clamping to grid bounds.
pub fn set_orbit_anchor(anchor: &mut OrbitAnchor, pos: WorldPos) {
    anchor.pos = clamp_pos(pos);
}

/// Snap the orbit anchor to the selected cell (recenter view).
pub fn recenter_on_selection(anchor: &mut OrbitAnchor, selection: &SelectedCell) {
    anchor.pos = selection.pos;
}

/// Pan the orbit anchor so `selection` lies within the viewport.
pub fn pan_anchor_to_show_selection(
    pose: ViewPose,
    anchor: &mut OrbitAnchor,
    selection: WorldPos,
    active_depth: u16,
    anchor_uv: (f32, f32),
    zoom_cells: f32,
    aspect: f32,
) {
    let sel_uv = cell_to_uv_f(pose, selection);
    let (u, v) = anchor_uv_to_include_selection(anchor_uv, sel_uv, zoom_cells, aspect);
    anchor.pos = cell_at_uv_depth(pose, u, v, active_depth);
}

/// Pan the orbit anchor one quarter viewport on the active slice.
pub fn pan_orbit_anchor(
    pose: ViewPose,
    anchor: &mut OrbitAnchor,
    active_depth: u16,
    dir: crate::render::ScreenDir,
    camera: &ViewCameraState,
    aspect: f32,
) {
    pan_anchor_on_slice(
        pose,
        &mut anchor.pos,
        active_depth,
        dir,
        camera.zoom_cells,
        aspect,
    );
}

/// Effective look-at UV on the active slice (includes MMB drag offset).
#[must_use]
pub fn orbit_look_at_uv(
    pose: ViewPose,
    anchor: &OrbitAnchor,
    mmb: &MmbPanState,
) -> (f32, f32) {
    if mmb.dragging {
        (
            mmb.anchor_uv_start.0 + mmb.uv_offset.0,
            mmb.anchor_uv_start.1 + mmb.uv_offset.1,
        )
    } else {
        cell_to_uv_f(pose, anchor.pos)
    }
}

/// Apply a fractional UV pan delta during MMB drag.
pub fn apply_mmb_uv_delta(mmb: &mut MmbPanState, du: f32, dv: f32) {
    mmb.uv_offset.0 += du;
    mmb.uv_offset.1 += dv;
}

/// Snap orbit anchor to the viewport center cell after MMB release.
pub fn finish_mmb_pan(
    anchor: &mut OrbitAnchor,
    mmb: &mut MmbPanState,
    pose: ViewPose,
    active_depth: u16,
) {
    if !mmb.dragging {
        return;
    }
    let u = mmb.anchor_uv_start.0 + mmb.uv_offset.0;
    let v = mmb.anchor_uv_start.1 + mmb.uv_offset.1;
    anchor.pos = cell_at_uv_depth(pose, u, v, active_depth);
    set_orbit_anchor(anchor, anchor.pos);
    *mmb = MmbPanState::default();
}

#[must_use]
pub fn slice_depth(pose: ViewPose, selection: &SelectedCell) -> u16 {
    pose.slice_depth(selection.pos)
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
    use crate::render::OrthoView;

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
        assert_eq!(slice_depth(ViewPose::top_default(), &selection), 9);
        assert_eq!(
            slice_depth(OrthoView::Front.default_pose(), &selection),
            3
        );
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

    #[test]
    fn recenter_on_selection_sets_anchor() {
        let selection = SelectedCell {
            pos: WorldPos::new(10, 20, 30),
        };
        let mut anchor = OrbitAnchor {
            pos: WorldPos::new(1, 2, 3),
        };
        recenter_on_selection(&mut anchor, &selection);
        assert_eq!(anchor.pos, selection.pos);
    }
}
