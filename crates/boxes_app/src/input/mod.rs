//! Mouse and keyboard tools for grid editing.

mod pick;
mod selection;
mod tools;

pub use pick::{
    cell_to_uv_f, cursor_delta_to_uv, pick_slice_cell, pick_surface_cell,
    selection_in_viewport, world_from_uv_on_slice,
};
#[allow(unused_imports)]
pub use pick::pick_surface_at_uv;
pub use selection::{
    apply_mmb_uv_delta, finish_mmb_pan, orbit_look_at_uv, pan_orbit_anchor, random_selection,
    recenter_on_selection, screen_dir_from_uv_delta, set_selection, slice_depth,
    LastSelectionMove, MmbPanState, OrbitAnchor, SelectedCell,
};
pub use tools::{ActiveTool, PalettePreset, ToolState};
pub use tools::{direction_label, palette_slot_label, reduce_mode_label};

use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use boxes_sim::{Cell, WorldPos};

use crate::render::{ActiveView, GridCamera, GridCameraEntity, PendingChunkRebuilds, ScreenDir, ViewCameraState};
use crate::sim_bridge::{queue_rebuild_for_positions, GridSimulation};

/// Input plugin: picking, placement tools, slice offset, inspect.
pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ToolState>()
            .init_resource::<SelectedCell>()
            .init_resource::<OrbitAnchor>()
            .init_resource::<LastSelectionMove>()
            .init_resource::<MmbPanState>()
            .add_systems(
                Update,
                (
                    tool_keyboard_system,
                    keyboard_nav_system,
                    pan_keyboard_system,
                    recenter_system,
                    slice_keyboard_system,
                    zoom_keyboard_system,
                    wheel_input_system,
                    pointer_select_system,
                    auto_pan_system,
                    pan_pointer_system,
                    pointer_tool_system,
                )
                    .chain(),
            );
    }
}

fn ctrl_held(keyboard: &ButtonInput<KeyCode>) -> bool {
    keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight)
}

fn shift_held(keyboard: &ButtonInput<KeyCode>) -> bool {
    keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight)
}

fn viewport_aspect(window: &Window) -> f32 {
    let h = window.height().max(1.0);
    window.width() / h
}

fn arrow_screen_dir(keyboard: &ButtonInput<KeyCode>) -> Option<ScreenDir> {
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

    if shift_held(&keyboard) && !ctrl_held(&keyboard) {
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

fn keyboard_nav_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    active: Res<ActiveView>,
    mut selection: ResMut<SelectedCell>,
    mut last_move: ResMut<LastSelectionMove>,
) {
    if ctrl_held(&keyboard) || shift_held(&keyboard) {
        return;
    }

    let Some(dir) = arrow_screen_dir(&keyboard) else {
        return;
    };

    let next = active.pose.nudge_screen(selection.pos, dir);
    set_selection(&mut selection, next);
    last_move.dir = Some(dir);
}

fn pan_keyboard_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    active: Res<ActiveView>,
    selection: Res<SelectedCell>,
    mut anchor: ResMut<OrbitAnchor>,
    camera: Res<ViewCameraState>,
) {
    if !shift_held(&keyboard) || ctrl_held(&keyboard) {
        return;
    }

    let Some(dir) = arrow_screen_dir(&keyboard) else {
        return;
    };

    let Ok(window) = windows.single() else {
        return;
    };

    let depth = slice_depth(active.pose, &selection);
    pan_orbit_anchor(
        active.pose,
        &mut anchor,
        depth,
        dir,
        &camera,
        viewport_aspect(window),
    );
}

fn recenter_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    selection: Res<SelectedCell>,
    mut anchor: ResMut<OrbitAnchor>,
) {
    if keyboard.just_pressed(KeyCode::KeyC) {
        recenter_on_selection(&mut anchor, &selection);
    }
}

fn slice_depth_delta(keyboard: &ButtonInput<KeyCode>) -> Option<i16> {
    if keyboard.just_pressed(KeyCode::BracketLeft) {
        Some(-1)
    } else if keyboard.just_pressed(KeyCode::BracketRight) {
        Some(1)
    } else {
        None
    }
}

fn slice_keyboard_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    active: Res<ActiveView>,
    mut selection: ResMut<SelectedCell>,
) {
    if ctrl_held(&keyboard) {
        return;
    }
    let Some(delta) = slice_depth_delta(&keyboard) else {
        return;
    };
    let steps = active.pose.depth_step_delta(delta);
    let next = active.pose.nudge_depth(selection.pos, steps);
    set_selection(&mut selection, next);
}

fn zoom_keyboard_system(keyboard: Res<ButtonInput<KeyCode>>, mut camera: ResMut<ViewCameraState>) {
    if !ctrl_held(&keyboard) {
        return;
    }
    if keyboard.just_pressed(KeyCode::BracketLeft) {
        camera.nudge_zoom(-4.0);
    } else if keyboard.just_pressed(KeyCode::BracketRight) {
        camera.nudge_zoom(4.0);
    }
}

fn wheel_input_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut wheel_events: EventReader<MouseWheel>,
    active: Res<ActiveView>,
    mut selection: ResMut<SelectedCell>,
    mut camera: ResMut<ViewCameraState>,
) {
    let mut delta = 0.0f32;
    for event in wheel_events.read() {
        delta += event.y;
    }
    if delta == 0.0 {
        return;
    }

    if ctrl_held(&keyboard) {
        camera.nudge_zoom(delta * 2.0);
        return;
    }

    let forward_steps = delta.signum() as i16;
    let steps = active.pose.depth_step_delta(forward_steps);
    let next = active.pose.nudge_depth(selection.pos, steps);
    set_selection(&mut selection, next);
}

#[allow(clippy::too_many_arguments)]
fn pointer_select_system(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    active: Res<ActiveView>,
    camera_entity: Res<GridCameraEntity>,
    camera_query: Query<(&Camera, &GlobalTransform), With<GridCamera>>,
    mut selection: ResMut<SelectedCell>,
    mut last_move: ResMut<LastSelectionMove>,
    mut last_uv: Local<Option<(f32, f32)>>,
) {
    let select_held = mouse.pressed(MouseButton::Left);
    let select_pressed = mouse.just_pressed(MouseButton::Left);
    if !select_held && !select_pressed {
        *last_uv = None;
        return;
    }

    let Ok(window) = windows.single() else {
        return;
    };

    let Ok((camera, transform)) = camera_query.get(camera_entity.0) else {
        return;
    };

    let Some((origin, direction)) = cursor_ray(window, camera, transform) else {
        return;
    };

    let depth = slice_depth(active.pose, &selection);
    let Some(pos) = pick_slice_cell(active.pose, depth, origin, direction) else {
        return;
    };

    if select_pressed || (select_held && selection.pos != pos) {
        let prev_uv = last_uv.or_else(|| Some(cell_to_uv_f(active.pose, selection.pos)));
        set_selection(&mut selection, pos);
        if let Some((pu, pv)) = prev_uv {
            let (nu, nv) = cell_to_uv_f(active.pose, pos);
            last_move.dir = screen_dir_from_uv_delta(active.pose, nu - pu, nv - pv);
        }
        *last_uv = Some(cell_to_uv_f(active.pose, pos));
    }
}

fn auto_pan_system(
    windows: Query<&Window, With<PrimaryWindow>>,
    active: Res<ActiveView>,
    selection: Res<SelectedCell>,
    mut anchor: ResMut<OrbitAnchor>,
    camera: Res<ViewCameraState>,
    last_move: Res<LastSelectionMove>,
) {
    if !selection.is_changed() {
        return;
    }

    let Ok(window) = windows.single() else {
        return;
    };

    let pose = active.pose;
    let aspect = viewport_aspect(window);
    let anchor_uv = orbit_look_at_uv(pose, &anchor, &MmbPanState::default());

    if selection_in_viewport(pose, selection.pos, anchor_uv, camera.zoom_cells, aspect) {
        return;
    }

    let Some(dir) = last_move.dir else {
        return;
    };

    let depth = slice_depth(pose, &selection);
    pan_orbit_anchor(pose, &mut anchor, depth, dir, &camera, aspect);
}

#[allow(clippy::too_many_arguments)]
fn pan_pointer_system(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    active: Res<ActiveView>,
    selection: Res<SelectedCell>,
    mut anchor: ResMut<OrbitAnchor>,
    mut mmb: ResMut<MmbPanState>,
    camera: Res<ViewCameraState>,
    mut last_cursor: Local<Option<Vec2>>,
) {
    if mouse.just_pressed(MouseButton::Middle) {
        mmb.dragging = true;
        mmb.anchor_uv_start = cell_to_uv_f(active.pose, anchor.pos);
        mmb.uv_offset = (0.0, 0.0);
        if let Ok(window) = windows.single() {
            *last_cursor = window.cursor_position();
        }
        return;
    }

    if mouse.just_released(MouseButton::Middle) {
        let depth = slice_depth(active.pose, &selection);
        finish_mmb_pan(&mut anchor, &mut mmb, active.pose, depth);
        *last_cursor = None;
        return;
    }

    if !mouse.pressed(MouseButton::Middle) || !mmb.dragging {
        return;
    }

    let Ok(window) = windows.single() else {
        return;
    };

    let Some(cursor) = window.cursor_position() else {
        return;
    };

    if let Some(prev) = *last_cursor {
        let delta = cursor - prev;
        let (du, dv) = cursor_delta_to_uv(
            active.pose,
            delta,
            camera.zoom_cells,
            window.height(),
        );
        apply_mmb_uv_delta(&mut mmb, du, dv);
    }
    *last_cursor = Some(cursor);
}

#[allow(clippy::too_many_arguments)]
fn pointer_tool_system(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    active: Res<ActiveView>,
    camera_entity: Res<GridCameraEntity>,
    camera_query: Query<(&Camera, &GlobalTransform), With<GridCamera>>,
    tools: Res<ToolState>,
    selection: Res<SelectedCell>,
    mut sim: ResMut<GridSimulation>,
    mut pending: ResMut<PendingChunkRebuilds>,
    mut last_drag_pos: Local<Option<WorldPos>>,
) {
    let apply_pressed = mouse.just_pressed(MouseButton::Right);
    let apply_held = mouse.pressed(MouseButton::Right);

    if !apply_pressed && !apply_held {
        *last_drag_pos = None;
        return;
    }

    let Ok(window) = windows.single() else {
        return;
    };

    let Ok((camera, transform)) = camera_query.get(camera_entity.0) else {
        return;
    };

    let Some((origin, direction)) = cursor_ray(window, camera, transform) else {
        return;
    };

    let pose = active.pose;
    let depth = slice_depth(pose, &selection);

    let target = match tools.active {
        ActiveTool::Erase => pick_surface_cell(&sim.0, pose, depth, origin, direction),
        ActiveTool::Place => pick_slice_cell(pose, depth, origin, direction),
        ActiveTool::Inspect => pick_surface_cell(&sim.0, pose, depth, origin, direction),
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
            queue_rebuild_for_positions(&[pos], pose.face(), &mut pending);
        }
        ActiveTool::Place => {
            let cell = tools.selected_preset().to_cell();
            sim.0.world.set(pos, cell);
            sim.0.mark_dirty(pos);
            queue_rebuild_for_positions(&[pos], pose.face(), &mut pending);
        }
        ActiveTool::Inspect => {
            let cell = sim.0.world.get(pos);
            info!(
                "inspect ({}, {}, {}): type={} state={} flags={}",
                pos.x, pos.y, pos.z, cell.type_id, cell.state, cell.flags
            );
        }
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
            .insert_resource(ActiveView::default())
            .insert_resource(GridCameraEntity(Entity::PLACEHOLDER));
    }

    #[test]
    fn app_with_input_plugin_builds() {
        App::new().add_plugins(BoxesAppPlugin);
    }
}
