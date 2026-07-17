//! Factory overlay: type palette, inspector, sim controls, debug readout.

use bevy::prelude::*;
use bevy::render::camera::ClearColorConfig;

use crate::input::{
    palette_slot_label, ActiveTool, InspectedCell, ToolState, ViewSlice,
};
use crate::render::ActiveView;
use crate::sim_bridge::{GridSimulation, SimPlayback, SimTickStats};
use crate::ui::format::{format_inspector, format_throughput};

/// Root marker for the factory UI hierarchy.
#[derive(Component)]
pub struct FactoryUiRoot;

/// Palette slot button — carries palette index.
#[derive(Component, Clone, Copy)]
pub struct PaletteButton(pub usize);

/// Tool mode button.
#[derive(Component, Clone, Copy)]
pub struct ToolButton(pub ActiveTool);

/// Sim control buttons.
#[derive(Component, Clone, Copy)]
pub enum SimControlButton {
    Pause,
    Step,
    Speed,
    DebugOverlay,
}

/// Text panel updated each frame.
#[derive(Component, Clone, Copy)]
pub enum FactoryTextPanel {
    Inspector,
    Throughput,
    DepthSlice,
    DebugOverlay,
}

/// Tracks spawned UI entities for systems that update text.
#[derive(Resource, Default)]
pub struct FactoryUiEntities {
    pub palette_buttons: Vec<Entity>,
}

pub struct FactoryUiPlugin;

impl Plugin for FactoryUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FactoryUiEntities>()
            .add_systems(Startup, setup_factory_ui)
            .add_systems(
                Update,
                (
                    refresh_inspected_cell,
                    palette_button_system,
                    tool_button_system,
                    sim_control_button_system,
                    update_palette_highlight,
                    update_tool_highlight,
                    update_factory_text,
                ),
            );
    }
}

fn base_panel_node() -> Node {
    Node {
        position_type: PositionType::Absolute,
        padding: UiRect::all(Val::Px(8.0)),
        ..default()
    }
}

fn button_node() -> Node {
    Node {
        width: Val::Px(140.0),
        height: Val::Px(28.0),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        border: UiRect::all(Val::Px(1.0)),
        ..default()
    }
}

fn tool_button_node() -> Node {
    Node {
        width: Val::Px(72.0),
        height: Val::Px(28.0),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        border: UiRect::all(Val::Px(1.0)),
        ..default()
    }
}

fn apply_selected_style(bg: &mut BackgroundColor, border: &mut BorderColor, selected: bool) {
    *bg = if selected {
        BackgroundColor(Color::srgb(0.25, 0.35, 0.55))
    } else {
        BackgroundColor(Color::srgba(0.15, 0.16, 0.2, 0.92))
    };
    *border = if selected {
        BorderColor(Color::srgb(0.55, 0.7, 0.95))
    } else {
        BorderColor(Color::srgb(0.35, 0.38, 0.45))
    };
}

fn spawn_text(
    parent: &mut ChildSpawnerCommands,
    content: &str,
    font_size: f32,
    extra: impl Bundle,
) {
    parent.spawn((
        Text::new(content),
        TextFont {
            font_size,
            ..default()
        },
        TextColor(Color::srgb(0.92, 0.93, 0.95)),
        extra,
    ));
}

fn spawn_button(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    marker: impl Bundle,
) {
    parent
        .spawn((
            Button,
            button_node(),
            BackgroundColor(Color::srgba(0.15, 0.16, 0.2, 0.92)),
            BorderColor(Color::srgb(0.35, 0.38, 0.45)),
            marker,
        ))
        .with_children(|btn| {
            spawn_text(btn, label, 14.0, ());
        });
}

fn setup_factory_ui(mut commands: Commands, tools: Res<ToolState>, mut entities: ResMut<FactoryUiEntities>) {
    // Dedicated overlay camera — grid cameras share order 0, so UI would otherwise bind to
    // whichever 3D camera spawned last (left) and only render when that view is active.
    commands.spawn((
        Camera2d,
        Camera {
            order: 100,
            clear_color: ClearColorConfig::None,
            ..default()
        },
        IsDefaultUiCamera,
    ));

    commands
        .spawn((
            FactoryUiRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
        ))
        .with_children(|root| {
            // Left column: tools + type palette
            let mut left_panel = base_panel_node();
            left_panel.flex_direction = FlexDirection::Column;
            left_panel.row_gap = Val::Px(8.0);
            left_panel.top = Val::Px(12.0);
            left_panel.left = Val::Px(12.0);
            root.spawn((
                left_panel,
                BackgroundColor(Color::srgba(0.08, 0.09, 0.11, 0.85)),
            ))
            .with_children(|left| {
                spawn_text(left, "Tool", 16.0, ());
                let tool_row = Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(4.0),
                    ..default()
                };
                left.spawn(tool_row).with_children(|tools_row| {
                    for tool in [ActiveTool::Place, ActiveTool::Erase, ActiveTool::Inspect] {
                        tools_row
                            .spawn((
                                Button,
                                tool_button_node(),
                                BackgroundColor(Color::srgba(0.15, 0.16, 0.2, 0.92)),
                                BorderColor(Color::srgb(0.35, 0.38, 0.45)),
                                ToolButton(tool),
                            ))
                            .with_children(|btn| {
                                spawn_text(btn, tool.label(), 14.0, ());
                            });
                    }
                });

                spawn_text(left, "Type palette", 16.0, ());
                for slot in 0..tools.palette.len() {
                    let label = palette_slot_label(slot, tools.palette[slot]);
                    let entity = left
                        .spawn((
                            Button,
                            button_node(),
                            BackgroundColor(Color::srgba(0.15, 0.16, 0.2, 0.92)),
                            BorderColor(Color::srgb(0.35, 0.38, 0.45)),
                            PaletteButton(slot),
                        ))
                        .with_children(|btn| {
                            spawn_text(btn, &label, 14.0, ());
                        })
                        .id();
                    entities.palette_buttons.push(entity);
                }
            });

            // Top-right sim controls
            let mut controls_panel = base_panel_node();
            controls_panel.flex_direction = FlexDirection::Row;
            controls_panel.column_gap = Val::Px(6.0);
            controls_panel.top = Val::Px(12.0);
            controls_panel.right = Val::Px(12.0);
            root.spawn((
                controls_panel,
                BackgroundColor(Color::srgba(0.08, 0.09, 0.11, 0.85)),
            ))
            .with_children(|controls| {
                spawn_button(controls, "Pause", SimControlButton::Pause);
                spawn_button(controls, "Step", SimControlButton::Step);
                spawn_button(controls, "1x", SimControlButton::Speed);
                spawn_button(controls, "Debug", SimControlButton::DebugOverlay);
            });

            // Bottom-left inspector
            let mut inspector_panel = base_panel_node();
            inspector_panel.flex_direction = FlexDirection::Column;
            inspector_panel.row_gap = Val::Px(4.0);
            inspector_panel.bottom = Val::Px(12.0);
            inspector_panel.left = Val::Px(12.0);
            inspector_panel.min_width = Val::Px(220.0);
            root.spawn((
                inspector_panel,
                BackgroundColor(Color::srgba(0.08, 0.09, 0.11, 0.85)),
            ))
            .with_children(|inspector| {
                spawn_text(inspector, "Inspector", 16.0, ());
                spawn_text(
                    inspector,
                    "Inspect a cell (RMB or inspect tool)",
                    14.0,
                    FactoryTextPanel::Inspector,
                );
            });

            // Bottom-right throughput + depth
            let mut hud_panel = base_panel_node();
            hud_panel.flex_direction = FlexDirection::Column;
            hud_panel.row_gap = Val::Px(4.0);
            hud_panel.bottom = Val::Px(12.0);
            hud_panel.right = Val::Px(12.0);
            hud_panel.align_items = AlignItems::FlexEnd;
            root.spawn((
                hud_panel,
                BackgroundColor(Color::srgba(0.08, 0.09, 0.11, 0.85)),
            ))
            .with_children(|hud| {
                spawn_text(hud, "tick: 0", 14.0, FactoryTextPanel::Throughput);
                spawn_text(hud, "depth slice", 14.0, FactoryTextPanel::DepthSlice);
                spawn_text(hud, "", 13.0, FactoryTextPanel::DebugOverlay);
            });
        });
}

fn refresh_inspected_cell(
    sim: Res<GridSimulation>,
    mut inspected: ResMut<InspectedCell>,
) {
    let Some(pos) = inspected.pos else {
        return;
    };
    inspected.cell = sim.0.world.get(pos);
}

fn palette_button_system(
    mut interaction_q: Query<(&Interaction, &PaletteButton), Changed<Interaction>>,
    mut tools: ResMut<ToolState>,
) {
    for (interaction, button) in &mut interaction_q {
        if *interaction == Interaction::Pressed {
            tools.selected_slot = button.0;
            tools.active = ActiveTool::Place;
        }
    }
}

fn tool_button_system(
    mut interaction_q: Query<(&Interaction, &ToolButton), Changed<Interaction>>,
    mut tools: ResMut<ToolState>,
) {
    for (interaction, button) in &mut interaction_q {
        if *interaction == Interaction::Pressed {
            tools.active = button.0;
        }
    }
}

fn sim_control_button_system(
    mut interaction_q: Query<(&Interaction, &SimControlButton), Changed<Interaction>>,
    mut playback: ResMut<SimPlayback>,
    buttons: Query<(&SimControlButton, &Children)>,
    mut text_q: Query<&mut Text>,
) {
    for (interaction, control) in &mut interaction_q {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match control {
            SimControlButton::Pause => playback.paused = !playback.paused,
            SimControlButton::Step => playback.step_pending = true,
            SimControlButton::Speed => playback.speed = playback.speed.next(),
            SimControlButton::DebugOverlay => {
                playback.debug_overlay = !playback.debug_overlay;
            }
        }
    }

    for (control, children) in &buttons {
        let Ok(mut text) = text_q.get_mut(children[0]) else {
            continue;
        };
        let label = match control {
            SimControlButton::Pause => {
                if playback.paused {
                    "Resume"
                } else {
                    "Pause"
                }
            }
            SimControlButton::Step => "Step",
            SimControlButton::Speed => playback.speed.label(),
            SimControlButton::DebugOverlay => {
                if playback.debug_overlay {
                    "Debug*"
                } else {
                    "Debug"
                }
            }
        };
        **text = label.to_string();
    }
}

fn update_palette_highlight(
    tools: Res<ToolState>,
    mut buttons: Query<(&PaletteButton, &mut BackgroundColor, &mut BorderColor)>,
) {
    for (button, mut bg, mut border) in &mut buttons {
        apply_selected_style(&mut bg, &mut border, button.0 == tools.selected_slot);
    }
}

fn update_tool_highlight(
    tools: Res<ToolState>,
    mut buttons: Query<(&ToolButton, &mut BackgroundColor, &mut BorderColor)>,
) {
    for (button, mut bg, mut border) in &mut buttons {
        apply_selected_style(&mut bg, &mut border, button.0 == tools.active);
    }
}

fn update_factory_text(
    sim: Res<GridSimulation>,
    inspected: Res<InspectedCell>,
    slice: Res<ViewSlice>,
    active: Res<ActiveView>,
    playback: Res<SimPlayback>,
    stats: Res<SimTickStats>,
    mut text_q: Query<(&FactoryTextPanel, &mut Text)>,
) {
    for (panel, mut text) in &mut text_q {
        match panel {
            FactoryTextPanel::Inspector => {
                **text = format_inspector(inspected.pos, inspected.cell);
            }
            FactoryTextPanel::Throughput => {
                **text = format_throughput(
                    sim.0.tick,
                    sim.0.total_cell_updates,
                    stats.last_dirty_chunks,
                );
            }
            FactoryTextPanel::DepthSlice => {
                let view = active.0;
                let depth = slice.depth(view);
                let axis = ViewSlice::depth_axis_label(view);
                **text = format!("depth {axis}={depth} ({})", view.label());
            }
            FactoryTextPanel::DebugOverlay => {
                if playback.debug_overlay {
                    **text = format!(
                        "last tick: {} cell updates, {} dirty chunks",
                        stats.last_cell_updates, stats.last_dirty_chunks
                    );
                } else {
                    **text = String::new();
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BoxesAppPlugin;

    #[test]
    fn factory_ui_plugin_builds_without_panic() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, FactoryUiPlugin))
            .init_resource::<ToolState>()
            .init_resource::<InspectedCell>()
            .insert_resource(GridSimulation(boxes_sim::Simulation::new()))
            .init_resource::<SimPlayback>()
            .init_resource::<SimTickStats>()
            .init_resource::<ActiveView>()
            .init_resource::<ViewSlice>();
        app.update();
    }

    #[test]
    fn app_with_factory_ui_builds() {
        App::new().add_plugins(BoxesAppPlugin);
    }
}
