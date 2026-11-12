// Input abstraction: unifies keyboard + any connected gamepad/controller.
// Bevy's gilrs integration auto-detects all controllers, so "any controller" works.

use bevy::prelude::*;
use crate::live_input::LiveInput;

/// Normalized control inputs in [-1, 1] (throttle [0,1]).
#[derive(Resource, Default, Clone, Copy)]
pub struct DroneInput {
    pub throttle: f32, // 0..1
    pub pitch: f32,    // -1..1
    pub roll: f32,     // -1..1
    pub yaw: f32,      // -1..1
    pub arm: bool,
    pub reset: bool,
}

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DroneInput>()
            .add_systems(Update, gather_input.after(crate::live_input::update_live_input));
    }
}

fn gather_input(
    keys: Res<ButtonInput<KeyCode>>,
    live: Res<LiveInput>,
    gamepads: Query<&Gamepad>,
    mut input: ResMut<DroneInput>,
) {

    // Keyboard fallback (WASD + Space/Shift for throttle)
    let mut pitch = 0.0;
    let mut roll = 0.0;
    let mut yaw = 0.0;
    let mut throttle = 0.0;

    if keys.pressed(KeyCode::KeyW) { pitch += 1.0; }
    if keys.pressed(KeyCode::KeyS) { pitch -= 1.0; }
    if keys.pressed(KeyCode::KeyA) { roll -= 1.0; }
    if keys.pressed(KeyCode::KeyD) { roll += 1.0; }
    if keys.pressed(KeyCode::KeyQ) { yaw -= 1.0; }
    if keys.pressed(KeyCode::KeyE) { yaw += 1.0; }
    let keyboard_throttle = if keys.pressed(KeyCode::Space) {
        Some(1.0)
    } else if keys.pressed(KeyCode::ShiftLeft) {
        Some(0.0)
    } else {
        None
    };

    // Gamepad/controller: use the first connected one (works with any controller)
    let mut gamepad_arm = false;
    let mut gamepad_reset = false;
    // Gamepad contribution from LiveInput (mapped/calibrated)
    if live.connected {
        roll += live.roll;
        pitch += live.pitch;
        yaw += live.yaw;
        throttle += live.throttle;
    }
    for gamepad in gamepads.iter() {
        if gamepad.just_pressed(GamepadButton::South) { gamepad_arm = true; }
        if gamepad.just_pressed(GamepadButton::Start) { gamepad_reset = true; }
        let _ = gamepad;
    }

    // Normalize throttle to 0..1
    let t = if let Some(kt) = keyboard_throttle {
        kt
    } else {
        ((throttle * 0.5 + 0.5).clamp(0.0, 1.0))
    };

    input.throttle = t;
    input.pitch = pitch.clamp(-1.0, 1.0);
    input.roll = roll.clamp(-1.0, 1.0);
    input.yaw = yaw.clamp(-1.0, 1.0);
    input.arm = keys.just_pressed(KeyCode::Enter) || gamepad_arm;
    input.reset = keys.just_pressed(KeyCode::KeyR) || gamepad_reset;
}
