// Calibration state and logic.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use crate::mapping::{InputMapping, default_range};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CalibrationPhase {
    #[default]
    Idle,
    Center,
    MinMax,
}

#[derive(Component)]
pub struct CalibrateBtn;

#[derive(Resource, Clone, Serialize, Deserialize)]
pub struct CalibrationState {
    pub phase: CalibrationPhase,
    pub calibrated: bool,
    pub center: [f32; 4],
    pub min: [f32; 4],
    pub max: [f32; 4],
    pub deadzone: f32,
    #[serde(skip)]
    pub samples: u32,
    #[serde(skip)]
    pub accum: [f32; 4],
    #[serde(skip)]
    pub min_seen: [f32; 4],
    #[serde(skip)]
    pub max_seen: [f32; 4],
}

impl Default for CalibrationState {
    fn default() -> Self {
        Self {
            phase: CalibrationPhase::Idle,
            calibrated: false,
            center: [0.0; 4],
            min: [-1.0, -1.0, -1.0, 0.0],
            max: [1.0, 1.0, 1.0, 1.0],
            deadzone: 0.05,
            samples: 0,
            accum: [0.0; 4],
            min_seen: [f32::MAX; 4],
            max_seen: [f32::MIN; 4],
        }
    }
}

pub struct CalibrationPlugin;
impl Plugin for CalibrationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CalibrationState>()
            .add_systems(Update, (calibration_system, handle_calibrate_button));
    }
}

pub fn handle_calibrate_button(
    q: Query<&Interaction, (Changed<Interaction>, With<CalibrateBtn>)>,
    mut cal: ResMut<CalibrationState>,
) {
    for i in q.iter() {
        if *i == Interaction::Pressed {
            cal.phase = CalibrationPhase::Center;
            cal.samples = 0;
            cal.accum = [0.0; 4];
            cal.min_seen = [f32::MAX; 4];
            cal.max_seen = [f32::MIN; 4];
            cal.calibrated = false;
        }
    }
}

pub fn calibration_system(
    gamepads: Query<&Gamepad>,
    mapping: Res<InputMapping>,
    mut cal: ResMut<CalibrationState>,
) {
    let Some(gamepad) = gamepads.iter().next() else { return; };
    let sources = [mapping.roll, mapping.pitch, mapping.yaw, mapping.throttle];
    let mut raw = [0.0f32; 4];
    for i in 0..4 {
        raw[i] = sources[i].read(gamepad);
    }
    match cal.phase {
        CalibrationPhase::Idle => {}
        CalibrationPhase::Center => {
            for i in 0..4 {
                cal.accum[i] += raw[i];
            }
            cal.samples += 1;
            if cal.samples >= 90 {
                for i in 0..4 {
                    cal.center[i] = cal.accum[i] / cal.samples as f32;
                    cal.min_seen[i] = cal.center[i];
                    cal.max_seen[i] = cal.center[i];
                }
                cal.phase = CalibrationPhase::MinMax;
                cal.samples = 0;
                cal.accum = [0.0; 4];
            }
        }
        CalibrationPhase::MinMax => {
            for i in 0..4 {
                cal.min_seen[i] = cal.min_seen[i].min(raw[i]);
                cal.max_seen[i] = cal.max_seen[i].max(raw[i]);
            }
            cal.samples += 1;
            if cal.samples >= 240 {
                for i in 0..4 {
                    cal.min[i] = cal.min_seen[i];
                    cal.max[i] = cal.max_seen[i];
                    if (cal.max[i] - cal.min[i]).abs() < 1e-3 {
                        let (c, mn, mx) = default_range(sources[i]);
                        cal.center[i] = c;
                        cal.min[i] = mn;
                        cal.max[i] = mx;
                    }
                }
                cal.deadzone = 0.06;
                cal.calibrated = true;
                cal.phase = CalibrationPhase::Idle;
            }
        }
    }
}
