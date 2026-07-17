//! Shared cube mesh and per-type materials.

use bevy::prelude::*;
use boxes_sim::{TYPE_AGGREGATOR, TYPE_GENERATOR, TYPE_TRANSFORMER};

/// Shared mesh and materials for automatic GPU instancing.
#[derive(Resource)]
pub struct GridMaterials {
    pub mesh: Handle<Mesh>,
    pub generator: Handle<StandardMaterial>,
    pub transformer: Handle<StandardMaterial>,
    pub aggregator: Handle<StandardMaterial>,
    pub default: Handle<StandardMaterial>,
}

impl GridMaterials {
    pub fn setup(
        mut commands: Commands,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<StandardMaterial>>,
    ) {
        let mesh = meshes.add(Cuboid::new(0.95, 0.95, 0.95));
        let grid = GridMaterials {
            mesh,
            generator: materials.add(StandardMaterial {
                base_color: Color::srgb(0.92, 0.55, 0.18),
                unlit: true,
                ..default()
            }),
            transformer: materials.add(StandardMaterial {
                base_color: Color::srgb(0.28, 0.68, 0.95),
                unlit: true,
                ..default()
            }),
            aggregator: materials.add(StandardMaterial {
                base_color: Color::srgb(0.55, 0.86, 0.38),
                unlit: true,
                ..default()
            }),
            default: materials.add(StandardMaterial {
                base_color: Color::srgb(0.45, 0.55, 0.75),
                unlit: true,
                ..default()
            }),
        };
        commands.insert_resource(grid);
    }

    #[must_use]
    pub fn material_for(&self, type_id: u8) -> Handle<StandardMaterial> {
        match type_id {
            TYPE_GENERATOR => self.generator.clone(),
            TYPE_TRANSFORMER => self.transformer.clone(),
            TYPE_AGGREGATOR => self.aggregator.clone(),
            _ => self.default.clone(),
        }
    }
}
