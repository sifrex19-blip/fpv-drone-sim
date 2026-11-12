// FPV Drone Simulator - main entry point

mod camera;
mod controllers;
mod drone;
mod input;
mod live_input;
mod calibration;
mod mapping;
mod persistence;
mod settings;
mod ui;

use bevy::prelude::*;
use bevy::window::WindowResolution;
use drone::DronePlugin;
use input::InputPlugin;
use live_input::LiveInputPlugin;
use calibration::CalibrationPlugin;
use mapping::MappingPlugin;
use settings::SettingsPlugin;
use ui::UiPlugin;

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum AppState {
    #[default]
    Home,
    Play,
    Controller,
    Settings,
    Addons,
    Statistics,
    About,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "FPV Drone Sim".into(),
                resolution: WindowResolution::new(1280, 720),
                resizable: true,
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.05, 0.05, 0.08)))
        .init_state::<AppState>()
        .add_plugins(DronePlugin)
        .add_plugins(camera::CameraPlugin)
        .add_plugins(InputPlugin)
        .add_plugins(LiveInputPlugin)
        .add_plugins(CalibrationPlugin)
        .add_plugins(MappingPlugin)
        .add_plugins(SettingsPlugin)
        .add_plugins(persistence::PersistencePlugin)
        .add_plugins(UiPlugin)
        .run();
}
