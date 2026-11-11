// FPV camera: follows drone with low latency, minimal smoothing for responsiveness.

use bevy::prelude::*;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera)
            .add_systems(Update, follow_drone);
    }
}

#[derive(Component)]
struct FpvCamera;

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 5.0, 0.0),
        FpvCamera,
    ));
    // Sun
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            ..default()
        },
        Transform::from_xyz(10.0, 20.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.insert_resource(GlobalAmbientLight {
        color: Color::WHITE,
        brightness: 200.0,
        ..default()
    });
}

fn follow_drone(
    drone_q: Query<&Transform, (With<crate::drone::Drone>, Without<FpvCamera>)>,
    mut cam_q: Query<&mut Transform, With<FpvCamera>>,
) {
    if let (Ok(drone_t), Ok(mut cam_t)) = (drone_q.single(), cam_q.single_mut()) {
        // FPV: camera sits slightly behind/above drone, looks forward along drone heading
        let offset = drone_t.rotation * Vec3::new(0.0, 0.05, 0.15);
        cam_t.translation = drone_t.translation + offset;
        cam_t.rotation = drone_t.rotation;
    }
}
