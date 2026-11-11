// FPV Drone Simulator - main entry point

mod camera;
mod drone;
mod input;
mod settings;
mod ui;

use bevy::prelude::*;
use bevy::window::WindowResolution;
use drone::DronePlugin;
use input::InputPlugin;
use settings::SettingsPlugin;
use ui::UiPlugin;

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum AppState {
    #[default]
    Menu,
    InGame,
    Settings,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "FPV Drone Sim".into(),
                resolution: WindowResolution::new(1280, 720),
                ..default()
            }),
            ..default()
        }))
        .init_state::<AppState>()
        .insert_resource(ClearColor(Color::srgb(0.53,0.75,0.98)))
        .add_plugins(DronePlugin)
        .add_plugins(camera::CameraPlugin)
        .add_plugins(InputPlugin)
        .add_plugins(SettingsPlugin)
        .add_plugins(UiPlugin)
        .run();
}
