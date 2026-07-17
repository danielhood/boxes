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
        let max = WORLD_SIZE as u16 - 1;
        let clamp = |v: i32| v.clamp(0, i32::from(max)) as u16;
        match self {
            Self::Top | Self::Bottom => WorldPos::new(
                clamp(i32::from(pos.x) + i32::from(du)),
                pos.y,
                clamp(i32::from(pos.z) + i32::from(dv)),
            ),
            Self::Front | Self::Back => WorldPos::new(
                clamp(i32::from(pos.x) + i32::from(du)),
                clamp(i32::from(pos.y) + i32::from(dv)),
                pos.z,
            ),
            Self::Left | Self::Right => WorldPos::new(
                pos.x,
                clamp(i32::from(pos.y) + i32::from(du)),
                clamp(i32::from(pos.z) + i32::from(dv)),
            ),
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
            (Self::Bottom, ScreenDir::Left) => Self::Left,
            (Self::Bottom, ScreenDir::Right) => Self::Right,

            (Self::Front, ScreenDir::Up) => Self::Top,
            (Self::Front, ScreenDir::Down) => Self::Bottom,
            (Self::Front, ScreenDir::Left) => Self::Left,
            (Self::Front, ScreenDir::Right) => Self::Right,

            (Self::Back, ScreenDir::Up) => Self::Top,
            (Self::Back, ScreenDir::Down) => Self::Bottom,
            (Self::Back, ScreenDir::Left) => Self::Right,
            (Self::Back, ScreenDir::Right) => Self::Left,

            (Self::Left, ScreenDir::Up) => Self::Top,
            (Self::Left, ScreenDir::Down) => Self::Bottom,
            (Self::Left, ScreenDir::Left) => Self::Back,
            (Self::Left, ScreenDir::Right) => Self::Front,

            (Self::Right, ScreenDir::Up) => Self::Top,
            (Self::Right, ScreenDir::Down) => Self::Bottom,
            (Self::Right, ScreenDir::Left) => Self::Front,
            (Self::Right, ScreenDir::Right) => Self::Back,
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
            Self::Front | Self::Back | Self::Left | Self::Right => Vec3::Y,
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
    fn left_rotate_adjacency_matches_spec() {
        assert_eq!(OrthoView::Left.rotate(ScreenDir::Up), OrthoView::Top);
        assert_eq!(OrthoView::Left.rotate(ScreenDir::Down), OrthoView::Bottom);
        assert_eq!(OrthoView::Left.rotate(ScreenDir::Left), OrthoView::Back);
        assert_eq!(OrthoView::Left.rotate(ScreenDir::Right), OrthoView::Front);
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
}
