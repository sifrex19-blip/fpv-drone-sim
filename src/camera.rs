// Camera: FPV view (rigid + configurable uptilt) and smoothed chase cam.

use bevy::prelude::*;
use bevy::camera::{Projection, PerspectiveProjection};
use bevy::pbr::{DistanceFog, FogFalloff};
use crate::AppState;
use crate::settings::GameSettings;
use crate::hud::{Flash, not_paused};
use crate::input::{DroneInput, gather_input};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_sun)
            .add_systems(Startup, spawn_ui_camera)
            .add_systems(OnEnter(AppState::Play), spawn_camera)
            .add_systems(OnExit(AppState::Play), despawn_camera)
            .add_systems(
                Update,
                (follow_drone, apply_camera_settings)
                    .chain()
                    .after(gather_input)
                    .run_if(in_state(AppState::Play).and_then(not_paused)),
            );
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

fn despawn_camera(mut commands: Commands, q: Query<Entity, With<FpvCamera>>) {
    for e in q.iter() { commands.entity(e).despawn(); }
}

fn spawn_ui_camera(mut commands: Commands) {
    // One persistent UI camera for the whole app. Order 1 keeps UI above the
    // 3D view (order 0) once Play spawns the FPV camera; spawning a second UI
    // camera per state would leave two cameras on priority 0/1 at once.
    commands.spawn((
        Camera2d::default(),
        Camera { order: 1, ..default() },
        UiCamera,
        IsDefaultUiCamera,
    ));
}

fn spawn_camera(mut commands: Commands, settings: Res<GameSettings>) {
    commands.spawn((
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection {
            fov: settings.fov.to_radians(),
            near: 0.05,
            ..default()
        }),
        DistanceFog {
            color: Color::srgb(0.55, 0.66, 0.82),
            falloff: FogFalloff::from_visibility(350.0),
            ..default()
        },
        Transform::from_xyz(0.0, 5.0, 0.0),
        FpvCamera,
    ));
}

/// Toggles FPV <-> chase (C / East button) and keeps FOV in sync with settings.
fn apply_camera_settings(
    mut settings: ResMut<GameSettings>,
    input: Res<DroneInput>,
    mut cam_q: Query<&mut Projection, With<FpvCamera>>,
    mut flash: ResMut<Flash>,
) {
    let mut changed = settings.is_changed();
    if input.toggle_cam {
        settings.chase_cam = !settings.chase_cam;
        flash.set(
            if settings.chase_cam { "CAMERA: CHASE" } else { "CAMERA: FPV" },
            1.2,
        );
        changed = true;
    }
    if !changed {
        return;
    }
    let fov = settings.fov.to_radians();
    for mut proj in cam_q.iter_mut() {
        if let Projection::Perspective(p) = proj.as_mut() {
            if (p.fov - fov).abs() > 0.001 {
                p.fov = fov;
            }
        }
    }
}

fn follow_drone(
    settings: Res<GameSettings>,
    time: Res<Time>,
    drone_q: Query<&Transform, (With<crate::drone::Drone>, Without<FpvCamera>)>,
    mut cam_q: Query<&mut Transform, With<FpvCamera>>,
) {
    let Ok(drone_t) = drone_q.single() else { return };
    let Ok(mut cam_t) = cam_q.single_mut() else { return };

    if settings.chase_cam {
        // Smoothed chase camera behind the drone.
        let target = drone_t.translation + drone_t.rotation * Vec3::new(0.0, 0.9, 4.5);
        let k = 1.0 - (-6.0 * time.delta_secs()).exp();
        cam_t.translation = cam_t.translation.lerp(target, k);
        cam_t.rotation = Quat::slerp(cam_t.rotation, drone_t.rotation, k);
    } else {
        // Rigid FPV camera with configurable uptilt.
        cam_t.translation = drone_t.translation + drone_t.rotation * Vec3::new(0.0, 0.05, 0.1);
        cam_t.rotation = drone_t.rotation * Quat::from_rotation_x(settings.camera_tilt.to_radians());
    }
}
