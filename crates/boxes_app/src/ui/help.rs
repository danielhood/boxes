//! Key and mouse binding help overlay (`?` to toggle).

use bevy::prelude::*;

/// Root marker for the help overlay (hidden by default).
#[derive(Component)]
pub struct HelpOverlayRoot;

/// Semi-transparent backdrop — click to dismiss.
#[derive(Component)]
pub struct HelpBackdrop;

#[derive(Resource, Default)]
pub struct HelpOverlayState {
    pub visible: bool,
}

pub struct HelpUiPlugin;

impl Plugin for HelpUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HelpOverlayState>()
            .add_systems(Startup, setup_help_overlay)
            .add_systems(Update, (help_toggle_system, help_dismiss_system));
    }
}

const HELP_TEXT: &str = "\
Selection & navigation
  Arrow keys     Move selected cell (view stays)
  [ / ]          Step depth slice (selection)
  Mouse wheel    Step depth slice
  LMB            Select cell on active slice (drag to track)
  Ctrl+arrows    Rotate to adjacent face
  T              Snap to top view

Pan & recenter
  Shift+arrows   Pan view (¼ viewport per press)
  MMB drag       Pan view (release snaps anchor)
  C              Center view on selected cell

Views & zoom
  Ctrl+wheel     Zoom in/out (8–64 cells)
  Ctrl+[ / ]     Zoom out / in

Tools & palette
  RMB            Apply active tool (drag supported)
  P / E / I      Place / Erase / Inspect tool
  Shift+1–9      Palette slot (place mode)

Sim controls (HUD)
  Pause / Resume   Toggle simulation
  Step             One tick while paused
  Speed            Cycle 0.5x / 1x / 2x
  Debug            Toggle tick stats overlay

  ?              Toggle this help
  Esc            Close help";

fn setup_help_overlay(mut commands: Commands, mut state: ResMut<HelpOverlayState>) {
    state.visible = false;

    commands
        .spawn((
            HelpOverlayRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                display: Display::None,
                ..default()
            },
        ))
        .with_children(|root| {
            root.spawn((
                HelpBackdrop,
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    position_type: PositionType::Absolute,
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55)),
            ));

            root.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(16.0)),
                    max_width: Val::Px(480.0),
                    max_height: Val::Percent(80.0),
                    overflow: Overflow::scroll_y(),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.08, 0.09, 0.11, 0.96)),
                BorderColor(Color::srgb(0.35, 0.38, 0.45)),
            ))
            .with_children(|panel| {
                panel.spawn((
                    Text::new("Help — key & mouse bindings"),
                    TextFont {
                        font_size: 18.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.92, 0.93, 0.95)),
                    Node {
                        margin: UiRect::bottom(Val::Px(8.0)),
                        ..default()
                    },
                ));
                panel.spawn((
                    Text::new(HELP_TEXT),
                    TextFont {
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.85, 0.87, 0.9)),
                ));
            });
        });
}

fn question_pressed(keyboard: &ButtonInput<KeyCode>) -> bool {
    keyboard.just_pressed(KeyCode::Slash)
        && (keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight))
}

fn help_toggle_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<HelpOverlayState>,
    mut overlay: Query<&mut Node, With<HelpOverlayRoot>>,
) {
    if !question_pressed(&keyboard) {
        return;
    }
    state.visible = !state.visible;
    if let Ok(mut node) = overlay.single_mut() {
        node.display = if state.visible {
            Display::Flex
        } else {
            Display::None
        };
    }
}

fn help_dismiss_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<HelpOverlayState>,
    interactions: Query<&Interaction, (Changed<Interaction>, With<HelpBackdrop>)>,
    mut overlay: Query<&mut Node, With<HelpOverlayRoot>>,
) {
    let dismiss = keyboard.just_pressed(KeyCode::Escape)
        || interactions.iter().any(|i| *i == Interaction::Pressed);

    if !dismiss || !state.visible {
        return;
    }

    state.visible = false;
    if let Ok(mut node) = overlay.single_mut() {
        node.display = Display::None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn help_plugin_builds_without_panic() {
        App::new().add_plugins((MinimalPlugins, HelpUiPlugin));
    }
}
