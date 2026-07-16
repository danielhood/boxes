//! Bevy application shell for Boxes — window, orthographic camera, placeholder scene.

use bevy::prelude::*;
use bevy::render::camera::{OrthographicProjection, Projection, ScalingMode};

/// Root plugin: clear color, orthographic viewport, and a minimal placeholder scene.
pub struct BoxesAppPlugin;

impl Plugin for BoxesAppPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::srgb(0.08, 0.09, 0.11)))
            .add_systems(Startup, setup_scene);
    }
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Top-down orthographic view (placeholder for view switching in P3/P4).
    commands.spawn((
        Camera3d::default(),
        Projection::from(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: 20.0,
            },
            ..OrthographicProjection::default_3d()
        }),
        Transform::from_xyz(0.0, 30.0, 0.0).looking_at(Vec3::ZERO, Vec3::NEG_Z),
    ));

    commands.spawn(DirectionalLight {
        illuminance: light_consts::lux::OVERCAST_DAY,
        ..default()
    });

    // Placeholder cube — chunked GPU instancing replaces this in P3.
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.45, 0.55, 0.75))),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    // Touch the sim stub so the workspace link stays wired until P1.
    let _ = boxes_sim::STUB;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plugin_builds_without_panic() {
        App::new().add_plugins(BoxesAppPlugin);
    }
}
