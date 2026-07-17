//! Per-chunk GPU instance entities and incremental rebuild.

use std::collections::{HashMap, HashSet};

use bevy::prelude::*;
use boxes_sim::{Cell, ChunkCoord, WorldPos};

use super::materials::GridMaterials;
use super::surface::{surface_cells_for_chunk, visible_surface};
use super::view::{cell_to_world, ActiveView, OrthoView};
use crate::input::{slice_depth, SelectedCell};

/// Instance cube spawned for one visible surface cell.
#[derive(Component)]
pub struct GridInstance {
    #[allow(dead_code)]
    pub chunk: ChunkCoord,
}

/// Tracks spawned instance entities per chunk.
#[derive(Resource, Default)]
pub struct ChunkRenderCache {
    pub instances: HashMap<ChunkCoord, Vec<Entity>>,
    pub rebuilt_chunks: HashSet<ChunkCoord>,
}

/// Chunks queued for instance buffer rebuild.
#[derive(Resource, Default)]
pub struct PendingChunkRebuilds {
    pub chunks: HashSet<ChunkCoord>,
    pub rebuild_all: bool,
}

impl PendingChunkRebuilds {
    pub fn mark_dirty(&mut self, coords: impl IntoIterator<Item = ChunkCoord>) {
        self.chunks.extend(coords);
    }

    pub fn mark_all(&mut self) {
        self.rebuild_all = true;
    }
}

pub fn queue_initial_rebuild(mut pending: ResMut<PendingChunkRebuilds>) {
    pending.mark_all();
}

pub fn mark_view_change(
    active: Res<ActiveView>,
    mut last_view: Local<Option<OrthoView>>,
    mut pending: ResMut<PendingChunkRebuilds>,
) {
    if *last_view != Some(active.0) {
        *last_view = Some(active.0);
        pending.mark_all();
    }
}

pub fn mark_selection_depth_change(
    active: Res<ActiveView>,
    selection: Res<SelectedCell>,
    mut last_depth: Local<Option<u16>>,
    mut pending: ResMut<PendingChunkRebuilds>,
) {
    let depth = slice_depth(active.0, &selection);
    if *last_depth != Some(depth) {
        *last_depth = Some(depth);
        pending.mark_all();
    }
}

pub fn rebuild_chunk_instances(
    mut commands: Commands,
    active: Res<ActiveView>,
    selection: Res<SelectedCell>,
    materials: Res<GridMaterials>,
    sim: Res<crate::GridSimulation>,
    mut cache: ResMut<ChunkRenderCache>,
    mut pending: ResMut<PendingChunkRebuilds>,
) {
    if !pending.rebuild_all && pending.chunks.is_empty() {
        return;
    }

    let slice_depth = slice_depth(active.0, &selection);
    let surface = visible_surface(&sim.0, active.0, slice_depth);

    let targets: HashSet<ChunkCoord> = if pending.rebuild_all {
        surface
            .values()
            .map(|(pos, _)| pos.chunk_coord())
            .collect()
    } else {
        pending.chunks.drain().collect()
    };
    pending.rebuild_all = false;

    cache.rebuilt_chunks.clear();

    for coord in targets {
        if let Some(old) = cache.instances.remove(&coord) {
            for entity in old {
                commands.entity(entity).despawn();
            }
        }

        let cells = surface_cells_for_chunk(&surface, coord);
        if cells.is_empty() {
            continue;
        }

        let mut entities = Vec::with_capacity(cells.len());
        for (pos, cell) in cells {
            let entity = spawn_instance(&mut commands, &materials, coord, pos, cell);
            entities.push(entity);
        }

        cache.instances.insert(coord, entities);
        cache.rebuilt_chunks.insert(coord);
    }
}

fn spawn_instance(
    commands: &mut Commands,
    materials: &GridMaterials,
    chunk: ChunkCoord,
    pos: WorldPos,
    cell: Cell,
) -> Entity {
    commands
        .spawn((
            Mesh3d(materials.mesh.clone()),
            MeshMaterial3d(materials.material_for(cell.type_id)),
            Transform::from_translation(cell_to_world(pos)),
            GridInstance { chunk },
        ))
        .id()
}

#[cfg(test)]
mod tests {
    use super::*;
    use boxes_sim::{make_generator, Simulation};

    use super::super::surface::unclipped_slice_depth;

    #[test]
    fn surface_cells_grouped_by_chunk() {
        let mut sim = Simulation::new();
        sim.world
            .set(WorldPos::new(33, 40, 33), make_generator(20, 1));
        let surface = visible_surface(&sim, OrthoView::Top, unclipped_slice_depth(OrthoView::Top));
        let coord = WorldPos::new(33, 40, 33).chunk_coord();
        let cells = surface_cells_for_chunk(&surface, coord);
        assert_eq!(cells.len(), 1);
    }
}
