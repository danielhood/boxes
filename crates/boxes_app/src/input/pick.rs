//! Orthographic ray pick math — view UV, depth slice, surface resolution.

use bevy::prelude::*;
use boxes_sim::{Simulation, WorldPos, WORLD_SIZE};

use crate::render::{visible_surface, ViewPose, OrthoView, WORLD_CENTER};

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
            Self::Top | Self::Bottom => DepthAxis::Y,
            Self::Front | Self::Back => DepthAxis::Z,
            Self::Left | Self::Right => DepthAxis::X,
        }
    }
}

/// World-space point on the view plane at `depth` (cell coordinate along depth axis).
#[must_use]
pub fn view_plane_point(pose: ViewPose, depth: u16) -> Vec3 {
    let d = depth as f32 - WORLD_CENTER;
    pose.view_dir * d
}

/// Outward normal of the view plane (toward the camera).
#[must_use]
pub fn view_plane_normal(pose: ViewPose) -> Vec3 {
    pose.view_dir
}

/// Map a world-space hit on the view plane to 2D grid coordinates for the active pose.
#[must_use]
pub fn world_hit_to_uv(pose: ViewPose, hit: Vec3) -> Option<(u16, u16)> {
    let u = (hit.dot(pose.u_axis) + WORLD_CENTER).round() as i32;
    let v = (hit.dot(pose.v_axis) + WORLD_CENTER).round() as i32;

    let max = WORLD_SIZE as i32;
    if (0..max).contains(&u) && (0..max).contains(&v) {
        Some((u as u16, v as u16))
    } else {
        None
    }
}

/// Build a world cell from view UV and depth slice along the face normal.
#[must_use]
pub fn uv_depth_to_cell(pose: ViewPose, u: u16, v: u16, depth: u16) -> WorldPos {
    let depth_axis = pose.face().depth_world_axis();
    let rel = pose.u_axis * (u as f32 - WORLD_CENTER)
        + pose.v_axis * (v as f32 - WORLD_CENTER)
        + depth_axis * (depth as f32 - WORLD_CENTER);
    WorldPos::new(
        (rel.x + WORLD_CENTER).round() as u16,
        (rel.y + WORLD_CENTER).round() as u16,
        (rel.z + WORLD_CENTER).round() as u16,
    )
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

/// Resolve the visible surface cell at view UV, if the column is non-empty.
#[must_use]
pub fn pick_surface_at_uv(
    sim: &Simulation,
    pose: ViewPose,
    u: u16,
    v: u16,
    slice_depth: u16,
) -> Option<WorldPos> {
    let surface = visible_surface(sim, pose, slice_depth);
    surface.get(&(u, v)).map(|(pos, _)| *pos)
}

/// Pick the cell under the cursor for surface interaction (inspect / erase).
#[must_use]
pub fn pick_surface_cell(
    sim: &Simulation,
    pose: ViewPose,
    slice_depth: u16,
    ray_origin: Vec3,
    ray_direction: Vec3,
) -> Option<WorldPos> {
    let plane_point = Vec3::ZERO;
    let plane_normal = view_plane_normal(pose);
    let hit = ray_plane_intersect(ray_origin, ray_direction, plane_point, plane_normal)?;
    let (u, v) = world_hit_to_uv(pose, hit)?;
    pick_surface_at_uv(sim, pose, u, v, slice_depth)
}

/// Pick a cell at the current depth slice (for placement into empty columns).
#[must_use]
pub fn pick_slice_cell(
    pose: ViewPose,
    depth: u16,
    ray_origin: Vec3,
    ray_direction: Vec3,
) -> Option<WorldPos> {
    let plane_point = view_plane_point(pose, depth);
    let plane_normal = view_plane_normal(pose);
    let hit = ray_plane_intersect(ray_origin, ray_direction, plane_point, plane_normal)?;
    let (u, v) = world_hit_to_uv(pose, hit)?;
    let pos = uv_depth_to_cell(pose, u, v, depth);
    pos.is_in_bounds().then_some(pos)
}

#[cfg(test)]
mod tests {
    use super::*;
    use boxes_sim::{make_generator, make_transformer, Direction};

    use crate::render::{cell_to_world, unclipped_slice_depth, OrthoView};

    const EPS: f32 = 0.01;

    fn assert_vec3_close(a: Vec3, b: Vec3) {
        assert!(
            (a - b).length() < EPS,
            "expected {b:?}, got {a:?}"
        );
    }

    #[test]
    fn top_view_world_hit_maps_xz() {
        let pose = OrthoView::Top.default_pose();
        let hit = Vec3::new(10.0 - WORLD_CENTER, 0.0, 20.0 - WORLD_CENTER);
        let uv = world_hit_to_uv(pose, hit).unwrap();
        assert_eq!(uv, (10, 20));
    }

    #[test]
    fn bottom_view_world_hit_maps_xz() {
        let pose = OrthoView::Bottom.default_pose();
        let hit = Vec3::new(10.0 - WORLD_CENTER, 0.0, WORLD_CENTER - 20.0);
        let uv = world_hit_to_uv(pose, hit).unwrap();
        assert_eq!(uv, (10, 20));
    }

    #[test]
    fn back_view_world_hit_maps_xy() {
        let pose = OrthoView::Back.default_pose();
        let hit = Vec3::new(WORLD_CENTER - 5.0, 7.0 - WORLD_CENTER, 0.0);
        let uv = world_hit_to_uv(pose, hit).unwrap();
        assert_eq!(uv, (5, WORLD_SIZE as u16 - 1 - 7));
    }

    #[test]
    fn right_view_world_hit_maps_yz() {
        let pose = OrthoView::Right.default_pose();
        let hit = Vec3::new(0.0, WORLD_CENTER - 3.0, WORLD_CENTER - 9.0);
        let uv = world_hit_to_uv(pose, hit).unwrap();
        assert_eq!(uv, (3, 9));
    }

    #[test]
    fn front_view_world_hit_maps_xy() {
        let pose = OrthoView::Front.default_pose();
        let hit = Vec3::new(5.0 - WORLD_CENTER, 7.0 - WORLD_CENTER, 0.0);
        let uv = world_hit_to_uv(pose, hit).unwrap();
        assert_eq!(uv, (5, 7));
    }

    #[test]
    fn left_view_world_hit_maps_yz() {
        let pose = OrthoView::Left.default_pose();
        let hit = Vec3::new(0.0, 3.0 - WORLD_CENTER, 9.0 - WORLD_CENTER);
        let uv = world_hit_to_uv(pose, hit).unwrap();
        assert_eq!(uv, (3, 9));
    }

    #[test]
    fn uv_depth_round_trip_all_faces() {
        let pos = WorldPos::new(42, 17, 99);
        for view in OrthoView::ALL {
            let pose = view.default_pose();
            let depth = view.slice_depth(pos);
            let hit = cell_to_world(pos);
            let (u, v) = world_hit_to_uv(pose, hit).unwrap();
            assert_eq!(uv_depth_to_cell(pose, u, v, depth), pos, "round-trip failed for {view:?}");
        }
    }

    #[test]
    fn top_surface_pick_highest_y() {
        let mut sim = Simulation::new();
        sim.world
            .set(WorldPos::new(10, 2, 10), make_generator(20, 1));
        sim.world
            .set(WorldPos::new(10, 8, 10), make_generator(20, 2));

        let picked = pick_surface_at_uv(
            &sim,
            OrthoView::Top.default_pose(),
            10,
            10,
            unclipped_slice_depth(OrthoView::Top),
        )
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
            OrthoView::Front.default_pose(),
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
            OrthoView::Left.default_pose(),
            5,
            5,
            unclipped_slice_depth(OrthoView::Left),
        )
        .unwrap();
        assert_eq!(picked, WorldPos::new(1, 5, 5));
    }

    #[test]
    fn slice_pick_targets_depth_not_column_surface() {
        let mut sim = Simulation::new();
        sim.world
            .set(WorldPos::new(10, 8, 10), make_generator(20, 1));
        sim.world
            .set(WorldPos::new(10, 12, 10), make_generator(20, 2));

        let origin = Vec3::new(10.0 - WORLD_CENTER, 100.0, 10.0 - WORLD_CENTER);
        let direction = Vec3::NEG_Y;
        let slice = 14_u16;

        let surface = pick_surface_cell(
            &sim,
            OrthoView::Top.default_pose(),
            slice,
            origin,
            direction,
        ).unwrap();
        assert_eq!(surface, WorldPos::new(10, 12, 10));

        let at_slice = pick_slice_cell(OrthoView::Top.default_pose(), slice, origin, direction).unwrap();
        assert_eq!(at_slice, WorldPos::new(10, 14, 10));
    }

    #[test]
    fn top_ray_pick_test_vector() {
        let origin = Vec3::new(10.0 - WORLD_CENTER, 100.0, 10.0 - WORLD_CENTER);
        let direction = Vec3::NEG_Y;

        let mut sim = Simulation::new();
        sim.world
            .set(WorldPos::new(10, 8, 10), make_transformer(Direction::PosX, 0));

        let picked = pick_surface_cell(
            &sim,
            OrthoView::Top.default_pose(),
            unclipped_slice_depth(OrthoView::Top),
            origin,
            direction,
        )
        .unwrap();
        assert_eq!(picked, WorldPos::new(10, 8, 10));
    }

    #[test]
    fn slice_pick_at_depth() {
        let origin = Vec3::new(15.0 - WORLD_CENTER, 100.0, 20.0 - WORLD_CENTER);
        let direction = Vec3::NEG_Y;
        let picked = pick_slice_cell(OrthoView::Top.default_pose(), 7, origin, direction).unwrap();
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
