// Settings: controller sensitivity, graphics, audio. In-memory (can be persisted later).

use bevy::prelude::*;

#[derive(Resource, Clone)]
pub struct GameSettings {
    pub master_volume: f32,
    pub graphics_quality: u8, // 0 low,1 med,2 high,3 ultra
    pub controller_sensitivity: f32,
    pub invert_pitch: bool,
    pub fov: f32,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            master_volume: 0.8,
            graphics_quality: 2,
            controller_sensitivity: 1.0,
            invert_pitch: false,
            fov: 90.0,
        }
    }
}

pub struct SettingsPlugin;
impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameSettings>();
    }
}
