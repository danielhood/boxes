//! White inner-border highlight for the selected cell.

use bevy::prelude::*;

use crate::input::SelectedCell;

use super::view::cell_to_world;

/// Slightly larger than the 0.95 cell mesh so the border is not depth-occluded.
const HIGHLIGHT_SIZE: f32 = 1.0;
/// Border bar thickness.
const BORDER: f32 = 0.05;

#[derive(Component)]
pub struct SelectionHighlight;

#[derive(Resource)]
pub struct SelectionHighlightEntity(pub Entity);

pub fn setup_selection_highlight(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let highlight_mat = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        unlit: true,
        depth_bias: 2.0,
        ..default()
    });

    let inner = HIGHLIGHT_SIZE - BORDER * 2.0;
    let offset = HIGHLIGHT_SIZE / 2.0 - BORDER / 2.0;

    let x_bar = meshes.add(Cuboid::new(inner, BORDER, BORDER));
    let y_bar = meshes.add(Cuboid::new(BORDER, inner, BORDER));
    let z_bar = meshes.add(Cuboid::new(BORDER, BORDER, inner));

    let signs = [-1.0_f32, 1.0];

    let entity = commands
        .spawn((
            SelectionHighlight,
            Transform::default(),
            Visibility::default(),
        ))
        .with_children(|parent| {
            for &y in &signs {
                for &z in &signs {
                    parent.spawn((
                        Mesh3d(x_bar.clone()),
                        MeshMaterial3d(highlight_mat.clone()),
                        Transform::from_xyz(0.0, y * offset, z * offset),
                    ));
                }
            }
            for &x in &signs {
                for &z in &signs {
                    parent.spawn((
                        Mesh3d(y_bar.clone()),
                        MeshMaterial3d(highlight_mat.clone()),
                        Transform::from_xyz(x * offset, 0.0, z * offset),
                    ));
                }
            }
            for &x in &signs {
                for &y in &signs {
                    parent.spawn((
                        Mesh3d(z_bar.clone()),
                        MeshMaterial3d(highlight_mat.clone()),
                        Transform::from_xyz(x * offset, y * offset, 0.0),
                    ));
                }
            }
        })
        .id();

    commands.insert_resource(SelectionHighlightEntity(entity));
}

pub fn sync_selection_highlight(
    selection: Res<SelectedCell>,
    highlight: Res<SelectionHighlightEntity>,
    mut transforms: Query<&mut Transform, With<SelectionHighlight>>,
) {
    let Ok(mut transform) = transforms.get_mut(highlight.0) else {
        return;
    };
    transform.translation = cell_to_world(selection.pos);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn highlight_frame_larger_than_cell_mesh() {
        assert!(HIGHLIGHT_SIZE > 0.95);
        assert!(BORDER > 0.0);
    }
}
