//! Orthographic view faces, camera framing, and view rotation.

use bevy::prelude::*;
use bevy::render::camera::{OrthographicProjection, Projection, ScalingMode};
use boxes_sim::{WorldPos, WORLD_SIZE};

/// World-space offset so grid center sits at origin.
pub const WORLD_CENTER: f32 = (WORLD_SIZE as f32 - 1.0) / 2.0;

pub const ZOOM_CELLS_DEFAULT: f32 = 32.0;
pub const ZOOM_CELLS_MIN: f32 = 8.0;
pub const ZOOM_CELLS_MAX: f32 = 64.0;

/// Screen-relative direction for view rotation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScreenDir {
    Up,
    Down,
    Left,
    Right,
}

/// Active orthographic face (six axis-aligned views).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum OrthoView {
    #[default]
    Top,
    Bottom,
    Front,
    Back,
    Left,
    Right,
}

impl OrthoView {
    pub const ALL: [Self; 6] = [
        Self::Top,
        Self::Bottom,
        Self::Front,
        Self::Back,
        Self::Left,
        Self::Right,
    ];

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Top => "Top",
            Self::Bottom => "Bottom",
            Self::Front => "Front",
            Self::Back => "Back",
            Self::Left => "Left",
            Self::Right => "Right",
        }
    }

    #[must_use]
    pub const fn depth_axis_label(self) -> &'static str {
        match self {
            Self::Top | Self::Bottom => "Y",
            Self::Front | Self::Back => "Z",
            Self::Left | Self::Right => "X",
        }
    }

    #[must_use]
    pub const fn slice_uses_lte(self) -> bool {
        matches!(self, Self::Top | Self::Front | Self::Right)
    }

    #[must_use]
    pub fn slice_depth(self, pos: WorldPos) -> u16 {
        match self {
            Self::Top | Self::Bottom => pos.y,
            Self::Front | Self::Back => pos.z,
            Self::Left | Self::Right => pos.x,
        }
    }

    #[must_use]
    pub fn nudge_depth(self, pos: WorldPos, delta: i16) -> WorldPos {
        let max = WORLD_SIZE as u16 - 1;
        match self {
            Self::Top | Self::Bottom => {
                let y = (i32::from(pos.y) + i32::from(delta)).clamp(0, i32::from(max)) as u16;
                WorldPos::new(pos.x, y, pos.z)
            }
            Self::Front | Self::Back => {
                let z = (i32::from(pos.z) + i32::from(delta)).clamp(0, i32::from(max)) as u16;
                WorldPos::new(pos.x, pos.y, z)
            }
            Self::Left | Self::Right => {
                let x = (i32::from(pos.x) + i32::from(delta)).clamp(0, i32::from(max)) as u16;
                WorldPos::new(x, pos.y, pos.z)
            }
        }
    }

    #[must_use]
    pub fn nudge_uv(self, pos: WorldPos, du: i16, dv: i16) -> WorldPos {
        let (dx, dy, dz) = self.uv_delta(du, dv);
        self.apply_delta(pos, dx, dy, dz)
    }

    /// Move selection along screen-space arrow keys for this face.
    #[must_use]
    pub fn nudge_screen(self, pos: WorldPos, dir: ScreenDir) -> WorldPos {
        let (dx, dy, dz) = self.screen_delta(dir);
        self.apply_delta(pos, dx, dy, dz)
    }

    fn apply_delta(self, pos: WorldPos, dx: i16, dy: i16, dz: i16) -> WorldPos {
        let max = WORLD_SIZE as u16 - 1;
        let clamp = |v: i32| v.clamp(0, i32::from(max)) as u16;
        WorldPos::new(
            clamp(i32::from(pos.x) + i32::from(dx)),
            clamp(i32::from(pos.y) + i32::from(dy)),
            clamp(i32::from(pos.z) + i32::from(dz)),
        )
    }

    /// World-axis delta for abstract UV nudges (used by pick round-trips).
    const fn uv_delta(self, du: i16, dv: i16) -> (i16, i16, i16) {
        match self {
            Self::Top | Self::Bottom => (du, 0, dv),
            Self::Front | Self::Back => (du, dv, 0),
            Self::Left | Self::Right => (0, du, dv),
        }
    }

    /// World-axis delta matching on-screen arrow directions for this face.
    const fn screen_delta(self, dir: ScreenDir) -> (i16, i16, i16) {
        match (self, dir) {
            (Self::Top, ScreenDir::Up) => (0, 0, -1),
            (Self::Top, ScreenDir::Down) => (0, 0, 1),
            (Self::Top, ScreenDir::Left) => (-1, 0, 0),
            (Self::Top, ScreenDir::Right) => (1, 0, 0),

            (Self::Bottom, ScreenDir::Up) => (0, 0, 1),
            (Self::Bottom, ScreenDir::Down) => (0, 0, -1),
            (Self::Bottom, ScreenDir::Left) => (-1, 0, 0),
            (Self::Bottom, ScreenDir::Right) => (1, 0, 0),

            (Self::Front, ScreenDir::Up) => (0, 1, 0),
            (Self::Front, ScreenDir::Down) => (0, -1, 0),
            (Self::Front, ScreenDir::Left) => (-1, 0, 0),
            (Self::Front, ScreenDir::Right) => (1, 0, 0),

            (Self::Back, ScreenDir::Up) => (0, -1, 0),
            (Self::Back, ScreenDir::Down) => (0, 1, 0),
            (Self::Back, ScreenDir::Left) => (1, 0, 0),
            (Self::Back, ScreenDir::Right) => (-1, 0, 0),

            (Self::Left, ScreenDir::Up) => (0, 1, 0),
            (Self::Left, ScreenDir::Down) => (0, -1, 0),
            (Self::Left, ScreenDir::Left) => (0, 0, -1),
            (Self::Left, ScreenDir::Right) => (0, 0, 1),

            (Self::Right, ScreenDir::Up) => (0, 1, 0),
            (Self::Right, ScreenDir::Down) => (0, -1, 0),
            (Self::Right, ScreenDir::Left) => (0, 0, 1),
            (Self::Right, ScreenDir::Right) => (0, 0, -1),
        }
    }

    #[must_use]
    pub const fn rotate(self, dir: ScreenDir) -> Self {
        match (self, dir) {
            (Self::Top, ScreenDir::Up) => Self::Back,
            (Self::Top, ScreenDir::Down) => Self::Front,
            (Self::Top, ScreenDir::Left) => Self::Left,
            (Self::Top, ScreenDir::Right) => Self::Right,

            (Self::Bottom, ScreenDir::Up) => Self::Front,
            (Self::Bottom, ScreenDir::Down) => Self::Back,
            // Horizontal orbit: Top → Left → Bottom → Right → Top
            (Self::Bottom, ScreenDir::Left) => Self::Right,
            (Self::Bottom, ScreenDir::Right) => Self::Left,

            (Self::Front, ScreenDir::Up) => Self::Top,
            (Self::Front, ScreenDir::Down) => Self::Bottom,
            (Self::Front, ScreenDir::Left) => Self::Left,
            (Self::Front, ScreenDir::Right) => Self::Right,

            // Vertical orbit: Top → Back → Bottom → Front → Top
            (Self::Back, ScreenDir::Up) => Self::Bottom,
            (Self::Back, ScreenDir::Down) => Self::Top,
            (Self::Back, ScreenDir::Left) => Self::Right,
            (Self::Back, ScreenDir::Right) => Self::Left,

            (Self::Left, ScreenDir::Up) => Self::Top,
            (Self::Left, ScreenDir::Down) => Self::Bottom,
            (Self::Left, ScreenDir::Left) => Self::Bottom,
            (Self::Left, ScreenDir::Right) => Self::Top,

            (Self::Right, ScreenDir::Up) => Self::Top,
            (Self::Right, ScreenDir::Down) => Self::Bottom,
            (Self::Right, ScreenDir::Left) => Self::Top,
            (Self::Right, ScreenDir::Right) => Self::Bottom,
        }
    }

    /// Camera position offset from look-at target (along outward normal × distance).
    #[must_use]
    pub fn camera_offset(self, distance: f32) -> Vec3 {
        match self {
            Self::Top => Vec3::Y * distance,
            Self::Bottom => Vec3::NEG_Y * distance,
            Self::Front => Vec3::Z * distance,
            Self::Back => Vec3::NEG_Z * distance,
            Self::Left => Vec3::NEG_X * distance,
            Self::Right => Vec3::X * distance,
        }
    }

    /// World up vector for `Transform::looking_at`.
    #[must_use]
    pub const fn camera_up(self) -> Vec3 {
        match self {
            Self::Top => Vec3::NEG_Z,
            Self::Bottom => Vec3::Z,
            Self::Front | Self::Left | Self::Right => Vec3::Y,
            Self::Back => Vec3::NEG_Y,
        }
    }
}

/// Handle for the single grid orthographic camera.
#[derive(Resource)]
pub struct GridCameraEntity(pub Entity);

/// Currently displayed orthographic face.
#[derive(Resource, Default)]
pub struct ActiveView(pub OrthoView);

/// Zoom level — visible world units along the orthographic viewport height.
#[derive(Resource, Clone, Copy, Debug)]
pub struct ViewCameraState {
    pub zoom_cells: f32,
}

impl Default for ViewCameraState {
    fn default() -> Self {
        Self {
            zoom_cells: ZOOM_CELLS_DEFAULT,
        }
    }
}

impl ViewCameraState {
    #[must_use]
    pub fn clamp_zoom(zoom: f32) -> f32 {
        zoom.clamp(ZOOM_CELLS_MIN, ZOOM_CELLS_MAX)
    }

    pub fn nudge_zoom(&mut self, delta: f32) {
        self.zoom_cells = Self::clamp_zoom(self.zoom_cells - delta);
    }
}

/// Marker for the grid view camera.
#[derive(Component)]
pub struct GridCamera;

#[must_use]
pub fn cell_to_world(pos: WorldPos) -> Vec3 {
    Vec3::new(
        pos.x as f32 - WORLD_CENTER,
        pos.y as f32 - WORLD_CENTER,
        pos.z as f32 - WORLD_CENTER,
    )
}

pub fn setup_cameras(mut commands: Commands, mut active: ResMut<ActiveView>) {
    let distance = WORLD_SIZE as f32 * 1.2;
    let projection = Projection::from(OrthographicProjection {
        scaling_mode: ScalingMode::FixedVertical {
            viewport_height: ZOOM_CELLS_DEFAULT,
        },
        ..OrthographicProjection::default_3d()
    });

    let camera = commands
        .spawn((
            Camera3d::default(),
            projection,
            GridCamera,
            Transform::from_xyz(0.0, distance, 0.0).looking_at(Vec3::ZERO, Vec3::NEG_Z),
            Camera {
                order: 0,
                is_active: true,
                ..default()
            },
        ))
        .id();

    commands.insert_resource(GridCameraEntity(camera));
    active.0 = OrthoView::Top;
}

pub fn apply_camera_framing(
    active: Res<ActiveView>,
    camera_state: Res<ViewCameraState>,
    selection: Res<crate::input::SelectedCell>,
    camera_entity: Res<GridCameraEntity>,
    mut transforms: Query<&mut Transform, With<GridCamera>>,
    mut projections: Query<&mut Projection, With<GridCamera>>,
) {
    let target = cell_to_world(selection.pos);
    let distance = WORLD_SIZE as f32 * 1.2;
    let view = active.0;

    if let Ok(mut transform) = transforms.get_mut(camera_entity.0) {
        *transform = Transform::from_translation(target + view.camera_offset(distance))
            .looking_at(target, view.camera_up());
    }

    if let Ok(mut projection) = projections.get_mut(camera_entity.0) {
        if let Projection::Orthographic(ref mut ortho) = *projection {
            ortho.scaling_mode = ScalingMode::FixedVertical {
                viewport_height: camera_state.zoom_cells,
            };
        }
    }
}

fn ctrl_held(keyboard: &ButtonInput<KeyCode>) -> bool {
    keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight)
}

fn screen_dir_from_arrow(keyboard: &ButtonInput<KeyCode>) -> Option<ScreenDir> {
    if keyboard.just_pressed(KeyCode::ArrowUp) {
        Some(ScreenDir::Up)
    } else if keyboard.just_pressed(KeyCode::ArrowDown) {
        Some(ScreenDir::Down)
    } else if keyboard.just_pressed(KeyCode::ArrowLeft) {
        Some(ScreenDir::Left)
    } else if keyboard.just_pressed(KeyCode::ArrowRight) {
        Some(ScreenDir::Right)
    } else {
        None
    }
}

pub fn view_rotate_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut active: ResMut<ActiveView>,
) {
    if !ctrl_held(&keyboard) {
        return;
    }
    let Some(dir) = screen_dir_from_arrow(&keyboard) else {
        return;
    };
    active.0 = active.0.rotate(dir);
}

pub fn snap_top_view_system(keyboard: Res<ButtonInput<KeyCode>>, mut active: ResMut<ActiveView>) {
    if keyboard.just_pressed(KeyCode::KeyT) {
        active.0 = OrthoView::Top;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn top_rotate_adjacency_matches_spec() {
        assert_eq!(OrthoView::Top.rotate(ScreenDir::Up), OrthoView::Back);
        assert_eq!(OrthoView::Top.rotate(ScreenDir::Down), OrthoView::Front);
        assert_eq!(OrthoView::Top.rotate(ScreenDir::Left), OrthoView::Left);
        assert_eq!(OrthoView::Top.rotate(ScreenDir::Right), OrthoView::Right);
    }

    #[test]
    fn ctrl_up_orbits_top_back_bottom_front_top() {
        let mut view = OrthoView::Top;
        for expected in [
            OrthoView::Back,
            OrthoView::Bottom,
            OrthoView::Front,
            OrthoView::Top,
        ] {
            view = view.rotate(ScreenDir::Up);
            assert_eq!(view, expected);
        }
    }

    #[test]
    fn ctrl_down_orbits_top_front_bottom_back_top() {
        let mut view = OrthoView::Top;
        for expected in [
            OrthoView::Front,
            OrthoView::Bottom,
            OrthoView::Back,
            OrthoView::Top,
        ] {
            view = view.rotate(ScreenDir::Down);
            assert_eq!(view, expected);
        }
    }

    #[test]
    fn left_rotate_adjacency_matches_spec() {
        assert_eq!(OrthoView::Left.rotate(ScreenDir::Up), OrthoView::Top);
        assert_eq!(OrthoView::Left.rotate(ScreenDir::Down), OrthoView::Bottom);
        assert_eq!(OrthoView::Left.rotate(ScreenDir::Left), OrthoView::Bottom);
        assert_eq!(OrthoView::Left.rotate(ScreenDir::Right), OrthoView::Top);
    }

    #[test]
    fn ctrl_left_orbits_top_left_bottom_right_top() {
        let mut view = OrthoView::Top;
        for expected in [
            OrthoView::Left,
            OrthoView::Bottom,
            OrthoView::Right,
            OrthoView::Top,
        ] {
            view = view.rotate(ScreenDir::Left);
            assert_eq!(view, expected);
        }
    }

    #[test]
    fn ctrl_right_orbits_top_right_bottom_left_top() {
        let mut view = OrthoView::Top;
        for expected in [
            OrthoView::Right,
            OrthoView::Bottom,
            OrthoView::Left,
            OrthoView::Top,
        ] {
            view = view.rotate(ScreenDir::Right);
            assert_eq!(view, expected);
        }
    }

    #[test]
    fn zoom_clamps_to_limits() {
        assert_eq!(ViewCameraState::clamp_zoom(4.0), ZOOM_CELLS_MIN);
        assert_eq!(ViewCameraState::clamp_zoom(100.0), ZOOM_CELLS_MAX);
        assert_eq!(ViewCameraState::clamp_zoom(32.0), 32.0);
    }

    #[test]
    fn slice_depth_from_position_per_view() {
        let pos = WorldPos::new(10, 20, 30);
        assert_eq!(OrthoView::Top.slice_depth(pos), 20);
        assert_eq!(OrthoView::Front.slice_depth(pos), 30);
        assert_eq!(OrthoView::Left.slice_depth(pos), 10);
    }

    #[test]
    fn left_view_screen_left_moves_along_z() {
        let pos = WorldPos::new(10, 5, 8);
        let next = OrthoView::Left.nudge_screen(pos, ScreenDir::Left);
        assert_eq!(next, WorldPos::new(10, 5, 7));
        let next = OrthoView::Left.nudge_screen(pos, ScreenDir::Right);
        assert_eq!(next, WorldPos::new(10, 5, 9));
    }

    #[test]
    fn left_view_screen_up_moves_along_y() {
        let pos = WorldPos::new(10, 5, 8);
        let next = OrthoView::Left.nudge_screen(pos, ScreenDir::Up);
        assert_eq!(next, WorldPos::new(10, 6, 8));
    }

    #[test]
    fn top_view_screen_up_moves_toward_negative_z() {
        let pos = WorldPos::new(10, 5, 8);
        let next = OrthoView::Top.nudge_screen(pos, ScreenDir::Up);
        assert_eq!(next, WorldPos::new(10, 5, 7));
    }

    #[test]
    fn back_view_screen_left_moves_toward_positive_x() {
        let pos = WorldPos::new(10, 5, 8);
        let next = OrthoView::Back.nudge_screen(pos, ScreenDir::Left);
        assert_eq!(next, WorldPos::new(11, 5, 8));
    }
}
