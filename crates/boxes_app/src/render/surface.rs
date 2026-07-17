//! View-dependent surface extraction for six orthographic faces.

use std::collections::HashMap;

use boxes_sim::{Cell, ChunkCoord, Simulation, WorldPos, CHUNK_SIZE, CHUNKS_PER_AXIS, WORLD_SIZE};

use super::view::{OrthoView, ViewPose};

/// 2D projection key for a view face.
#[must_use]
pub fn view_key(pos: WorldPos, view: OrthoView) -> (u16, u16) {
    match view {
        OrthoView::Top | OrthoView::Bottom => (pos.x, pos.z),
        OrthoView::Front | OrthoView::Back => (pos.x, pos.y),
        OrthoView::Left | OrthoView::Right => (pos.y, pos.z),
    }
}

fn is_better(candidate: WorldPos, current: WorldPos, view: OrthoView) -> bool {
    match view {
        OrthoView::Top => candidate.y > current.y,
        OrthoView::Bottom => candidate.y < current.y,
        OrthoView::Front => candidate.z > current.z,
        OrthoView::Back => candidate.z < current.z,
        OrthoView::Left => candidate.x < current.x,
        OrthoView::Right => candidate.x > current.x,
    }
}

/// Depth coordinate of `pos` along the axis perpendicular to `view`.
#[must_use]
pub fn cell_depth(pos: WorldPos, view: OrthoView) -> u16 {
    view.slice_depth(pos)
}

/// Slice depth that shows the full grid (no clipping) for `view`.
#[must_use]
#[cfg_attr(not(test), allow(dead_code))]
pub fn unclipped_slice_depth(view: OrthoView) -> u16 {
    match view {
        OrthoView::Left | OrthoView::Bottom | OrthoView::Back => 0,
        OrthoView::Top | OrthoView::Front | OrthoView::Right => WORLD_SIZE as u16 - 1,
    }
}

/// True when the cell is at or behind the slice plane (not between slice and camera).
#[must_use]
pub fn is_cell_visible_at_slice(pos: WorldPos, pose: ViewPose, slice_depth: u16) -> bool {
    let depth = cell_depth(pos, pose.face());
    if pose.slice_uses_lte() {
        depth <= slice_depth
    } else {
        depth >= slice_depth
    }
}

/// True when the cell lies on the active slice plane.
#[must_use]
pub fn is_on_slice(pos: WorldPos, pose: ViewPose, slice_depth: u16) -> bool {
    cell_depth(pos, pose.face()) == slice_depth
}

/// Sort key for back-to-front draw order (farther cells first).
#[must_use]
pub fn depth_draw_order(pos: WorldPos, pose: ViewPose) -> u16 {
    let depth = cell_depth(pos, pose.face());
    if pose.slice_uses_lte() {
        depth
    } else {
        u16::MAX - depth
    }
}

/// Build the visible surface map for the active orthographic face, clipped at `slice_depth`.
#[must_use]
pub fn visible_surface(
    sim: &Simulation,
    pose: ViewPose,
    slice_depth: u16,
) -> HashMap<(u16, u16), (WorldPos, Cell)> {
    let view = pose.face();
    let mut surface = HashMap::new();

    for (pos, cell) in sim.world.chunks.iter_non_empty() {
        if cell.is_empty() || !is_cell_visible_at_slice(pos, pose, slice_depth) {
            continue;
        }

        let key = view_key(pos, view);
        surface
            .entry(key)
            .and_modify(|(best_pos, best_cell)| {
                if is_better(pos, *best_pos, view) {
                    *best_pos = pos;
                    *best_cell = cell;
                }
            })
            .or_insert((pos, cell));
    }

    surface
}

/// All non-empty cells visible at `slice_depth` (not just the outermost per column).
#[must_use]
pub fn visible_cells(
    sim: &Simulation,
    pose: ViewPose,
    slice_depth: u16,
) -> Vec<(WorldPos, Cell)> {
    let mut cells = Vec::new();

    for (pos, cell) in sim.world.chunks.iter_non_empty() {
        if cell.is_empty() || !is_cell_visible_at_slice(pos, pose, slice_depth) {
            continue;
        }
        cells.push((pos, cell));
    }

    cells.sort_by_key(|(pos, _)| depth_draw_order(*pos, pose));
    cells
}

/// Chunks whose rendered instance buffers may change when `pos` updates.
#[must_use]
pub fn affected_chunks(pos: WorldPos, view: OrthoView) -> Vec<ChunkCoord> {
    let cx = pos.x / CHUNK_SIZE;
    let cy = pos.y / CHUNK_SIZE;
    let cz = pos.z / CHUNK_SIZE;

    match view {
        OrthoView::Top | OrthoView::Bottom => (0..CHUNKS_PER_AXIS)
            .map(|chunk_y| ChunkCoord::new(cx, chunk_y, cz))
            .collect(),
        OrthoView::Front | OrthoView::Back => (0..CHUNKS_PER_AXIS)
            .map(|chunk_z| ChunkCoord::new(cx, cy, chunk_z))
            .collect(),
        OrthoView::Left | OrthoView::Right => (0..CHUNKS_PER_AXIS)
            .map(|chunk_x| ChunkCoord::new(chunk_x, cy, cz))
            .collect(),
    }
}

/// Visible cells that render from a given chunk.
#[must_use]
pub fn cells_for_chunk(
    cells: &[(WorldPos, Cell)],
    coord: ChunkCoord,
) -> Vec<(WorldPos, Cell)> {
    cells
        .iter()
        .copied()
        .filter(|(pos, _)| pos.chunk_coord() == coord)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use boxes_sim::{make_generator, Simulation};

    use super::super::surface::unclipped_slice_depth;

    #[test]
    fn top_view_picks_highest_cell_in_column() {
        let mut sim = Simulation::new();
        sim.world
            .set(WorldPos::new(10, 1, 10), make_generator(20, 1));
        sim.world
            .set(WorldPos::new(10, 5, 10), make_generator(20, 2));

        let surface = visible_surface(
            &sim,
            OrthoView::Top.default_pose(),
            unclipped_slice_depth(OrthoView::Top),
        );
        let (_, cell) = surface[&(10, 10)];
        assert_eq!(cell.state, 2);
    }

    #[test]
    fn bottom_view_picks_lowest_y() {
        let mut sim = Simulation::new();
        sim.world
            .set(WorldPos::new(10, 1, 10), make_generator(20, 1));
        sim.world
            .set(WorldPos::new(10, 5, 10), make_generator(20, 2));

        let surface = visible_surface(
            &sim,
            OrthoView::Bottom.default_pose(),
            unclipped_slice_depth(OrthoView::Bottom),
        );
        let (_, cell) = surface[&(10, 10)];
        assert_eq!(cell.state, 1);
    }

    #[test]
    fn back_view_picks_min_z() {
        let mut sim = Simulation::new();
        sim.world
            .set(WorldPos::new(5, 5, 1), make_generator(20, 1));
        sim.world
            .set(WorldPos::new(5, 5, 9), make_generator(20, 2));

        let surface = visible_surface(
            &sim,
            OrthoView::Back.default_pose(),
            unclipped_slice_depth(OrthoView::Back),
        );
        let (_, cell) = surface[&(5, 5)];
        assert_eq!(cell.state, 1);
    }

    #[test]
    fn right_view_picks_max_x() {
        let mut sim = Simulation::new();
        sim.world
            .set(WorldPos::new(1, 5, 5), make_generator(20, 1));
        sim.world
            .set(WorldPos::new(9, 5, 5), make_generator(20, 2));

        let surface = visible_surface(
            &sim,
            OrthoView::Right.default_pose(),
            unclipped_slice_depth(OrthoView::Right),
        );
        let (_, cell) = surface[&(5, 5)];
        assert_eq!(cell.state, 2);
    }

    #[test]
    fn front_view_picks_max_z() {
        let mut sim = Simulation::new();
        sim.world
            .set(WorldPos::new(5, 5, 1), make_generator(20, 1));
        sim.world
            .set(WorldPos::new(5, 5, 9), make_generator(20, 2));

        let surface = visible_surface(
            &sim,
            OrthoView::Front.default_pose(),
            unclipped_slice_depth(OrthoView::Front),
        );
        let (_, cell) = surface[&(5, 5)];
        assert_eq!(cell.state, 2);
    }

    #[test]
    fn top_slice_hides_cells_above_depth() {
        let mut sim = Simulation::new();
        sim.world
            .set(WorldPos::new(10, 3, 10), make_generator(20, 1));
        sim.world
            .set(WorldPos::new(10, 8, 10), make_generator(20, 2));

        let surface = visible_surface(&sim, OrthoView::Top.default_pose(), 5);
        let (pos, cell) = surface[&(10, 10)];
        assert_eq!(pos, WorldPos::new(10, 3, 10));
        assert_eq!(cell.state, 1);
    }

    #[test]
    fn left_slice_hides_cells_between_slice_and_camera() {
        let mut sim = Simulation::new();
        sim.world
            .set(WorldPos::new(2, 5, 5), make_generator(20, 1));
        sim.world
            .set(WorldPos::new(8, 5, 5), make_generator(20, 2));

        let surface = visible_surface(&sim, OrthoView::Left.default_pose(), 5);
        let (pos, cell) = surface[&(5, 5)];
        assert_eq!(pos, WorldPos::new(8, 5, 5));
        assert_eq!(cell.state, 2);
    }

    #[test]
    fn is_cell_visible_at_slice_respects_view_axis() {
        assert!(is_cell_visible_at_slice(
            WorldPos::new(10, 5, 10),
            OrthoView::Top.default_pose(),
            5
        ));
        assert!(!is_cell_visible_at_slice(
            WorldPos::new(10, 6, 10),
            OrthoView::Top.default_pose(),
            5
        ));
        assert!(is_cell_visible_at_slice(
            WorldPos::new(8, 5, 5),
            OrthoView::Left.default_pose(),
            5
        ));
        assert!(!is_cell_visible_at_slice(
            WorldPos::new(4, 5, 5),
            OrthoView::Left.default_pose(),
            5
        ));
    }

    #[test]
    fn visible_cells_includes_all_layers_in_column() {
        let mut sim = Simulation::new();
        sim.world
            .set(WorldPos::new(10, 2, 10), make_generator(20, 1));
        sim.world
            .set(WorldPos::new(10, 5, 10), make_generator(20, 2));

        let cells = visible_cells(&sim, OrthoView::Top.default_pose(), 5);
        assert_eq!(cells.len(), 2);
    }

    #[test]
    fn is_on_slice_matches_depth() {
        let pose = OrthoView::Top.default_pose();
        assert!(is_on_slice(WorldPos::new(10, 5, 10), pose, 5));
        assert!(!is_on_slice(WorldPos::new(10, 4, 10), pose, 5));
    }

    #[test]
    fn depth_draw_order_puts_slice_cells_last_for_top_view() {
        let pose = OrthoView::Top.default_pose();
        let near = depth_draw_order(WorldPos::new(10, 5, 10), pose);
        let far = depth_draw_order(WorldPos::new(10, 2, 10), pose);
        assert!(far < near);
    }
}
