//! Orthographic view faces, camera framing, and view rotation.

use bevy::prelude::*;
use bevy::render::camera::{OrthographicProjection, Projection, ScalingMode};
use boxes_sim::{WorldPos, WORLD_SIZE};

/// World-space offset so grid center sits at origin.
pub const WORLD_CENTER: f32 = (WORLD_SIZE as f32 - 1.0) / 2.0;

pub const ZOOM_CELLS_DEFAULT: f32 = 32.0;
pub const ZOOM_CELLS_MIN: f32 = 8.0;
pub const ZOOM_CELLS_MAX: f32 = 64.0;

const CARDINALS: [Vec3; 6] = [
    Vec3::Y,
    Vec3::NEG_Y,
    Vec3::Z,
    Vec3::NEG_Z,
    Vec3::NEG_X,
    Vec3::X,
];

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

/// Full camera orientation around a look-at target (cardinal axes only).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ViewPose {
    /// Unit vector from target toward camera.
    pub view_dir: Vec3,
    /// Unit camera-up, orthogonal to `view_dir`.
    pub up: Vec3,
    /// World axis along which UV `u` increases on this face.
    pub u_axis: Vec3,
    /// World axis along which UV `v` increases on this face.
    pub v_axis: Vec3,
}

impl ViewPose {
    #[must_use]
    pub fn top_default() -> Self {
        Self {
            view_dir: Vec3::Y,
            up: Vec3::NEG_Z,
            u_axis: Vec3::X,
            v_axis: Vec3::Z,
        }
    }

    #[must_use]
    pub fn face(&self) -> OrthoView {
        OrthoView::from_view_dir(self.view_dir)
    }

    #[must_use]
    pub fn screen_right(&self) -> Vec3 {
        snap_cardinal(self.up.cross(self.view_dir))
    }

    #[must_use]
    pub fn slice_uses_lte(self) -> bool {
        self.face().slice_uses_lte_for_dir(self.view_dir)
    }

    #[must_use]
    pub fn slice_depth(self, pos: WorldPos) -> u16 {
        self.face().slice_depth(pos)
    }

    #[must_use]
    pub fn nudge_depth(self, pos: WorldPos, delta: i16) -> WorldPos {
        self.face().nudge_depth(pos, delta)
    }

    /// Sign for stepping one cell forward into the scene (scroll up / `]`).
    #[must_use]
    pub fn depth_step_forward_sign(self) -> i16 {
        if self.slice_uses_lte() { -1 } else { 1 }
    }

    /// Map a forward/backward step count to the depth-axis delta for `nudge_depth`.
    #[must_use]
    pub fn depth_step_delta(self, forward_steps: i16) -> i16 {
        forward_steps * self.depth_step_forward_sign()
    }

    #[must_use]
    pub fn nudge_screen(self, pos: WorldPos, dir: ScreenDir) -> WorldPos {
        let (dx, dy, dz) = self.screen_delta(dir);
        let max = WORLD_SIZE as u16 - 1;
        let clamp = |v: i32| v.clamp(0, i32::from(max)) as u16;
        WorldPos::new(
            clamp(i32::from(pos.x) + i32::from(dx)),
            clamp(i32::from(pos.y) + i32::from(dy)),
            clamp(i32::from(pos.z) + i32::from(dz)),
        )
    }

    #[must_use]
    pub fn screen_delta(self, dir: ScreenDir) -> (i16, i16, i16) {
        let vec = match dir {
            ScreenDir::Up => self.up,
            ScreenDir::Down => -self.up,
            ScreenDir::Left => -self.screen_right(),
            ScreenDir::Right => self.screen_right(),
        };
        quantize_cell_delta(vec)
    }

    pub fn rotate(&mut self, dir: ScreenDir) {
        let angle = match dir {
            ScreenDir::Up => -std::f32::consts::FRAC_PI_2,
            ScreenDir::Down => std::f32::consts::FRAC_PI_2,
            ScreenDir::Left => -std::f32::consts::FRAC_PI_2,
            ScreenDir::Right => std::f32::consts::FRAC_PI_2,
        };
        let axis = match dir {
            ScreenDir::Up | ScreenDir::Down => self.screen_right(),
            ScreenDir::Left | ScreenDir::Right => self.up,
        };
        let q = Quat::from_axis_angle(axis, angle);
        self.view_dir = snap_cardinal(q * self.view_dir);
        self.up = snap_cardinal(q * self.up);
        self.u_axis = snap_cardinal(q * self.u_axis);
        self.v_axis = snap_cardinal(q * self.v_axis);
    }

    #[must_use]
    pub fn camera_offset(self, distance: f32) -> Vec3 {
        self.view_dir * distance
    }

    #[must_use]
    pub fn camera_up(self) -> Vec3 {
        self.up
    }
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
    pub const fn depth_world_axis(self) -> Vec3 {
        match self {
            Self::Top | Self::Bottom => Vec3::Y,
            Self::Front | Self::Back => Vec3::Z,
            Self::Left | Self::Right => Vec3::X,
        }
    }

    #[must_use]
    pub fn slice_uses_lte_for_dir(self, view_dir: Vec3) -> bool {
        view_dir.dot(self.depth_world_axis()) > 0.0
    }

    #[must_use]
    pub fn from_view_dir(dir: Vec3) -> Self {
        match snap_cardinal(dir) {
            Vec3::Y => Self::Top,
            Vec3::NEG_Y => Self::Bottom,
            Vec3::Z => Self::Front,
            Vec3::NEG_Z => Self::Back,
            Vec3::NEG_X => Self::Left,
            Vec3::X => Self::Right,
            _ => Self::Top,
        }
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

    /// Default pose for tests and canonical face presets.
    #[must_use]
    pub fn default_pose(self) -> ViewPose {
        match self {
            Self::Top => ViewPose::top_default(),
            Self::Bottom => ViewPose {
                view_dir: Vec3::NEG_Y,
                up: Vec3::Z,
                u_axis: Vec3::X,
                v_axis: Vec3::NEG_Z,
            },
            Self::Front => ViewPose {
                view_dir: Vec3::Z,
                up: Vec3::Y,
                u_axis: Vec3::X,
                v_axis: Vec3::Y,
            },
            Self::Back => ViewPose {
                view_dir: Vec3::NEG_Z,
                up: Vec3::NEG_Y,
                u_axis: Vec3::NEG_X,
                v_axis: Vec3::NEG_Y,
            },
            Self::Left => ViewPose {
                view_dir: Vec3::NEG_X,
                up: Vec3::Y,
                u_axis: Vec3::Y,
                v_axis: Vec3::Z,
            },
            Self::Right => ViewPose {
                view_dir: Vec3::X,
                up: Vec3::Y,
                u_axis: Vec3::NEG_Y,
                v_axis: Vec3::NEG_Z,
            },
        }
    }
}

fn snap_cardinal(v: Vec3) -> Vec3 {
    CARDINALS
        .into_iter()
        .max_by(|a, b| a.dot(v).partial_cmp(&b.dot(v)).unwrap())
        .unwrap_or(Vec3::Y)
}

fn quantize_cell_delta(vec: Vec3) -> (i16, i16, i16) {
    let sx = vec.x.round() as i32;
    let sy = vec.y.round() as i32;
    let sz = vec.z.round() as i32;
    (
        sx.clamp(-1, 1) as i16,
        sy.clamp(-1, 1) as i16,
        sz.clamp(-1, 1) as i16,
    )
}

/// Currently displayed camera pose around the selection.
#[derive(Resource, Clone, Copy, Debug)]
pub struct ActiveView {
    pub pose: ViewPose,
}

impl Default for ActiveView {
    fn default() -> Self {
        Self {
            pose: ViewPose::top_default(),
        }
    }
}

impl ActiveView {
    #[must_use]
    pub fn face(&self) -> OrthoView {
        self.pose.face()
    }

    pub fn rotate(&mut self, dir: ScreenDir) {
        self.pose.rotate(dir);
    }

    pub fn snap_top(&mut self) {
        self.pose = ViewPose::top_default();
    }
}

/// Handle for the single grid orthographic camera.
#[derive(Resource)]
pub struct GridCameraEntity(pub Entity);

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
    active.snap_top();
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
    let pose = active.pose;

    if let Ok(mut transform) = transforms.get_mut(camera_entity.0) {
        *transform = Transform::from_translation(target + pose.camera_offset(distance))
            .looking_at(target, pose.camera_up());
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
    active.rotate(dir);
}

pub fn snap_top_view_system(keyboard: Res<ButtonInput<KeyCode>>, mut active: ResMut<ActiveView>) {
    if keyboard.just_pressed(KeyCode::KeyT) {
        active.snap_top();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rotate(mut active: ActiveView, dir: ScreenDir) -> ActiveView {
        active.rotate(dir);
        active
    }

    fn assert_vec_close(a: Vec3, b: Vec3) {
        assert!(
            (a - b).length_squared() < 0.01,
            "expected {b:?}, got {a:?}"
        );
    }

    #[test]
    fn top_up_moves_to_back_with_inverted_y() {
        let active = rotate(ActiveView::default(), ScreenDir::Up);
        assert_eq!(active.face(), OrthoView::Back);
        assert_vec_close(active.pose.up, Vec3::NEG_Y);
    }

    #[test]
    fn top_down_moves_to_front_with_normal_y() {
        let active = rotate(ActiveView::default(), ScreenDir::Down);
        assert_eq!(active.face(), OrthoView::Front);
        assert_vec_close(active.pose.up, Vec3::Y);
    }

    #[test]
    fn repeated_right_from_top_cycles_equatorial_faces() {
        let mut active = ActiveView::default();
        for expected in [
            OrthoView::Right,
            OrthoView::Bottom,
            OrthoView::Left,
            OrthoView::Top,
            OrthoView::Right,
        ] {
            active = rotate(active, ScreenDir::Right);
            assert_eq!(active.face(), expected);
        }
    }

    #[test]
    fn repeated_left_from_top_cycles_equatorial_faces() {
        let mut active = ActiveView::default();
        for expected in [
            OrthoView::Left,
            OrthoView::Bottom,
            OrthoView::Right,
            OrthoView::Top,
            OrthoView::Left,
        ] {
            active = rotate(active, ScreenDir::Left);
            assert_eq!(active.face(), expected);
        }
    }

    #[test]
    fn top_screen_up_moves_toward_negative_z() {
        let pos = WorldPos::new(10, 5, 8);
        let next = ViewPose::top_default().nudge_screen(pos, ScreenDir::Up);
        assert_eq!(next, WorldPos::new(10, 5, 7));
    }

    #[test]
    fn left_view_screen_left_moves_along_z() {
        let pos = WorldPos::new(10, 5, 8);
        let pose = OrthoView::Left.default_pose();
        let next = pose.nudge_screen(pos, ScreenDir::Left);
        assert_eq!(next, WorldPos::new(10, 5, 7));
        let next = pose.nudge_screen(pos, ScreenDir::Right);
        assert_eq!(next, WorldPos::new(10, 5, 9));
    }

    #[test]
    fn left_view_screen_up_moves_along_y() {
        let pos = WorldPos::new(10, 5, 8);
        let next = OrthoView::Left.default_pose().nudge_screen(pos, ScreenDir::Up);
        assert_eq!(next, WorldPos::new(10, 6, 8));
    }

    #[test]
    fn back_from_top_up_screen_left_moves_along_negative_x() {
        let pose = rotate(ActiveView::default(), ScreenDir::Up).pose;
        let pos = WorldPos::new(10, 5, 8);
        let next = pose.nudge_screen(pos, ScreenDir::Left);
        assert_eq!(next, WorldPos::new(9, 5, 8));
    }

    #[test]
    fn ctrl_left_steps_through_equatorial_faces() {
        let mut active = ActiveView::default();
        for expected in [OrthoView::Left, OrthoView::Bottom, OrthoView::Right] {
            active = rotate(active, ScreenDir::Left);
            assert_eq!(active.face(), expected);
        }
    }

    #[test]
    fn ctrl_right_from_back_orbits_through_side_faces() {
        let mut active = rotate(ActiveView::default(), ScreenDir::Up);
        assert_eq!(active.face(), OrthoView::Back);

        for expected in [
            OrthoView::Right,
            OrthoView::Front,
            OrthoView::Left,
            OrthoView::Back,
        ] {
            active = rotate(active, ScreenDir::Right);
            assert_eq!(active.face(), expected, "after ctrl+right step");
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
    fn depth_step_forward_sign_for_default_faces() {
        let cases = [
            (OrthoView::Top, -1),
            (OrthoView::Bottom, 1),
            (OrthoView::Left, 1),
            (OrthoView::Right, -1),
            (OrthoView::Back, 1),
            (OrthoView::Front, -1),
        ];
        for (face, expected) in cases {
            assert_eq!(
                face.default_pose().depth_step_forward_sign(),
                expected,
                "{face:?}"
            );
        }
    }

    #[test]
    fn depth_step_forward_moves_into_scene_for_top_view() {
        let pose = OrthoView::Top.default_pose();
        let pos = WorldPos::new(10, 10, 10);
        let next = pose.nudge_depth(pos, pose.depth_step_delta(1));
        assert_eq!(next, WorldPos::new(10, 9, 10));
    }

    #[test]
    fn depth_step_forward_moves_into_scene_for_bottom_view() {
        let pose = OrthoView::Bottom.default_pose();
        let pos = WorldPos::new(10, 10, 10);
        let next = pose.nudge_depth(pos, pose.depth_step_delta(1));
        assert_eq!(next, WorldPos::new(10, 11, 10));
    }
}
