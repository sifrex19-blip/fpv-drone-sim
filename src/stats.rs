// Flight statistics: tracked during play, shown on the Statistics page, saved to disk.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use crate::AppState;
use crate::drone::Drone;
use crate::hud::Pause;

#[derive(Resource, Clone, Serialize, Deserialize)]
pub struct FlightStats {
    pub flights: u32,
    pub flight_time: f32,   // seconds with motors running
    pub distance: f32,      // meters travelled
    pub top_speed: f32,     // km/h
    pub crashes: u32,
    pub laps: u32,
    pub best_lap: f32,      // seconds; 0 = none
}

impl Default for FlightStats {
    fn default() -> Self {
        Self {
            flights: 0,
            flight_time: 0.0,
            distance: 0.0,
            top_speed: 0.0,
            crashes: 0,
            laps: 0,
            best_lap: 0.0,
        }
    }
}

pub struct StatsPlugin;
impl Plugin for StatsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FlightStats>()
            .add_systems(Startup, load_stats)
            .add_systems(OnEnter(AppState::Play), count_flight)
            .add_systems(OnExit(AppState::Play), save_stats)
            .add_systems(Update, track_flight.run_if(in_state(AppState::Play).and_then(not_paused)));
    }
}

fn not_paused(pause: Res<Pause>) -> bool {
    !pause.0
}

fn stats_path() -> Option<std::path::PathBuf> {
    dirs::config_dir().map(|d| d.join("fpv-drone-sim").join("stats.ron"))
}

/// Writes current stats to disk (used on discrete events like crashes/laps).
pub fn write_stats(stats: &FlightStats) {
    if let Some(path) = stats_path() {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(data) = ron::to_string(stats) {
            let _ = std::fs::write(&path, data);
        }
    }
}

pub fn save_stats(stats: Res<FlightStats>) {
    write_stats(&stats);
}

fn load_stats(mut stats: ResMut<FlightStats>) {
    if let Some(path) = stats_path() {
        if let Ok(data) = std::fs::read_to_string(&path) {
            if let Ok(saved) = ron::from_str::<FlightStats>(&data) {
                *stats = saved;
            }
        }
    }
}

fn count_flight(mut stats: ResMut<FlightStats>) {
    stats.flights += 1;
}

/// Accumulates flight time, distance and top speed while armed.
fn track_flight(
    time: Res<Time>,
    drone_q: Query<&Drone>,
    mut stats: ResMut<FlightStats>,
) {
    let Ok(drone) = drone_q.single() else { return };
    if !drone.armed {
        return;
    }
    let dt = time.delta_secs();
    stats.flight_time += dt;
    let speed_ms = drone.velocity.length();
    stats.distance += speed_ms * dt;
    stats.top_speed = stats.top_speed.max(speed_ms * 3.6);
}
