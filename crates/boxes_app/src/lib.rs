//! Bevy application shell for Boxes — simulation bridge and chunked grid rendering.

mod input;
mod render;
mod sim_bridge;
mod ui;

use bevy::prelude::*;

pub use input::{ActiveTool, InputPlugin, PalettePreset, SelectedCell, ToolState};
pub use render::{ActiveView, GridRenderPlugin, OrthoView, ViewCameraState, ViewPose};
pub use sim_bridge::{GridSimulation, SimPlayback, SimSpeed, SimTickStats};
pub use ui::{FactoryUiPlugin, HelpUiPlugin};

use sim_bridge::{setup_simulation, sim_step_system};

/// Root plugin: clear color, grid rendering, and simulation stepping.
pub struct BoxesAppPlugin;

impl Plugin for BoxesAppPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::srgb(0.08, 0.09, 0.11)))
            .add_plugins((GridRenderPlugin, InputPlugin, FactoryUiPlugin, HelpUiPlugin))
            .add_systems(Startup, setup_simulation)
            .add_systems(Update, sim_step_system);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plugin_builds_without_panic() {
        App::new().add_plugins(BoxesAppPlugin);
    }
}
