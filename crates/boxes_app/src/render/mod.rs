//! Chunked grid rendering for Boxes.

mod chunk;
mod materials;
mod surface;
mod view;

pub use chunk::{ChunkRenderCache, PendingChunkRebuilds};
pub use surface::{affected_chunks, visible_surface};
#[allow(unused_imports)]
pub use surface::unclipped_slice_depth;
pub use view::{
    ActiveView, GridCamera, GridCameraEntity, OrthoView, ViewCameraState, WORLD_CENTER,
};

use bevy::prelude::*;

use chunk::{mark_selection_depth_change, mark_view_change, queue_initial_rebuild, rebuild_chunk_instances};
use materials::GridMaterials;
use view::{apply_camera_framing, setup_cameras, snap_top_view_system, view_rotate_system};

/// Rendering plugin: orthographic views, GPU-instanced chunk draws, sim bridge.
pub struct GridRenderPlugin;

impl Plugin for GridRenderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChunkRenderCache>()
            .init_resource::<PendingChunkRebuilds>()
            .init_resource::<ActiveView>()
            .init_resource::<ViewCameraState>()
            .add_systems(
                Startup,
                (GridMaterials::setup, setup_cameras, queue_initial_rebuild),
            )
            .add_systems(
                Update,
                (
                    view_rotate_system,
                    snap_top_view_system,
                    mark_view_change,
                    mark_selection_depth_change,
                    apply_camera_framing,
                    rebuild_chunk_instances,
                )
                    .chain(),
            );
    }
}
