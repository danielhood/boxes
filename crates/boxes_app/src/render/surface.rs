//! View-dependent surface extraction (top / front / left).

use std::collections::HashMap;

use boxes_sim::{Cell, ChunkCoord, Simulation, WorldPos, CHUNK_SIZE, CHUNKS_PER_AXIS, WORLD_SIZE};

use super::view::OrthoView;

/// 2D projection key for a view face.
#[must_use]
pub fn view_key(pos: WorldPos, view: OrthoView) -> (u16, u16) {
    match view {
        OrthoView::Top => (pos.x, pos.z),
        OrthoView::Front => (pos.x, pos.y),
        OrthoView::Left => (pos.y, pos.z),
    }
}

fn is_better(candidate: WorldPos, current: WorldPos, view: OrthoView) -> bool {
    match view {
        // Top: highest Y column cell.
        OrthoView::Top => candidate.y > current.y,
        // Front: nearest toward +Z (camera looks from +Z toward origin).
        OrthoView::Front => candidate.z > current.z,
        // Left: nearest toward -X (camera looks from -X toward origin).
        OrthoView::Left => candidate.x < current.x,
    }
}

/// Depth coordinate of `pos` along the axis perpendicular to `view`.
#[must_use]
pub fn cell_depth(pos: WorldPos, view: OrthoView) -> u16 {
    match view {
        OrthoView::Top => pos.y,
        OrthoView::Front => pos.z,
        OrthoView::Left => pos.x,
    }
}

/// Slice depth that shows the full grid (no clipping) for `view`.
#[must_use]
#[cfg_attr(not(test), allow(dead_code))]
pub fn unclipped_slice_depth(view: OrthoView) -> u16 {
    match view {
        // Camera at -X: all cells satisfy x >= 0.
        OrthoView::Left => 0,
        OrthoView::Top | OrthoView::Front => WORLD_SIZE as u16 - 1,
    }
}
/// True when the cell is at or behind the slice plane (not between slice and camera).
#[must_use]
pub fn is_cell_visible_at_slice(pos: WorldPos, view: OrthoView, slice_depth: u16) -> bool {
    match view {
        OrthoView::Top | OrthoView::Front => cell_depth(pos, view) <= slice_depth,
        OrthoView::Left => cell_depth(pos, view) >= slice_depth,
    }
}

/// Build the visible surface map for the active orthographic face, clipped at `slice_depth`.
#[must_use]
pub fn visible_surface(
    sim: &Simulation,
    view: OrthoView,
    slice_depth: u16,
) -> HashMap<(u16, u16), (WorldPos, Cell)> {
    let mut surface = HashMap::new();

    for (pos, cell) in sim.world.chunks.iter_non_empty() {
        if cell.is_empty() || !is_cell_visible_at_slice(pos, view, slice_depth) {
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

/// Chunks whose rendered instance buffers may change when `pos` updates.
#[must_use]
pub fn affected_chunks(pos: WorldPos, view: OrthoView) -> Vec<ChunkCoord> {
    let cx = pos.x / CHUNK_SIZE;
    let cy = pos.y / CHUNK_SIZE;
    let cz = pos.z / CHUNK_SIZE;

    match view {
        OrthoView::Top => (0..CHUNKS_PER_AXIS)
            .map(|chunk_y| ChunkCoord::new(cx, chunk_y, cz))
            .collect(),
        OrthoView::Front => (0..CHUNKS_PER_AXIS)
            .map(|chunk_z| ChunkCoord::new(cx, cy, chunk_z))
            .collect(),
        OrthoView::Left => (0..CHUNKS_PER_AXIS)
            .map(|chunk_x| ChunkCoord::new(chunk_x, cy, cz))
            .collect(),
    }
}

/// Surface cells that render from a given chunk.
#[must_use]
pub fn surface_cells_for_chunk(
    surface: &HashMap<(u16, u16), (WorldPos, Cell)>,
    coord: ChunkCoord,
) -> Vec<(WorldPos, Cell)> {
    surface
        .values()
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

        let surface = visible_surface(&sim, OrthoView::Top, unclipped_slice_depth(OrthoView::Top));
        let (_, cell) = surface[&(10, 10)];
        assert_eq!(cell.state, 2);
    }

    #[test]
    fn front_view_picks_max_z() {
        let mut sim = Simulation::new();
        sim.world
            .set(WorldPos::new(5, 5, 1), make_generator(20, 1));
        sim.world
            .set(WorldPos::new(5, 5, 9), make_generator(20, 2));

        let surface = visible_surface(&sim, OrthoView::Front, unclipped_slice_depth(OrthoView::Front));
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

        let surface = visible_surface(&sim, OrthoView::Top, 5);
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

        let surface = visible_surface(&sim, OrthoView::Left, 5);
        let (pos, cell) = surface[&(5, 5)];
        assert_eq!(pos, WorldPos::new(8, 5, 5));
        assert_eq!(cell.state, 2);
    }

    #[test]
    fn is_cell_visible_at_slice_respects_view_axis() {
        assert!(is_cell_visible_at_slice(
            WorldPos::new(10, 5, 10),
            OrthoView::Top,
            5
        ));
        assert!(!is_cell_visible_at_slice(
            WorldPos::new(10, 6, 10),
            OrthoView::Top,
            5
        ));
        assert!(is_cell_visible_at_slice(
            WorldPos::new(8, 5, 5),
            OrthoView::Left,
            5
        ));
        assert!(!is_cell_visible_at_slice(
            WorldPos::new(4, 5, 5),
            OrthoView::Left,
            5
        ));
    }
}
