use bevy::prelude::*;
use crate::AppState;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_sun)
            .add_systems(Startup, spawn_ui_camera)
            .add_systems(OnEnter(AppState::Play), spawn_camera)
            .add_systems(OnExit(AppState::Play), despawn_camera)
            .add_systems(Update, follow_drone.run_if(in_state(AppState::Play)));
    }
}

#[derive(Component)]
struct FpvCamera;

#[derive(Component)]
struct UiCamera;

fn spawn_sun(mut commands: Commands) {
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

fn despawn_camera(mut commands: Commands, q: Query<Entity, With<FpvCamera>>, q2: Query<Entity, With<UiCamera>>) {
    for e in q.iter() { commands.entity(e).despawn(); }
    for e in q2.iter() { commands.entity(e).despawn(); }
}

fn spawn_ui_camera(mut commands: Commands) {
    commands.spawn((Camera2d::default(), UiCamera));
}
fn despawn_ui_camera(mut commands: Commands, q: Query<Entity, With<UiCamera>>) {
    for e in q.iter() { commands.entity(e).despawn(); }
}
fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 5.0, 0.0),
        FpvCamera,
    ));
    // UI overlay camera
    commands.spawn((Camera2d::default(), Camera { order: 1, ..default() }, UiCamera));
}

fn follow_drone(
    drone_q: Query<&Transform, (With<crate::drone::Drone>, Without<FpvCamera>)>,
    mut cam_q: Query<&mut Transform, With<FpvCamera>>,
) {
    if let (Ok(drone_t), Ok(mut cam_t)) = (drone_q.single(), cam_q.single_mut()) {
        let offset = drone_t.rotation * Vec3::new(0.0, 0.05, 0.1);
        cam_t.translation = drone_t.translation + offset;
        cam_t.rotation = drone_t.rotation;
    }
}
