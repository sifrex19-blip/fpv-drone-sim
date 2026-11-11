// Drone flight dynamics: quadcopter rigid body with thrust, gravity, drag, torque.

use bevy::prelude::*;
use crate::input::DroneInput;
use crate::AppState;

#[derive(Component)]
pub struct Ground;

#[derive(Component)]
pub struct Drone {
    pub velocity: Vec3,
    pub angular_velocity: Vec3,
    pub armed: bool,
}

#[derive(Resource)]
pub struct DroneConfig {
    pub mass: f32,
    pub inertia: f32,
    pub thrust_max: f32,
    pub torque_strength: f32,
    pub drag: f32,
    pub angular_drag: f32,
    pub gravity: f32,
}

impl Default for DroneConfig {
    fn default() -> Self {
        Self {
            mass: 0.5,
            inertia: 0.02,
            thrust_max: 12.0,
            torque_strength: 6.0,
            drag: 0.5,
            angular_drag: 2.0,
            gravity: 9.81,
        }
    }
}

pub struct DronePlugin;

impl Plugin for DronePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DroneConfig>()
            .add_systems(OnEnter(AppState::InGame), spawn_drone)
            .add_systems(OnExit(AppState::InGame), despawn_drone)
            .add_systems(FixedUpdate, (drone_physics, drone_reset).chain().run_if(in_state(AppState::InGame)));
    }
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
    let ground_mat = materials.add(StandardMaterial { base_color: Color::srgb(0.12,0.3,0.12), perceptual_roughness: 0.9, ..default() });
    commands.spawn((Mesh3d(ground_mesh), MeshMaterial3d(ground_mat), Transform::from_xyz(0.0,0.0,0.0), Ground));
}

fn despawn_drone(mut commands: Commands, q: Query<Entity, Or<(With<Drone>, With<Ground>)>>) {
    for e in q.iter() { commands.entity(e).despawn(); }
}

fn drone_physics(time: Res<Time>, config: Res<DroneConfig>, mut input: ResMut<DroneInput>, settings: Res<crate::settings::GameSettings>, mut query: Query<(&mut Transform, &mut Drone)>) {
    let dt = time.delta_secs();
    let sens = settings.controller_sensitivity;
    let invert = if settings.invert_pitch { -1.0 } else { 1.0 };
    for (mut transform, mut drone) in query.iter_mut() {
        if input.arm { drone.armed = !drone.armed; input.arm = false; }
        if !drone.armed {
            drone.velocity.y -= config.gravity * dt;
            transform.translation += drone.velocity * dt;
            if transform.translation.y < 0.0 { transform.translation.y = 0.0; drone.velocity = Vec3::ZERO; }
            continue;
        }
        let up = transform.up().as_vec3();
        let thrust = input.throttle * config.thrust_max;
        let pitch = input.pitch * sens * invert;
        let roll = input.roll * sens;
        let yaw = input.yaw * sens;
        let torque = Vec3::new(pitch * config.torque_strength, yaw * config.torque_strength, -roll * config.torque_strength);
        let torque_world = transform.rotation * torque;
        let gravity = Vec3::new(0.0, -config.gravity * config.mass, 0.0);
        let drag = -drone.velocity * config.drag;
        let force = up * thrust + gravity + drag;
        let accel = force / config.mass;
        drone.velocity += accel * dt;
        let ang_drag = -drone.angular_velocity * config.angular_drag;
        drone.angular_velocity += (torque_world / config.inertia + ang_drag) * dt;
        transform.translation += drone.velocity * dt;
        let dq = Quat::from_scaled_axis(drone.angular_velocity * dt);
        transform.rotation = (transform.rotation * dq).normalize();
        if transform.translation.y < 0.1 { transform.translation.y = 0.1; drone.velocity.y = drone.velocity.y.max(0.0); drone.velocity.x *= 0.5; drone.velocity.z *= 0.5; }
    }
}

fn drone_reset(input: Res<DroneInput>, mut query: Query<(&mut Transform, &mut Drone)>) {
    if input.reset {
        for (mut t, mut d) in query.iter_mut() {
            t.translation = Vec3::new(0.0, 5.0, 0.0);
            t.rotation = Quat::IDENTITY;
            d.velocity = Vec3::ZERO;
            d.angular_velocity = Vec3::ZERO;
            d.armed = false;
        }
    }
}
