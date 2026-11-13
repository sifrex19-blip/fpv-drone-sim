// Drone flight dynamics: quadcopter rigid body with thrust, gravity, drag.
// FPV-sim style: Betaflight-like "Actual" rates in ACRO mode, self-leveling
// ANGLE mode, battery voltage sag, idle-thrust curve and crash detection.

use bevy::prelude::*;
use crate::input::DroneInput;
use crate::settings::GameSettings;
use crate::hud::{Flash, not_paused};
use crate::AppState;

#[derive(Component)]
pub struct Ground;

#[derive(Component)]
pub struct Drone {
    pub velocity: Vec3,
    /// Body-frame angular velocity (rad/s): x = pitch, y = yaw, z = roll.
    pub angular_velocity: Vec3,
    pub armed: bool,
}

/// 4S LiPo: starts at 16.8 V, sags under load as it drains.
#[derive(Resource, Clone, Copy)]
pub struct Battery {
    pub volts: f32,
}

impl Default for Battery {
    fn default() -> Self {
        Self { volts: 16.8 }
    }
}

impl Battery {
    pub fn full(&mut self) {
        self.volts = 16.8;
    }
    pub fn pct(&self) -> f32 {
        ((self.volts - 13.0) / (16.8 - 13.0)).clamp(0.0, 1.0)
    }
}

#[derive(Resource)]
pub struct DroneConfig {
    pub mass: f32,
    pub thrust_max: f32,
    pub drag: f32,
    pub gravity: f32,
}

impl Default for DroneConfig {
    fn default() -> Self {
        Self {
            mass: 0.5,
            thrust_max: 14.0,
            drag: 0.5,
            gravity: 9.81,
        }
    }
}

pub struct DronePlugin;

impl Plugin for DronePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DroneConfig>()
            .init_resource::<Battery>()
            .add_systems(OnEnter(AppState::Play), (spawn_drone, reset_battery).chain())
            .add_systems(OnExit(AppState::Play), despawn_drone)
            .add_systems(
                FixedUpdate,
                (drone_physics.run_if(not_paused), drone_reset)
                    .chain()
                    .run_if(in_state(AppState::Play)),
            );
    }
}

fn reset_battery(mut battery: ResMut<Battery>) {
    battery.full();
}

fn spawn_drone(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Improved drone model: body + 4 arms + 4 prop hubs
    let body = meshes.add(Cuboid::new(0.18, 0.06, 0.18));
    let arm = meshes.add(Cuboid::new(0.32, 0.02, 0.03));
    let hub = meshes.add(Cylinder::new(0.04, 0.02));
    let body_mat = materials.add(StandardMaterial { base_color: Color::srgb(0.15,0.15,0.2), metallic: 0.6, perceptual_roughness: 0.4, ..default() });
    let arm_mat = materials.add(StandardMaterial { base_color: Color::srgb(0.25,0.25,0.3), ..default() });
    let hub_mat = materials.add(StandardMaterial { base_color: Color::srgb(0.9,0.2,0.2), emissive: LinearRgba::rgb(0.5,0.1,0.1), ..default() });

    commands.spawn((
        Mesh3d(body), MeshMaterial3d(body_mat.clone()),
        Transform::from_xyz(0.0, 5.0, 0.0),
        Drone { velocity: Vec3::ZERO, angular_velocity: Vec3::ZERO, armed: false },
    )).with_children(|p| {
        for (x,z) in [(0.16f32,0.16f32),(-0.16,0.16),(0.16,-0.16),(-0.16,-0.16)] {
            p.spawn((Mesh3d(arm.clone()), MeshMaterial3d(arm_mat.clone()), Transform::from_xyz(x*0.5,0.0,z*0.5).with_rotation(Quat::from_rotation_y(if x*z>0.0{0.785}else{-0.785}))));
            p.spawn((Mesh3d(hub.clone()), MeshMaterial3d(hub_mat.clone()), Transform::from_xyz(x,0.03,z)));
        }
    });
    let ground_mesh = meshes.add(Plane3d::default().mesh().size(400.0, 400.0));
    let ground_mat = materials.add(StandardMaterial { base_color: Color::srgb(0.16,0.34,0.16), perceptual_roughness: 0.9, ..default() });
    commands.spawn((Mesh3d(ground_mesh), MeshMaterial3d(ground_mat), Transform::from_xyz(0.0,0.0,0.0), Ground));
}

fn despawn_drone(mut commands: Commands, q: Query<Entity, Or<(With<Drone>, With<Ground>)>>) {
    for e in q.iter() { commands.entity(e).despawn(); }
}

/// Betaflight-style "Actual" rates: center sensitivity + max rate + stick expo.
fn rate_output(stick: f32, center: f32, max_rate: f32, expo: f32) -> f32 {
    let s = if expo > 0.0 {
        stick * (1.0 - expo) + stick * stick * stick * expo
    } else {
        stick
    };
    center * s + (max_rate - center) * s * s * s
}

#[allow(clippy::too_many_arguments)]
fn drone_physics(
    time: Res<Time>,
    config: Res<DroneConfig>,
    mut input: ResMut<DroneInput>,
    mut settings: ResMut<GameSettings>,
    mut battery: ResMut<Battery>,
    mut stats: ResMut<crate::stats::FlightStats>,
    mut flash: ResMut<Flash>,
    mut query: Query<(&mut Transform, &mut Drone)>,
) {
    let dt = time.delta_secs();

    if input.toggle_mode {
        settings.flight_mode_acro = !settings.flight_mode_acro;
        flash.set(
            if settings.flight_mode_acro { "MODE: ACRO" } else { "MODE: ANGLE" },
            1.2,
        );
        input.toggle_mode = false;
    }

    let acro = settings.flight_mode_acro;
    let sens = settings.controller_sensitivity;
    let invert = if settings.invert_pitch { -1.0 } else { 1.0 };
    let roll_cmd = (input.roll * sens).clamp(-1.0, 1.0);
    let pitch_cmd = (input.pitch * sens * invert).clamp(-1.0, 1.0);
    let yaw_cmd = (input.yaw * sens).clamp(-1.0, 1.0);
    let center = settings.rate_center.to_radians() * settings.rotation_multiplier();
    let max_rate = settings.rate_max.to_radians() * settings.rotation_multiplier();
    let expo = settings.expo;
    let max_tilt = settings.max_tilt.to_radians();

    for (mut transform, mut drone) in query.iter_mut() {
        if input.arm {
            drone.armed = !drone.armed;
            drone.velocity = Vec3::ZERO;
            drone.angular_velocity = Vec3::ZERO;
            input.arm = false;
        }

        if !drone.armed {
            // Falling / resting while disarmed.
            drone.velocity.y -= config.gravity * dt;
            transform.translation += drone.velocity * dt;
            if transform.translation.y < 0.1 {
                transform.translation.y = 0.1;
                if drone.velocity.y < -6.0 {
                    stats.crashes += 1;
                    crate::stats::write_stats(&stats);
                    flash.set("CRASH", 1.4);
                }
                drone.velocity = Vec3::ZERO;
            }
            continue;
        }

        // --- Battery drain + voltage sag ---
        let t_lin = input.throttle.clamp(0.0, 1.0).powf(1.3);
        battery.volts = (battery.volts - (0.006 + t_lin * 0.022) * dt).max(13.0);
        let sag = ((battery.volts - 13.0) / (16.8 - 13.0)).clamp(0.5, 1.0);

        // --- Thrust: motor idle + nonlinear throttle curve ---
        let idle = config.thrust_max * 0.06;
        let thrust = idle + (config.thrust_max * sag - idle) * t_lin;

        let up = transform.up().as_vec3();
        let gravity = Vec3::new(0.0, -config.gravity * config.mass, 0.0);
        let drag = -drone.velocity * config.drag;
        let force = up * thrust + gravity + drag;
        drone.velocity += (force / config.mass) * dt;

        // --- Attitude: body-frame angular velocity ---
        let mut omega = drone.angular_velocity;
        if acro {
            // Rate mode: stick deflection commands rotation rate directly.
            let target = Vec3::new(
                rate_output(pitch_cmd, center, max_rate, expo),
                rate_output(yaw_cmd, center, max_rate, expo),
                rate_output(-roll_cmd, center, max_rate, expo),
            );
            let k = 1.0 - (-dt / 0.07).exp();
            omega += (target - omega) * k;
        } else {
            // Angle mode: aim the craft's up-vector at the commanded tilt.
            let up_body = transform.rotation.inverse() * Vec3::Y;
            let m = max_tilt.tan();
            let desired = Vec3::new(roll_cmd * m, 1.0, pitch_cmd * m).normalize();
            let err = desired - up_body;
            let target = Vec3::new(err.z * 6.0, rate_output(yaw_cmd, center, max_rate, expo), -err.x * 6.0)
                .clamp_length_max(max_rate * 0.7);
            let k = 1.0 - (-dt / 0.09).exp();
            omega += (target - omega) * k;
        }
        drone.angular_velocity = omega;

        transform.translation += drone.velocity * dt;
        let dq = Quat::from_scaled_axis(omega * dt);
        transform.rotation = (transform.rotation * dq).normalize();

        // --- Ground contact / crash ---
        if transform.translation.y < 0.1 {
            transform.translation.y = 0.1;
            let impact = drone.velocity.y;
            drone.velocity.y = drone.velocity.y.max(0.0);
            drone.velocity.x *= 0.5;
            drone.velocity.z *= 0.5;
            if impact < -6.0 {
                drone.armed = false;
                drone.angular_velocity = Vec3::ZERO;
                stats.crashes += 1;
                crate::stats::write_stats(&stats);
                flash.set("CRASH", 1.4);
            }
        }
    }
}

fn drone_reset(
    input: Res<DroneInput>,
    mut battery: ResMut<Battery>,
    mut query: Query<(&mut Transform, &mut Drone)>,
) {
    if input.reset {
        for (mut t, mut d) in query.iter_mut() {
            t.translation = Vec3::new(0.0, 5.0, 0.0);
            t.rotation = Quat::IDENTITY;
            d.velocity = Vec3::ZERO;
            d.angular_velocity = Vec3::ZERO;
            d.armed = false;
        }
        battery.full();
    }
}
