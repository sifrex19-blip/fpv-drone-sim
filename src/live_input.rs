// Live controller input tracking.

use bevy::prelude::*;
use bevy::ecs::name::Name;
use crate::settings::GameSettings;
use crate::mapping::{InputMapping, default_range};
use crate::calibration::CalibrationState;

#[derive(Resource, Default, Clone)]
pub struct LiveInput {
    pub connected: bool,
    pub name: String,
    pub roll: f32,
    pub pitch: f32,
    pub yaw: f32,
    pub throttle: f32,
    pub buttons: Vec<String>,
}

pub struct LiveInputPlugin;
impl Plugin for LiveInputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LiveInput>()
            .add_systems(Update, update_live_input);
    }
}

pub fn update_live_input(
    gamepads: Query<(&Name, &Gamepad)>,
    settings: Res<GameSettings>,
    mapping: Res<InputMapping>,
    calibration: Res<CalibrationState>,
    mut live: ResMut<LiveInput>,
) {
    if let Some((name, gamepad)) = gamepads.iter().next() {
        live.connected = true;
        live.name = name.to_string();
        let sources = [mapping.roll, mapping.pitch, mapping.yaw, mapping.throttle];
        let reversed = [mapping.roll_reversed, mapping.pitch_reversed, mapping.yaw_reversed, mapping.throttle_reversed];
        let mut vals = [0.0f32; 4];
        for i in 0..4 {
            let raw = sources[i].read(gamepad);
            let (c, mn, mx) = if calibration.calibrated {
                (calibration.center[i], calibration.min[i], calibration.max[i])
            } else {
                default_range(sources[i])
            };
            let mut norm = if raw >= c {
                if (mx - c).abs() < 1e-6 { 0.0 } else { (raw - c) / (mx - c) }
            } else {
                if (c - mn).abs() < 1e-6 { 0.0 } else { (raw - c) / (c - mn) }
            };
            norm = norm.clamp(-1.0, 1.0);
            if reversed[i] {
                norm = -norm;
            }
            let dz = if calibration.calibrated { calibration.deadzone } else { settings.deadzone };
            let a = norm.abs();
            let mut out = if a < dz { 0.0 } else { (a - dz) / (1.0 - dz).max(1e-6) };
            out *= norm.signum();
            out *= settings.controller_sensitivity * mapping.sensitivity;
            out = out.clamp(-1.0, 1.0);
            vals[i] = out;
        }
        live.roll = vals[0];
        live.pitch = vals[1];
        live.yaw = vals[2];
        live.throttle = vals[3];
        live.buttons = gamepad.get_pressed().map(|b| format!("{:?}", b)).collect();
    } else {
        live.connected = false;
        live.name.clear();
        live.roll = 0.0;
        live.pitch = 0.0;
        live.yaw = 0.0;
        live.throttle = 0.0;
        live.buttons.clear();
    }
}
