// Input mapping: axis assignments, reversal, sensitivity.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AxisSource {
    #[default]
    LeftStickX,
    LeftStickY,
    RightStickX,
    RightStickY,
    LeftTrigger,
    RightTrigger,
    DPadX,
    DPadY,
}

impl AxisSource {
    pub fn read(&self, gamepad: &Gamepad) -> f32 {
        match self {
            AxisSource::LeftStickX => gamepad.get(GamepadAxis::LeftStickX).unwrap_or(0.0),
            AxisSource::LeftStickY => gamepad.get(GamepadAxis::LeftStickY).unwrap_or(0.0),
            AxisSource::RightStickX => gamepad.get(GamepadAxis::RightStickX).unwrap_or(0.0),
            AxisSource::RightStickY => gamepad.get(GamepadAxis::RightStickY).unwrap_or(0.0),
            AxisSource::LeftTrigger => gamepad.get(GamepadAxis::LeftZ).unwrap_or(0.0),
            AxisSource::RightTrigger => gamepad.get(GamepadAxis::RightZ).unwrap_or(0.0),
            AxisSource::DPadX => gamepad.dpad().x,
            AxisSource::DPadY => gamepad.dpad().y,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            AxisSource::LeftStickX => "LeftStickX",
            AxisSource::LeftStickY => "LeftStickY",
            AxisSource::RightStickX => "RightStickX",
            AxisSource::RightStickY => "RightStickY",
            AxisSource::LeftTrigger => "LeftTrigger",
            AxisSource::RightTrigger => "RightTrigger",
            AxisSource::DPadX => "DPadX",
            AxisSource::DPadY => "DPadY",
        }
    }

    pub fn all() -> &'static [AxisSource] {
        &[
            AxisSource::LeftStickX,
            AxisSource::LeftStickY,
            AxisSource::RightStickX,
            AxisSource::RightStickY,
            AxisSource::LeftTrigger,
            AxisSource::RightTrigger,
            AxisSource::DPadX,
            AxisSource::DPadY,
        ]
    }

    pub fn next(&self) -> AxisSource {
        let all = Self::all();
        let idx = all.iter().position(|a| a == self).unwrap_or(0);
        all[(idx + 1) % all.len()]
    }
}

pub fn default_range(src: AxisSource) -> (f32, f32, f32) {
    match src {
        AxisSource::LeftTrigger | AxisSource::RightTrigger => (0.5, 0.0, 1.0),
        _ => (0.0, -1.0, 1.0),
    }
}

#[derive(Resource, Clone, Serialize, Deserialize)]
pub struct InputMapping {
    pub roll: AxisSource,
    pub pitch: AxisSource,
    pub yaw: AxisSource,
    pub throttle: AxisSource,
    pub roll_reversed: bool,
    pub pitch_reversed: bool,
    pub yaw_reversed: bool,
    pub throttle_reversed: bool,
    pub sensitivity: f32,
}

impl Default for InputMapping {
    fn default() -> Self {
        Self {
            roll: AxisSource::LeftStickX,
            pitch: AxisSource::LeftStickY,
            yaw: AxisSource::RightStickX,
            throttle: AxisSource::RightTrigger,
            roll_reversed: false,
            pitch_reversed: false,
            yaw_reversed: false,
            throttle_reversed: false,
            sensitivity: 1.0,
        }
    }
}

pub struct MappingPlugin;
impl Plugin for MappingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputMapping>();
    }
}
