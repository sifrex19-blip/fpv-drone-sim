// Minimal HUD: armed state, throttle.

use bevy::prelude::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_ui)
            .add_systems(Update, update_ui);
    }
}

#[derive(Component)]
struct HudText;

fn spawn_ui(mut commands: Commands) {
    commands.spawn((
        Text::new("FPV Drone Sim\nArmed: NO\nThrottle: 0.00\n\nWASD pitch/roll | Q/E yaw | Space/Shift throttle\nEnter arm | R reset\nAny controller auto-detected"),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
        HudText,
    ));
}

fn update_ui(
    input: Res<crate::input::DroneInput>,
    drone_q: Query<&crate::drone::Drone>,
    mut text_q: Query<&mut Text, With<HudText>>,
) {
    if let Ok(mut text) = text_q.single_mut() {
        let armed = drone_q.single().map(|d| d.armed).unwrap_or(false);
        text.0 = format!(
            "FPV Drone Sim\nArmed: {}\nThrottle: {:.2}\nPitch: {:.2} Roll: {:.2} Yaw: {:.2}",
            if armed { "YES" } else { "NO" },
            input.throttle,
            input.pitch,
            input.roll,
            input.yaw,
        );
    }
}
