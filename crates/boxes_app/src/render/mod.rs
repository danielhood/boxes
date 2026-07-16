//! Chunked grid rendering for Boxes.

mod chunk;
mod materials;
mod surface;
mod view;

pub use chunk::{ChunkRenderCache, PendingChunkRebuilds};
pub use surface::{affected_chunks, visible_surface};
pub use view::{ActiveView, GridCamera, OrthoView, ViewCameras, WORLD_CENTER};

use bevy::prelude::*;

use chunk::{mark_view_change, queue_initial_rebuild, rebuild_chunk_instances};
use materials::GridMaterials;
use view::{setup_cameras, switch_view_system};

/// Rendering plugin: orthographic views, GPU-instanced chunk draws, sim bridge.
pub struct GridRenderPlugin;

impl Plugin for GridRenderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChunkRenderCache>()
            .init_resource::<PendingChunkRebuilds>()
            .init_resource::<ActiveView>()
            .add_systems(
                Startup,
                (GridMaterials::setup, setup_cameras, queue_initial_rebuild),
            )
            .add_systems(
                Update,
                (
                    switch_view_system,
                    mark_view_change,
                    rebuild_chunk_instances,
                )
                    .chain(),
            );
    }
}
