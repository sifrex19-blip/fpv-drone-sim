// FPV Drone Simulator - main entry point

mod camera;
mod controllers;
mod drone;
mod hud;
mod input;
mod live_input;
mod calibration;
mod mapping;
mod persistence;
mod race;
mod settings;
mod stats;
mod theme;
mod ui;
mod world;

use bevy::prelude::*;
use bevy::window::WindowResolution;
use drone::DronePlugin;
use input::InputPlugin;
use live_input::LiveInputPlugin;
use calibration::CalibrationPlugin;
use mapping::MappingPlugin;
use settings::SettingsPlugin;
use ui::UiPlugin;

use std::path::PathBuf;
use std::sync::OnceLock;

static ASSET_ROOT: OnceLock<PathBuf> = OnceLock::new();

/// Root directory that holds the `assets/` folder.
/// Prefers the location next to the executable (packaged layout),
/// then falls back to `./assets` (running via `cargo run`).
///
/// Always absolute: Bevy resolves a relative `AssetPlugin::file_path`
/// against the executable's directory, not the current working directory.
pub fn asset_root() -> &'static PathBuf {
    ASSET_ROOT.get_or_init(|| {
        if let Ok(exe) = std::env::current_exe() {
            if let Some(dir) = exe.parent() {
                let bundled = dir.join("assets");
                if bundled.is_dir() {
                    return bundled;
                }
            }
        }
        std::env::current_dir()
            .map(|cwd| cwd.join("assets"))
            .unwrap_or_else(|_| PathBuf::from("assets"))
    })
}

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

/// Dev/testing helper: start on a specific page with `--page <name>`.
fn initial_page() -> AppState {
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--page" {
            return match args.next().as_deref() {
                Some("play") => AppState::Play,
                Some("controller") => AppState::Controller,
                Some("settings") => AppState::Settings,
                Some("addons") => AppState::Addons,
                Some("stats") | Some("statistics") => AppState::Statistics,
                Some("about") => AppState::About,
                _ => AppState::Home,
            };
        }
    }
    AppState::Home
}

/// Dev/testing helper: `--shot <path>` captures the app's own render target
/// (GPU readback — works even when other windows cover ours), saves it to
/// `<path>` and exits. The window is not focused in this mode so it never
/// steals focus from whatever the user is doing.
fn shot_path() -> Option<PathBuf> {
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--shot" {
            return args.next().map(PathBuf::from);
        }
    }
    None
}

#[derive(Resource)]
struct ShotCfg(PathBuf);

fn auto_shot(
    cfg: Option<Res<ShotCfg>>,
    time: Res<Time>,
    mut phase: Local<u8>,
    mut commands: Commands,
    shots: Query<(), With<bevy::render::view::screenshot::Screenshot>>,
) {
    let Some(cfg) = cfg else { return };
    match *phase {
        0 => {
            // Let fonts, images and the first frames settle before capturing.
            if time.elapsed_secs() >= 3.0 {
                info!("SHOT: capturing {}", cfg.0.display());
                commands
                    .spawn(bevy::render::view::screenshot::Screenshot::primary_window())
                    .observe(bevy::render::view::screenshot::save_to_disk(cfg.0.clone()));
                *phase = 1;
            }
        }
        1 => {
            // The screenshot entity despawns only after the save observer ran,
            // so an empty query means the file is on disk.
            if shots.is_empty() {
                info!("SHOT: saved {}", cfg.0.display());
                std::process::exit(0);
            }
            // Safety valve: never hang the harness if the capture stalls.
            if time.elapsed_secs() > 23.0 {
                error!("SHOT: timed out waiting for capture");
                std::process::exit(1);
            }
        }
        _ => {}
    }
}

fn main() {
    let shot = shot_path();

    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "FPV Drone Sim".into(),
            resolution: WindowResolution::new(1280, 720),
            resizable: true,
            // Screenshot mode must not steal focus from the user.
            focused: shot.is_none(),
            ..default()
        }),
        ..default()
    }).set(bevy::asset::AssetPlugin {
        file_path: asset_root().to_string_lossy().to_string(),
        ..default()
    }))
        .insert_resource(ClearColor(Color::srgb(0.05, 0.05, 0.08)))
        .insert_state(initial_page())
        .add_plugins(DronePlugin)
        .add_plugins(camera::CameraPlugin)
        .add_plugins(InputPlugin)
        .add_plugins(LiveInputPlugin)
        .add_plugins(CalibrationPlugin)
        .add_plugins(MappingPlugin)
        .add_plugins(SettingsPlugin)
        .add_plugins(persistence::PersistencePlugin)
        .add_plugins(stats::StatsPlugin)
        .add_plugins(world::WorldPlugin)
        .add_plugins(race::RacePlugin)
        .add_plugins(hud::HudPlugin)
        .add_plugins(UiPlugin);

    if let Some(path) = shot {
        app.insert_resource(ShotCfg(path)).add_systems(Update, auto_shot);
    }

    app.run();
}
