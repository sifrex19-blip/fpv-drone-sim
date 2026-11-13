// Persistence: save/load settings to disk (RON).

use bevy::prelude::*;
use bevy::ecs::schedule::SystemCondition;
use crate::settings::GameSettings;
use crate::mapping::InputMapping;
use crate::calibration::CalibrationState;
use serde::{Deserialize, Serialize};

#[derive(Resource, Default, Serialize, Deserialize, Clone)]
pub struct SavedSettings {
    pub master_volume: f32,
    pub graphics_quality: u8,
    pub controller_sensitivity: f32,
    pub invert_pitch: bool,
    pub fov: f32,
    pub rotation_profile: u8,
    pub deadzone: f32,
    pub expo: f32,
    pub selected_controller: String,
    // Newer gameplay fields: default when missing so old files still load.
    #[serde(default = "default_acro")]
    pub flight_mode_acro: bool,
    #[serde(default = "default_rate_center")]
    pub rate_center: f32,
    #[serde(default = "default_rate_max")]
    pub rate_max: f32,
    #[serde(default = "default_max_tilt")]
    pub max_tilt: f32,
    #[serde(default = "default_camera_tilt")]
    pub camera_tilt: f32,
    #[serde(default)]
    pub chase_cam: bool,
    pub input_mapping: Option<InputMapping>,
    pub calibration: Option<CalibrationState>,
}

fn default_acro() -> bool { true }
fn default_rate_center() -> f32 { 220.0 }
fn default_rate_max() -> f32 { 760.0 }
fn default_max_tilt() -> f32 { 40.0 }
fn default_camera_tilt() -> f32 { 15.0 }

impl From<&GameSettings> for SavedSettings {
    fn from(s: &GameSettings) -> Self {
        Self {
            master_volume: s.master_volume,
            graphics_quality: s.graphics_quality,
            controller_sensitivity: s.controller_sensitivity,
            invert_pitch: s.invert_pitch,
            fov: s.fov,
            rotation_profile: s.rotation_profile,
            deadzone: s.deadzone,
            expo: s.expo,
            selected_controller: s.selected_controller.clone(),
            flight_mode_acro: s.flight_mode_acro,
            rate_center: s.rate_center,
            rate_max: s.rate_max,
            max_tilt: s.max_tilt,
            camera_tilt: s.camera_tilt,
            chase_cam: s.chase_cam,
            input_mapping: None,
            calibration: None,
        }
    }
}

pub struct PersistencePlugin;
impl Plugin for PersistencePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, load_settings)
            .add_systems(Update, save_settings.run_if(resource_changed::<GameSettings>.or_else(resource_changed::<InputMapping>).or_else(resource_changed::<CalibrationState>)));
    }
}

fn config_path() -> Option<std::path::PathBuf> {
    dirs::config_dir().map(|d| d.join("fpv-drone-sim").join("settings.ron"))
}

fn load_settings(mut settings: ResMut<GameSettings>, mut mapping: ResMut<InputMapping>, mut calibration: ResMut<CalibrationState>) {
    if let Some(path) = config_path() {
        if let Ok(data) = std::fs::read_to_string(&path) {
            if let Ok(saved) = ron::from_str::<SavedSettings>(&data) {
                settings.master_volume = saved.master_volume;
                settings.graphics_quality = saved.graphics_quality;
                settings.controller_sensitivity = saved.controller_sensitivity;
                settings.invert_pitch = saved.invert_pitch;
                settings.fov = saved.fov;
                settings.rotation_profile = saved.rotation_profile;
                settings.deadzone = saved.deadzone;
                settings.expo = saved.expo;
                settings.flight_mode_acro = saved.flight_mode_acro;
                settings.rate_center = saved.rate_center;
                settings.rate_max = saved.rate_max;
                settings.max_tilt = saved.max_tilt;
                settings.camera_tilt = saved.camera_tilt;
                settings.chase_cam = saved.chase_cam;
                if !saved.selected_controller.is_empty() {
                    settings.selected_controller = saved.selected_controller;
                }
                if let Some(m) = saved.input_mapping {
                    *mapping = m;
                }
                if let Some(c) = saved.calibration {
                    *calibration = c;
                }
            }
        }
    }
}

fn save_settings(settings: Res<GameSettings>, mapping: Res<InputMapping>, calibration: Res<CalibrationState>) {
    if let Some(path) = config_path() {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let mut saved = SavedSettings::from(&*settings);
        saved.input_mapping = Some(mapping.clone());
        saved.calibration = Some(calibration.clone());
        if let Ok(data) = ron::to_string(&saved) {
            let _ = std::fs::write(&path, data);
        }
    }
}
