// FPV Drone Simulator - main entry point
// Built with Bevy for cross-platform 60fps+ performance.

mod camera;
mod drone;
mod input;
mod ui;

use bevy::prelude::*;
use bevy::window::WindowResolution;
use drone::DronePlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "FPV Drone Sim".into(),
                resolution: WindowResolution::new(1280, 720),                ..default()
            }),
            ..default()
        }))
        .add_plugins(DronePlugin)
        .add_plugins(camera::CameraPlugin)
        .add_plugins(input::InputPlugin)
        .add_plugins(ui::UiPlugin)
        .run();
}
