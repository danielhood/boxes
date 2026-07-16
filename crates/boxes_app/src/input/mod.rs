//! Mouse and keyboard tools for grid editing.

mod pick;
mod tools;

pub use pick::{pick_slice_cell, pick_surface_cell};
#[allow(unused_imports)]
pub use pick::pick_surface_at_uv;
pub use tools::{slice_nudge_delta, ActiveTool, InspectedCell, PalettePreset, ToolState, ViewSlice};

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use boxes_sim::{Cell, WorldPos};

use crate::render::{ActiveView, GridCamera, OrthoView, PendingChunkRebuilds, ViewCameras};
use crate::sim_bridge::{queue_rebuild_for_positions, GridSimulation};

/// Input plugin: picking, placement tools, slice offset, inspect.
pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ToolState>()
            .init_resource::<ViewSlice>()
            .init_resource::<InspectedCell>()
            .add_systems(
                Update,
                (
                    tool_keyboard_system,
                    slice_keyboard_system,
                    pointer_tool_system.after(tool_keyboard_system),
                ),
            );
    }
}

fn active_camera_entity(view: OrthoView, cameras: &ViewCameras) -> Entity {
    match view {
        OrthoView::Top => cameras.top,
        OrthoView::Front => cameras.front,
        OrthoView::Left => cameras.left,
    }
}

fn cursor_ray<'a>(
    window: &Window,
    camera: &'a Camera,
    camera_transform: &'a GlobalTransform,
) -> Option<(Vec3, Vec3)> {
    let cursor = window.cursor_position()?;
    let ray = camera.viewport_to_world(camera_transform, cursor).ok()?;
    Some((ray.origin, ray.direction.normalize()))
}

fn tool_keyboard_system(keyboard: Res<ButtonInput<KeyCode>>, mut tools: ResMut<ToolState>) {
    if keyboard.just_pressed(KeyCode::KeyE) {
        tools.active = ActiveTool::Erase;
    }
    if keyboard.just_pressed(KeyCode::KeyP) {
        tools.active = ActiveTool::Place;
    }
    if keyboard.just_pressed(KeyCode::KeyI) {
        tools.active = ActiveTool::Inspect;
    }

    if keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight) {
        let slot = if keyboard.just_pressed(KeyCode::Digit1) {
            Some(0)
        } else if keyboard.just_pressed(KeyCode::Digit2) {
            Some(1)
        } else if keyboard.just_pressed(KeyCode::Digit3) {
            Some(2)
        } else if keyboard.just_pressed(KeyCode::Digit4) {
            Some(3)
        } else if keyboard.just_pressed(KeyCode::Digit5) {
            Some(4)
        } else if keyboard.just_pressed(KeyCode::Digit6) {
            Some(5)
        } else if keyboard.just_pressed(KeyCode::Digit7) {
            Some(6)
        } else if keyboard.just_pressed(KeyCode::Digit8) {
            Some(7)
        } else if keyboard.just_pressed(KeyCode::Digit9) {
            Some(8)
        } else {
            None
        };
        if let Some(slot) = slot {
            tools.selected_slot = slot;
            tools.active = ActiveTool::Place;
        }
    }
}

fn slice_keyboard_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    active: Res<ActiveView>,
    mut slice: ResMut<ViewSlice>,
) {
    let Some(delta) = slice_nudge_delta(&keyboard) else {
        return;
    };

    let view = active.0;
    let next = slice.nudge(view, delta);
    slice.set_depth(view, next);
    info!(
        "depth slice {}={} ({})",
        ViewSlice::depth_axis_label(view),
        next,
        view.label()
    );
}

#[allow(clippy::too_many_arguments)]
fn pointer_tool_system(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    active: Res<ActiveView>,
    cameras: Res<ViewCameras>,
    camera_query: Query<(&Camera, &GlobalTransform), With<GridCamera>>,
    tools: Res<ToolState>,
    slice: Res<ViewSlice>,
    mut sim: ResMut<GridSimulation>,
    mut pending: ResMut<PendingChunkRebuilds>,
    mut inspected: ResMut<InspectedCell>,
    mut last_drag_pos: Local<Option<WorldPos>>,
) {
    let Ok(window) = windows.single() else {
        return;
    };

    let camera_entity = active_camera_entity(active.0, &cameras);
    let Ok((camera, transform)) = camera_query.get(camera_entity) else {
        return;
    };

    let Some((origin, direction)) = cursor_ray(window, camera, transform) else {
        return;
    };

    let inspect_pressed = mouse.just_pressed(MouseButton::Right);
    let apply_pressed = mouse.just_pressed(MouseButton::Left);
    let apply_held = mouse.pressed(MouseButton::Left);

    if !inspect_pressed && !apply_pressed && !apply_held {
        *last_drag_pos = None;
        return;
    }

    let view = active.0;
    let depth = slice.depth(view);

    if inspect_pressed || tools.active == ActiveTool::Inspect && apply_pressed {
        if let Some(pos) = pick_surface_cell(&sim.0, view, depth, origin, direction) {
            let cell = sim.0.world.get(pos);
            inspected.pos = Some(pos);
            inspected.cell = cell;
            info!(
                "inspect ({}, {}, {}): type={} state={} flags={}",
                pos.x, pos.y, pos.z, cell.type_id, cell.state, cell.flags
            );
        }
        return;
    }

    let should_apply = if apply_pressed {
        true
    } else if apply_held {
        // Drag placement: apply when cursor moves to a new cell.
        true
    } else {
        false
    };

    if !should_apply {
        return;
    }

    let target = match tools.active {
        ActiveTool::Erase => pick_surface_cell(&sim.0, view, depth, origin, direction),
        // Place at the current depth slice (not the visible surface).
        ActiveTool::Place => pick_slice_cell(view, depth, origin, direction),
        ActiveTool::Inspect => None,
    };

    let Some(pos) = target else {
        return;
    };

    if apply_held && !apply_pressed && *last_drag_pos == Some(pos) {
        return;
    }
    *last_drag_pos = Some(pos);

    match tools.active {
        ActiveTool::Erase => {
            if sim.0.world.get(pos).is_empty() {
                return;
            }
            sim.0.world.set(pos, Cell::empty());
            queue_rebuild_for_positions(&[pos], view, &mut pending);
        }
        ActiveTool::Place => {
            let cell = tools.selected_preset().to_cell();
            sim.0.world.set(pos, cell);
            sim.0.mark_dirty(pos);
            queue_rebuild_for_positions(&[pos], view, &mut pending);
        }
        ActiveTool::Inspect => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BoxesAppPlugin;

    #[test]
    fn input_plugin_builds_without_panic() {
        App::new()
            .add_plugins(InputPlugin)
            .insert_resource(ActiveView(crate::render::OrthoView::Top))
            .insert_resource(ViewCameras {
                top: Entity::PLACEHOLDER,
                front: Entity::PLACEHOLDER,
                left: Entity::PLACEHOLDER,
            });
    }

    #[test]
    fn app_with_input_plugin_builds() {
        App::new().add_plugins(BoxesAppPlugin);
    }
}
