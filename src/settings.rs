// Settings: flight, camera, controller, graphics. Persisted via persistence.rs.

use bevy::prelude::*;

#[derive(Resource, Clone)]
pub struct GameSettings {
    pub master_volume: f32,
    pub graphics_quality: u8, // 0 low,1 med,2 high,3 ultra
    pub controller_sensitivity: f32,
    pub invert_pitch: bool,
    pub fov: f32,               // vertical FOV in degrees
    pub rotation_profile: u8,   // 0 slow,1 normal,2 fast
    pub deadzone: f32,
    pub expo: f32,
    pub selected_controller: String,
    // --- gameplay (FPV-sim style) ---
    pub flight_mode_acro: bool, // true = acro (rate), false = angle (self-level)
    pub rate_center: f32,       // center sensitivity, deg/s
    pub rate_max: f32,          // max rotation rate, deg/s
    pub max_tilt: f32,          // angle-mode max tilt, degrees
    pub camera_tilt: f32,       // FPV camera uptilt, degrees
    pub chase_cam: bool,        // false = FPV, true = chase cam
}

impl GameSettings {
    pub fn rotation_multiplier(&self) -> f32 {
        match self.rotation_profile {
            0 => 0.5,
            2 => 1.8,
            _ => 1.0,
        }
    }

    pub fn flight_mode_label(&self) -> &'static str {
        if self.flight_mode_acro { "ACRO" } else { "ANGLE" }
    }
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            master_volume: 0.8,
            graphics_quality: 2,
            controller_sensitivity: 1.0,
            invert_pitch: false,
            fov: 100.0,
            rotation_profile: 1,
            deadzone: 0.05,
            expo: 0.0,
            selected_controller: "Generic Gamepad".into(),
            flight_mode_acro: true,
            rate_center: 220.0,
            rate_max: 760.0,
            max_tilt: 40.0,
            camera_tilt: 15.0,
            chase_cam: false,
        }
    }
}

pub struct SettingsPlugin;
impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameSettings>();
    }
}
