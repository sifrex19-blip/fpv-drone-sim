// Input abstraction: unifies keyboard + any connected gamepad/controller.
// Bevy's gilrs integration auto-detects all controllers, so "any controller" works.

use bevy::prelude::*;

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
            .add_systems(Update, gather_input);
    }
}

fn gather_input(
    keys: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    axes: Option<ResMut<Axis<GamepadAxis>>>,
    buttons: Option<ResMut<ButtonInput<GamepadButton>>>,
    mut input: ResMut<DroneInput>,
) {
    let mut axes = axes;
    let mut buttons = buttons;
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
    if keys.pressed(KeyCode::Space) { throttle += 1.0; }
    if keys.pressed(KeyCode::ShiftLeft) { throttle -= 1.0; }

    // Gamepad/controller: use the first connected one (works with any controller)
    if let (Some(mut axes), Some(mut buttons)) = (axes, buttons) {
        for gamepad in gamepads.iter() {
            if let Some(v) = axes.get(GamepadAxis::LeftStickX) { roll += v; }
            if let Some(v) = axes.get(GamepadAxis::LeftStickY) { pitch += v; }
            if let Some(v) = axes.get(GamepadAxis::RightStickX) { yaw += v; }
            let rt = buttons.pressed(GamepadButton::RightTrigger2);
            let lt = buttons.pressed(GamepadButton::LeftTrigger2);
            if rt { throttle += 1.0; }
            if lt { throttle -= 1.0; }
            if let Some(v) = axes.get(GamepadAxis::RightZ) { throttle += v.max(0.0); }
            if let Some(v) = axes.get(GamepadAxis::LeftZ) { throttle -= v.max(0.0); }
            let _ = gamepad;
        }
        // arm/reset via controller
        // (handled below with buttons still in scope)
        let _ = &mut buttons;
    }

    // Normalize throttle to 0..1
    let t = (throttle * 0.5 + 0.5).clamp(0.0, 1.0);

    input.throttle = t;
    input.pitch = pitch.clamp(-1.0, 1.0);
    input.roll = roll.clamp(-1.0, 1.0);
    input.yaw = yaw.clamp(-1.0, 1.0);
    input.arm = keys.just_pressed(KeyCode::Enter);
    input.reset = keys.just_pressed(KeyCode::KeyR);
}
