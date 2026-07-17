//! Mouse and keyboard tools for grid editing.

mod pick;
mod selection;
mod tools;

pub use pick::{pick_slice_cell, pick_surface_cell};
#[allow(unused_imports)]
pub use pick::pick_surface_at_uv;
pub use selection::{random_selection, set_selection, slice_depth, SelectedCell};
pub use tools::{ActiveTool, PalettePreset, ToolState};
pub use tools::{direction_label, palette_slot_label, reduce_mode_label};

use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use boxes_sim::{Cell, WorldPos};

use crate::render::{
    ActiveView, GridCamera, GridCameraEntity, PendingChunkRebuilds, ViewCameraState,
};
use crate::sim_bridge::{queue_rebuild_for_positions, GridSimulation};

/// Input plugin: picking, placement tools, slice offset, inspect.
pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ToolState>()
            .init_resource::<SelectedCell>()
            .add_systems(
                Update,
                (
                    tool_keyboard_system,
                    keyboard_nav_system,
                    slice_keyboard_system,
                    zoom_keyboard_system,
                    wheel_input_system,
                    pointer_select_system,
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

    if shift_held(&keyboard) {
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
) {
    if ctrl_held(&keyboard) {
        return;
    }

    let (du, dv) = if keyboard.just_pressed(KeyCode::ArrowUp) {
        (0, 1)
    } else if keyboard.just_pressed(KeyCode::ArrowDown) {
        (0, -1)
    } else if keyboard.just_pressed(KeyCode::ArrowLeft) {
        (-1, 0)
    } else if keyboard.just_pressed(KeyCode::ArrowRight) {
        (1, 0)
    } else {
        return;
    };

    let next = active.0.nudge_uv(selection.pos, du, dv);
    set_selection(&mut selection, next);
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
    let next = active.0.nudge_depth(selection.pos, delta);
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

    let steps = delta.signum() as i16;
    let next = active.0.nudge_depth(selection.pos, -steps);
    set_selection(&mut selection, next);
}

#[allow(clippy::too_many_arguments)]
fn pointer_select_system(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    active: Res<ActiveView>,
    camera_entity: Res<GridCameraEntity>,
    camera_query: Query<(&Camera, &GlobalTransform), With<GridCamera>>,
    selection: Res<SelectedCell>,
    mut selection_mut: ResMut<SelectedCell>,
    sim: Res<GridSimulation>,
) {
    let select_held = mouse.pressed(MouseButton::Left);
    let select_pressed = mouse.just_pressed(MouseButton::Left);
    if !select_held && !select_pressed {
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

    let depth = slice_depth(active.0, &selection);
    let Some(pos) = pick_surface_cell(&sim.0, active.0, depth, origin, direction) else {
        if select_pressed {
            if let Some(pos) = pick::pick_slice_cell(active.0, depth, origin, direction) {
                set_selection(&mut selection_mut, pos);
            }
        }
        return;
    };

    if select_pressed || (select_held && selection_mut.pos != pos) {
        set_selection(&mut selection_mut, pos);
    }
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

    let view = active.0;
    let depth = slice_depth(view, &selection);

    let target = match tools.active {
        ActiveTool::Erase => pick_surface_cell(&sim.0, view, depth, origin, direction),
        ActiveTool::Place => pick_slice_cell(view, depth, origin, direction),
        ActiveTool::Inspect => pick_surface_cell(&sim.0, view, depth, origin, direction),
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
            .insert_resource(ActiveView(crate::render::OrthoView::Top))
            .insert_resource(GridCameraEntity(Entity::PLACEHOLDER));
    }

    #[test]
    fn app_with_input_plugin_builds() {
        App::new().add_plugins(BoxesAppPlugin);
    }
}
