//! Shared cube mesh and per-type materials.

use bevy::prelude::*;
use boxes_sim::{TYPE_AGGREGATOR, TYPE_GENERATOR, TYPE_TRANSFORMER};

/// Brightness multiplier for cells below the active slice plane.
pub const BELOW_SLICE_BRIGHTNESS: f32 = 0.38;

/// Shared mesh and materials for automatic GPU instancing.
#[derive(Resource)]
pub struct GridMaterials {
    pub mesh: Handle<Mesh>,
    pub generator: Handle<StandardMaterial>,
    pub transformer: Handle<StandardMaterial>,
    pub aggregator: Handle<StandardMaterial>,
    pub default: Handle<StandardMaterial>,
    pub generator_dim: Handle<StandardMaterial>,
    pub transformer_dim: Handle<StandardMaterial>,
    pub aggregator_dim: Handle<StandardMaterial>,
    pub default_dim: Handle<StandardMaterial>,
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
            generator: materials.add(unlit_color(0.92, 0.55, 0.18)),
            transformer: materials.add(unlit_color(0.28, 0.68, 0.95)),
            aggregator: materials.add(unlit_color(0.55, 0.86, 0.38)),
            default: materials.add(unlit_color(0.45, 0.55, 0.75)),
            generator_dim: materials.add(unlit_color(
                0.92 * BELOW_SLICE_BRIGHTNESS,
                0.55 * BELOW_SLICE_BRIGHTNESS,
                0.18 * BELOW_SLICE_BRIGHTNESS,
            )),
            transformer_dim: materials.add(unlit_color(
                0.28 * BELOW_SLICE_BRIGHTNESS,
                0.68 * BELOW_SLICE_BRIGHTNESS,
                0.95 * BELOW_SLICE_BRIGHTNESS,
            )),
            aggregator_dim: materials.add(unlit_color(
                0.55 * BELOW_SLICE_BRIGHTNESS,
                0.86 * BELOW_SLICE_BRIGHTNESS,
                0.38 * BELOW_SLICE_BRIGHTNESS,
            )),
            default_dim: materials.add(unlit_color(
                0.45 * BELOW_SLICE_BRIGHTNESS,
                0.55 * BELOW_SLICE_BRIGHTNESS,
                0.75 * BELOW_SLICE_BRIGHTNESS,
            )),
        };
        commands.insert_resource(grid);
    }

    #[must_use]
    pub fn material_for(&self, type_id: u8, on_slice: bool) -> Handle<StandardMaterial> {
        match (type_id, on_slice) {
            (TYPE_GENERATOR, true) => self.generator.clone(),
            (TYPE_GENERATOR, false) => self.generator_dim.clone(),
            (TYPE_TRANSFORMER, true) => self.transformer.clone(),
            (TYPE_TRANSFORMER, false) => self.transformer_dim.clone(),
            (TYPE_AGGREGATOR, true) => self.aggregator.clone(),
            (TYPE_AGGREGATOR, false) => self.aggregator_dim.clone(),
            (_, true) => self.default.clone(),
            (_, false) => self.default_dim.clone(),
        }
    }
}

fn unlit_color(r: f32, g: f32, b: f32) -> StandardMaterial {
    StandardMaterial {
        base_color: Color::srgb(r, g, b),
        unlit: true,
        ..default()
    }
}
