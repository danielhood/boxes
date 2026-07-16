//! Orthographic ray pick math — view UV, depth slice, surface resolution.

use bevy::prelude::*;
use boxes_sim::{Simulation, WorldPos, WORLD_SIZE};

use crate::render::{visible_surface, OrthoView, WORLD_CENTER};

/// Axis perpendicular to an orthographic face (depth into the screen).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DepthAxis {
    X,
    Y,
    Z,
}

impl OrthoView {
    #[must_use]
    pub const fn depth_axis(self) -> DepthAxis {
        match self {
            Self::Top => DepthAxis::Y,
            Self::Front => DepthAxis::Z,
            Self::Left => DepthAxis::X,
        }
    }
}

/// World-space point on the view plane at `depth` (cell coordinate along depth axis).
#[must_use]
pub fn view_plane_point(view: OrthoView, depth: u16) -> Vec3 {
    let d = depth as f32 - WORLD_CENTER;
    match view {
        OrthoView::Top => Vec3::new(0.0, d, 0.0),
        OrthoView::Front => Vec3::new(0.0, 0.0, d),
        OrthoView::Left => Vec3::new(d, 0.0, 0.0),
    }
}

/// Outward normal of the view plane (toward the camera).
#[must_use]
pub fn view_plane_normal(view: OrthoView) -> Vec3 {
    match view {
        OrthoView::Top => Vec3::Y,
        OrthoView::Front => Vec3::Z,
        OrthoView::Left => Vec3::NEG_X,
    }
}

/// Intersect a ray with a plane defined by a point and normal.
#[must_use]
pub fn ray_plane_intersect(
    origin: Vec3,
    direction: Vec3,
    plane_point: Vec3,
    plane_normal: Vec3,
) -> Option<Vec3> {
    let denom = direction.dot(plane_normal);
    if denom.abs() < f32::EPSILON {
        return None;
    }
    let t = (plane_point - origin).dot(plane_normal) / denom;
    if t < 0.0 {
        return None;
    }
    Some(origin + direction * t)
}

/// Map a world-space hit on the view plane to 2D grid coordinates for the active face.
#[must_use]
pub fn world_hit_to_uv(view: OrthoView, hit: Vec3) -> Option<(u16, u16)> {
    let (u, v) = match view {
        OrthoView::Top => (
            (hit.x + WORLD_CENTER).round() as i32,
            (hit.z + WORLD_CENTER).round() as i32,
        ),
        OrthoView::Front => (
            (hit.x + WORLD_CENTER).round() as i32,
            (hit.y + WORLD_CENTER).round() as i32,
        ),
        OrthoView::Left => (
            (hit.y + WORLD_CENTER).round() as i32,
            (hit.z + WORLD_CENTER).round() as i32,
        ),
    };

    let max = WORLD_SIZE as i32;
    if (0..max).contains(&u) && (0..max).contains(&v) {
        Some((u as u16, v as u16))
    } else {
        None
    }
}

/// Build a world cell from view UV and depth slice along the face normal.
#[must_use]
pub fn uv_depth_to_cell(view: OrthoView, u: u16, v: u16, depth: u16) -> WorldPos {
    match view {
        OrthoView::Top => WorldPos::new(u, depth, v),
        OrthoView::Front => WorldPos::new(u, v, depth),
        OrthoView::Left => WorldPos::new(depth, u, v),
    }
}

/// Resolve the visible surface cell at view UV, if the column is non-empty.
#[must_use]
pub fn pick_surface_at_uv(
    sim: &Simulation,
    view: OrthoView,
    u: u16,
    v: u16,
    slice_depth: u16,
) -> Option<WorldPos> {
    let surface = visible_surface(sim, view, slice_depth);
    surface.get(&(u, v)).map(|(pos, _)| *pos)
}

/// Pick the cell under the cursor for surface interaction (inspect / erase / place-on-surface).
#[must_use]
pub fn pick_surface_cell(
    sim: &Simulation,
    view: OrthoView,
    slice_depth: u16,
    ray_origin: Vec3,
    ray_direction: Vec3,
) -> Option<WorldPos> {
    // Intersect with a plane through world origin; UV comes from the hit regardless of depth.
    let plane_point = Vec3::ZERO;
    let plane_normal = view_plane_normal(view);
    let hit = ray_plane_intersect(ray_origin, ray_direction, plane_point, plane_normal)?;
    let (u, v) = world_hit_to_uv(view, hit)?;
    pick_surface_at_uv(sim, view, u, v, slice_depth)
}

/// Pick a cell at the current depth slice (for placement into empty columns).
#[must_use]
pub fn pick_slice_cell(
    view: OrthoView,
    depth: u16,
    ray_origin: Vec3,
    ray_direction: Vec3,
) -> Option<WorldPos> {
    let plane_point = view_plane_point(view, depth);
    let plane_normal = view_plane_normal(view);
    let hit = ray_plane_intersect(ray_origin, ray_direction, plane_point, plane_normal)?;
    let (u, v) = world_hit_to_uv(view, hit)?;
    let pos = uv_depth_to_cell(view, u, v, depth);
    pos.is_in_bounds().then_some(pos)
}

#[cfg(test)]
mod tests {
    use super::*;
    use boxes_sim::{make_generator, make_transformer, Direction};

    use crate::render::unclipped_slice_depth;

    const EPS: f32 = 0.01;

    fn assert_vec3_close(a: Vec3, b: Vec3) {
        assert!(
            (a - b).length() < EPS,
            "expected {b:?}, got {a:?}"
        );
    }

    #[test]
    fn top_view_world_hit_maps_xz() {
        let hit = Vec3::new(10.0 - WORLD_CENTER, 0.0, 20.0 - WORLD_CENTER);
        let uv = world_hit_to_uv(OrthoView::Top, hit).unwrap();
        assert_eq!(uv, (10, 20));
    }

    #[test]
    fn front_view_world_hit_maps_xy() {
        let hit = Vec3::new(5.0 - WORLD_CENTER, 7.0 - WORLD_CENTER, 0.0);
        let uv = world_hit_to_uv(OrthoView::Front, hit).unwrap();
        assert_eq!(uv, (5, 7));
    }

    #[test]
    fn left_view_world_hit_maps_yz() {
        let hit = Vec3::new(0.0, 3.0 - WORLD_CENTER, 9.0 - WORLD_CENTER);
        let uv = world_hit_to_uv(OrthoView::Left, hit).unwrap();
        assert_eq!(uv, (3, 9));
    }

    #[test]
    fn uv_depth_round_trip_top() {
        let pos = WorldPos::new(42, 17, 99);
        let uv = (pos.x, pos.z);
        assert_eq!(uv_depth_to_cell(OrthoView::Top, uv.0, uv.1, pos.y), pos);
    }

    #[test]
    fn uv_depth_round_trip_front() {
        let pos = WorldPos::new(11, 22, 33);
        let uv = (pos.x, pos.y);
        assert_eq!(uv_depth_to_cell(OrthoView::Front, uv.0, uv.1, pos.z), pos);
    }

    #[test]
    fn uv_depth_round_trip_left() {
        let pos = WorldPos::new(44, 55, 66);
        let uv = (pos.y, pos.z);
        assert_eq!(uv_depth_to_cell(OrthoView::Left, uv.0, uv.1, pos.x), pos);
    }

    #[test]
    fn top_surface_pick_highest_y() {
        let mut sim = Simulation::new();
        sim.world
            .set(WorldPos::new(10, 2, 10), make_generator(20, 1));
        sim.world
            .set(WorldPos::new(10, 8, 10), make_generator(20, 2));

        let picked =
            pick_surface_at_uv(&sim, OrthoView::Top, 10, 10, unclipped_slice_depth(OrthoView::Top))
                .unwrap();
        assert_eq!(picked, WorldPos::new(10, 8, 10));
    }

    #[test]
    fn front_surface_pick_max_z() {
        let mut sim = Simulation::new();
        sim.world
            .set(WorldPos::new(5, 5, 1), make_generator(20, 1));
        sim.world
            .set(WorldPos::new(5, 5, 9), make_generator(20, 2));

        let picked = pick_surface_at_uv(
            &sim,
            OrthoView::Front,
            5,
            5,
            unclipped_slice_depth(OrthoView::Front),
        )
        .unwrap();
        assert_eq!(picked, WorldPos::new(5, 5, 9));
    }

    #[test]
    fn left_surface_pick_min_x() {
        let mut sim = Simulation::new();
        sim.world
            .set(WorldPos::new(9, 5, 5), make_generator(20, 1));
        sim.world
            .set(WorldPos::new(1, 5, 5), make_generator(20, 2));

        let picked = pick_surface_at_uv(
            &sim,
            OrthoView::Left,
            5,
            5,
            unclipped_slice_depth(OrthoView::Left),
        )
        .unwrap();
        assert_eq!(picked, WorldPos::new(1, 5, 5));
    }

    #[test]
    fn top_ray_pick_test_vector() {
        // Camera above origin, ray straight down through cell (10, 8, 10).
        let origin = Vec3::new(10.0 - WORLD_CENTER, 100.0, 10.0 - WORLD_CENTER);
        let direction = Vec3::NEG_Y;

        let mut sim = Simulation::new();
        sim.world
            .set(WorldPos::new(10, 8, 10), make_transformer(Direction::PosX, 0));

        let picked = pick_surface_cell(
            &sim,
            OrthoView::Top,
            unclipped_slice_depth(OrthoView::Top),
            origin,
            direction,
        )
        .unwrap();
        assert_eq!(picked, WorldPos::new(10, 8, 10));
    }

    #[test]
    fn front_ray_pick_test_vector() {
        let origin = Vec3::new(5.0 - WORLD_CENTER, 5.0 - WORLD_CENTER, 100.0);
        let direction = Vec3::NEG_Z;

        let mut sim = Simulation::new();
        sim.world
            .set(WorldPos::new(5, 5, 9), make_generator(20, 1));

        let picked = pick_surface_cell(
            &sim,
            OrthoView::Front,
            unclipped_slice_depth(OrthoView::Front),
            origin,
            direction,
        )
        .unwrap();
        assert_eq!(picked, WorldPos::new(5, 5, 9));
    }

    #[test]
    fn left_ray_pick_test_vector() {
        let origin = Vec3::new(-100.0, 5.0 - WORLD_CENTER, 5.0 - WORLD_CENTER);
        let direction = Vec3::X;

        let mut sim = Simulation::new();
        sim.world
            .set(WorldPos::new(1, 5, 5), make_generator(20, 1));

        let picked = pick_surface_cell(
            &sim,
            OrthoView::Left,
            unclipped_slice_depth(OrthoView::Left),
            origin,
            direction,
        )
        .unwrap();
        assert_eq!(picked, WorldPos::new(1, 5, 5));
    }

    #[test]
    fn slice_pick_at_depth() {
        let origin = Vec3::new(15.0 - WORLD_CENTER, 100.0, 20.0 - WORLD_CENTER);
        let direction = Vec3::NEG_Y;
        let picked = pick_slice_cell(OrthoView::Top, 7, origin, direction).unwrap();
        assert_eq!(picked, WorldPos::new(15, 7, 20));
    }

    #[test]
    fn ray_plane_intersect_hits() {
        let hit = ray_plane_intersect(
            Vec3::new(0.0, 10.0, 0.0),
            Vec3::NEG_Y,
            Vec3::new(0.0, 3.0, 0.0),
            Vec3::Y,
        )
        .unwrap();
        assert_vec3_close(hit, Vec3::new(0.0, 3.0, 0.0));
    }
}
