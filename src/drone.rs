// Drone flight dynamics: quadcopter rigid body with thrust, gravity, drag, torque.

use bevy::prelude::*;
use crate::input::DroneInput;

#[derive(Component)]
pub struct Drone {
    pub velocity: Vec3,
    pub angular_velocity: Vec3,
    pub armed: bool,
}

#[derive(Resource)]
pub struct DroneConfig {
    pub mass: f32,
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
            .add_systems(Startup, spawn_drone)
            .add_systems(FixedUpdate, (drone_physics, drone_reset).chain());
    }
}

fn spawn_drone(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = meshes.add(Cuboid::new(0.3, 0.08, 0.3));
    let mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.2, 0.2, 0.25),
        ..default()
    });
    commands.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(mat),
        Transform::from_xyz(0.0, 5.0, 0.0),
        Drone {
            velocity: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
            armed: false,
        },
    ));
    // Ground plane
    let ground_mesh = meshes.add(Plane3d::default().mesh().size(200.0, 200.0));
    let ground_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.15, 0.35, 0.15),
        ..default()
    });
    commands.spawn((
        Mesh3d(ground_mesh),
        MeshMaterial3d(ground_mat),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
}

fn drone_physics(
    time: Res<Time>,
    config: Res<DroneConfig>,
    input: Res<DroneInput>,
    mut query: Query<(&mut Transform, &mut Drone)>,
) {
    let dt = time.delta_secs();
    for (mut transform, mut drone) in query.iter_mut() {
        if input.arm {
            drone.armed = !drone.armed;
        }
        if !drone.armed {
            // Disarmed: just gravity
            drone.velocity.y -= config.gravity * dt;
            transform.translation += drone.velocity * dt;
            if transform.translation.y < 0.0 {
                transform.translation.y = 0.0;
                drone.velocity = Vec3::ZERO;
            }
            continue;
        }

        let forward = transform.forward().as_vec3();
        let up = transform.up().as_vec3();
        let right = transform.right().as_vec3();

        // Thrust
        let thrust = input.throttle * config.thrust_max;
        let thrust_force = up * thrust;

        // Torques from sticks
        let torque = Vec3::new(
            input.pitch * config.torque_strength,
            input.yaw * config.torque_strength,
            -input.roll * config.torque_strength,
        );
        // torque in local space -> world
        let torque_world = transform.rotation * torque;

        // Forces
        let gravity = Vec3::new(0.0, -config.gravity * config.mass, 0.0);
        let drag = -drone.velocity * config.drag;
        let force = thrust_force + gravity + drag;

        let accel = force / config.mass;
        drone.velocity += accel * dt;

        // Angular
        let ang_drag = -drone.angular_velocity * config.angular_drag;
        drone.angular_velocity += (torque_world / config.mass + ang_drag) * dt;

        // Integrate
        transform.translation += drone.velocity * dt;
        let dq = Quat::from_scaled_axis(drone.angular_velocity * dt);
        transform.rotation = (transform.rotation * dq).normalize();

        // Ground collision
        if transform.translation.y < 0.1 {
            transform.translation.y = 0.1;
            drone.velocity.y = drone.velocity.y.max(0.0);
            drone.velocity.x *= 0.5;
            drone.velocity.z *= 0.5;
        }
    }
}

fn drone_reset(
    input: Res<DroneInput>,
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
    }
}
