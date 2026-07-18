//! Orthographic ray pick math — view UV, depth slice, surface resolution.

use bevy::prelude::*;
use boxes_sim::{Simulation, WorldPos, WORLD_SIZE};

use crate::render::{cell_to_world, visible_surface, ViewPose, OrthoView, WORLD_CENTER};

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

/// View-plane UV coordinates for a cell (fractional).
#[must_use]
pub fn cell_to_uv_f(pose: ViewPose, pos: WorldPos) -> (f32, f32) {
    let world = cell_to_world(pos);
    (
        world.dot(pose.u_axis) + WORLD_CENTER,
        world.dot(pose.v_axis) + WORLD_CENTER,
    )
}

/// World-space point on the active slice at fractional UV.
#[must_use]
pub fn world_from_uv_on_slice(pose: ViewPose, u: f32, v: f32, depth: u16) -> Vec3 {
    let depth_axis = pose.face().depth_world_axis();
    pose.u_axis * (u - WORLD_CENTER)
        + pose.v_axis * (v - WORLD_CENTER)
        + depth_axis * (depth as f32 - WORLD_CENTER)
}

/// Snap fractional UV to the nearest cell on `depth`.
#[must_use]
pub fn cell_at_uv_depth(pose: ViewPose, u: f32, v: f32, depth: u16) -> WorldPos {
    let max = WORLD_SIZE as f32 - 1.0;
    let u = u.clamp(0.0, max).round() as u16;
    let v = v.clamp(0.0, max).round() as u16;
    uv_depth_to_cell(pose, u, v, depth)
}

/// Half-width and half-height of the visible viewport in cell units.
#[must_use]
pub fn viewport_half_extents(zoom_cells: f32, aspect: f32) -> (f32, f32) {
    let half_h = zoom_cells / 2.0;
    (half_h * aspect, half_h)
}

/// True when `selection` lies within the viewport centered on `anchor`.
#[must_use]
pub fn selection_in_viewport(
    pose: ViewPose,
    selection: WorldPos,
    anchor_uv: (f32, f32),
    zoom_cells: f32,
    aspect: f32,
) -> bool {
    let (sel_u, sel_v) = cell_to_uv_f(pose, selection);
    let (half_w, half_h) = viewport_half_extents(zoom_cells, aspect);
    let du = (sel_u - anchor_uv.0).abs();
    let dv = (sel_v - anchor_uv.1).abs();
    du <= half_w && dv <= half_h
}

/// UV delta for panning one screen direction by `amount` cells.
#[must_use]
pub fn pan_uv_delta(pose: ViewPose, dir: crate::render::ScreenDir, amount: f32) -> (f32, f32) {
    use crate::render::ScreenDir;
    let world = match dir {
        ScreenDir::Up => pose.up * amount,
        ScreenDir::Down => -pose.up * amount,
        ScreenDir::Left => -pose.screen_right() * amount,
        ScreenDir::Right => pose.screen_right() * amount,
    };
    (world.dot(pose.u_axis), world.dot(pose.v_axis))
}

/// Map cursor pixel movement to UV delta on the active slice.
#[must_use]
pub fn cursor_delta_to_uv(
    pose: ViewPose,
    pixel_delta: Vec2,
    zoom_cells: f32,
    viewport_height: f32,
) -> (f32, f32) {
    let scale = zoom_cells / viewport_height;
    let world =
        -pose.screen_right() * pixel_delta.x * scale + pose.up * pixel_delta.y * scale;
    (world.dot(pose.u_axis), world.dot(pose.v_axis))
}

/// Pan an anchor on the active slice by one quarter viewport.
pub fn pan_anchor_on_slice(
    pose: ViewPose,
    anchor: &mut WorldPos,
    active_depth: u16,
    dir: crate::render::ScreenDir,
    zoom_cells: f32,
    aspect: f32,
) {
    let quarter = zoom_cells / 4.0;
    let (du, dv) = pan_uv_delta(pose, dir, quarter);
    let (mut u, mut v) = cell_to_uv_f(pose, *anchor);
    u += du;
    v += dv;
    let (half_w, half_h) = viewport_half_extents(zoom_cells, aspect);
    let max = WORLD_SIZE as f32 - 1.0;
    u = u.clamp(half_w, max - half_w);
    v = v.clamp(half_h, max - half_h);
    *anchor = cell_at_uv_depth(pose, u, v, active_depth);
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

    #[test]
    fn cell_to_uv_f_round_trip() {
        let pos = WorldPos::new(42, 17, 99);
        let pose = OrthoView::Top.default_pose();
        let (u, v) = cell_to_uv_f(pose, pos);
        assert!((u - 42.0).abs() < 0.01);
        assert!((v - 99.0).abs() < 0.01);
    }

    #[test]
    fn selection_in_viewport_at_center() {
        let pose = OrthoView::Top.default_pose();
        let pos = WorldPos::new(100, 50, 100);
        let uv = cell_to_uv_f(pose, pos);
        assert!(selection_in_viewport(pose, pos, uv, 32.0, 1.0));
    }

    #[test]
    fn selection_outside_viewport() {
        let pose = OrthoView::Top.default_pose();
        let anchor = WorldPos::new(100, 50, 100);
        let anchor_uv = cell_to_uv_f(pose, anchor);
        let far = WorldPos::new(200, 50, 200);
        assert!(!selection_in_viewport(pose, far, anchor_uv, 32.0, 1.0));
    }

    #[test]
    fn pan_uv_delta_quarter_zoom() {
        let pose = OrthoView::Top.default_pose();
        let (_, dv) = pan_uv_delta(pose, crate::render::ScreenDir::Up, 8.0);
        assert!((dv + 8.0).abs() < 0.01);
    }

    #[test]
    fn cell_at_uv_depth_snaps() {
        let pose = OrthoView::Top.default_pose();
        let pos = cell_at_uv_depth(pose, 10.4, 20.6, 7);
        assert_eq!(pos, WorldPos::new(10, 7, 21));
    }
}
