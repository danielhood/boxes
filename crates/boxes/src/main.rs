use bevy::prelude::*;
use boxes_app::BoxesAppPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BoxesAppPlugin)
        .run();
}
