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
    pub input_mapping: Option<InputMapping>,
    pub calibration: Option<CalibrationState>,
}

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
