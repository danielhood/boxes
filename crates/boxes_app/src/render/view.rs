//! Orthographic view faces and camera setup.

use bevy::prelude::*;
use bevy::render::camera::{OrthographicProjection, Projection, ScalingMode};
use boxes_sim::WORLD_SIZE;

/// World-space offset so grid center sits at origin.
pub const WORLD_CENTER: f32 = (WORLD_SIZE as f32 - 1.0) / 2.0;

/// Active orthographic face.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum OrthoView {
    #[default]
    Top,
    Front,
    Left,
}

impl OrthoView {
    pub const ALL: [Self; 3] = [Self::Top, Self::Front, Self::Left];

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Top => "Top",
            Self::Front => "Front",
            Self::Left => "Left",
        }
    }
}

/// Handles for the three orthographic cameras.
#[derive(Resource)]
pub struct ViewCameras {
    pub top: Entity,
    pub front: Entity,
    pub left: Entity,
}

/// Currently displayed orthographic face.
#[derive(Resource, Default)]
pub struct ActiveView(pub OrthoView);

/// Marker for grid view cameras.
#[derive(Component)]
pub struct GridCamera;

pub fn setup_cameras(mut commands: Commands, mut active: ResMut<ActiveView>) {
    let projection = Projection::from(OrthographicProjection {
        scaling_mode: ScalingMode::FixedVertical {
            viewport_height: 80.0,
        },
        ..OrthographicProjection::default_3d()
    });

    let distance = WORLD_SIZE as f32 * 1.2;

    let top = commands
        .spawn((
            Camera3d::default(),
            projection.clone(),
            GridCamera,
            Transform::from_xyz(0.0, distance, 0.0).looking_at(Vec3::ZERO, Vec3::NEG_Z),
            Camera {
                order: 0,
                is_active: true,
                ..default()
            },
        ))
        .id();

    let front = commands
        .spawn((
            Camera3d::default(),
            projection.clone(),
            GridCamera,
            Transform::from_xyz(0.0, 0.0, distance).looking_at(Vec3::ZERO, Vec3::Y),
            Camera {
                order: 0,
                is_active: false,
                ..default()
            },
        ))
        .id();

    let left = commands
        .spawn((
            Camera3d::default(),
            projection,
            GridCamera,
            Transform::from_xyz(-distance, 0.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
            Camera {
                order: 0,
                is_active: false,
                ..default()
            },
        ))
        .id();

    commands.insert_resource(ViewCameras { top, front, left });
    active.0 = OrthoView::Top;
}

fn shift_held(keyboard: &ButtonInput<KeyCode>) -> bool {
    keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight)
}

pub fn switch_view_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut active: ResMut<ActiveView>,
    cameras: Res<ViewCameras>,
    mut camera_query: Query<&mut Camera>,
) {
    // Digit keys are palette slots when Shift is held (P4); letter keys always switch views.
    let next = if keyboard.just_pressed(KeyCode::KeyT) {
        Some(OrthoView::Top)
    } else if keyboard.just_pressed(KeyCode::KeyF) {
        Some(OrthoView::Front)
    } else if keyboard.just_pressed(KeyCode::KeyL) {
        Some(OrthoView::Left)
    } else if !shift_held(&keyboard) && keyboard.just_pressed(KeyCode::Digit1) {
        Some(OrthoView::Top)
    } else if !shift_held(&keyboard) && keyboard.just_pressed(KeyCode::Digit2) {
        Some(OrthoView::Front)
    } else if !shift_held(&keyboard) && keyboard.just_pressed(KeyCode::Digit3) {
        Some(OrthoView::Left)
    } else {
        None
    };

    let Some(next) = next else {
        return;
    };

    if active.0 == next {
        return;
    }

    active.0 = next;

    for (entity, view) in [
        (cameras.top, OrthoView::Top),
        (cameras.front, OrthoView::Front),
        (cameras.left, OrthoView::Left),
    ] {
        if let Ok(mut camera) = camera_query.get_mut(entity) {
            camera.is_active = view == next;
        }
    }
}

#[must_use]
pub fn cell_to_world(pos: boxes_sim::WorldPos) -> Vec3 {
    Vec3::new(
        pos.x as f32 - WORLD_CENTER,
        pos.y as f32 - WORLD_CENTER,
        pos.z as f32 - WORLD_CENTER,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shift_held_when_either_shift_key_pressed() {
        let mut keyboard = ButtonInput::<KeyCode>::default();
        assert!(!shift_held(&keyboard));
        keyboard.press(KeyCode::ShiftLeft);
        assert!(shift_held(&keyboard));
    }
}
