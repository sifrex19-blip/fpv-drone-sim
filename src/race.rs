// Race mode: floating gates, sequential gate detection, lap timing.

use bevy::prelude::*;
use crate::AppState;
use crate::drone::Drone;
use crate::hud::{Flash, not_paused};
use crate::input::{DroneInput, gather_input};
use crate::stats::FlightStats;
use crate::theme;

#[derive(Clone, Copy)]
pub struct Gate {
    pub pos: Vec3,
    pub normal: Vec3,
    pub radius: f32,
}

#[derive(Resource, Default)]
pub struct RaceState {
    pub gates: Vec<Gate>,
    pub next_gate: usize,
    pub laps: u32,
    pub lap_time: f32,
    pub best_lap: f32, // seconds; 0 = no lap yet
    pub dist_next: f32,
    prev_sides: Vec<f32>,
}

#[derive(Component)]
struct RaceGate {
    index: usize,
}

#[derive(Component)]
struct RaceGateVisual;

pub struct RacePlugin;
impl Plugin for RacePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RaceState>()
            .add_systems(OnEnter(AppState::Play), setup_race)
            .add_systems(OnExit(AppState::Play), despawn_gates)
            .add_systems(
                Update,
                race_logic
                    .after(gather_input)
                    .run_if(in_state(AppState::Play).and_then(not_paused)),
            );
    }
}

fn despawn_gates(mut commands: Commands, q: Query<Entity, With<RaceGateVisual>>) {
    for e in q.iter() {
        commands.entity(e).despawn();
    }
}

fn setup_race(
    mut commands: Commands,
    mut state: ResMut<RaceState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Oval course east of the spawn point; gates face along the flight direction.
    let center = Vec3::new(30.0, 0.0, -20.0);
    let radius = 35.0f32;
    // Course order matters: the drone spawns around θ≈146°, so gate 0 sits ahead of it
    // and the lap closes back at 120° near the spawn.
    let angles = [180.0f32, 240.0, 300.0, 0.0, 60.0, 120.0];
    let heights = [4.0f32, 4.5, 4.0, 7.0, 5.0, 4.0];

    let ring_mesh = meshes.add(Torus { major_radius: 3.15, minor_radius: 0.15 });
    let disc_mesh = meshes.add(Cylinder::new(1.6, 0.06));

    let mut gates = Vec::with_capacity(angles.len());
    for (i, (deg, h)) in angles.iter().zip(heights.iter()).enumerate() {
        let theta = deg.to_radians();
        let pos = Vec3::new(center.x + radius * theta.cos(), *h, center.z + radius * theta.sin());
        let normal = Vec3::new(-theta.sin(), 0.0, theta.cos()).normalize();
        gates.push(Gate { pos, normal, radius: 3.0 });

        let mat = materials.add(StandardMaterial {
            base_color: theme::accent_dim(),
            emissive: LinearRgba::from(theme::accent_dim()) * 0.6,
            perceptual_roughness: 0.5,
            ..default()
        });
        let rot = Quat::from_rotation_y(normal.x.atan2(normal.z)) * Quat::from_rotation_x(std::f32::consts::FRAC_PI_2);
        commands
            .spawn((
                Mesh3d(ring_mesh.clone()),
                MeshMaterial3d(mat),
                Transform::from_translation(pos).with_rotation(rot),
                RaceGateVisual,
                RaceGate { index: i },
            ))
            .with_children(|ring| {
                // ground marker disc below the gate
                ring.spawn((
                    Mesh3d(disc_mesh.clone()),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: theme::accent().with_alpha(0.55),
                        unlit: true,
                        ..default()
                    })),
                    Transform::from_xyz(0.0, -pos.y + 0.04, 0.0),
                ));
            });
    }

    // Initial sides relative to gate planes from the spawn point.
    let spawn = Vec3::new(0.0, 5.0, 0.0);
    let prev_sides = gates.iter().map(|g| (spawn - g.pos).dot(g.normal)).collect();

    state.gates = gates;
    state.next_gate = 0;
    state.laps = 0;
    state.lap_time = 0.0;
    state.best_lap = 0.0;
    state.dist_next = 0.0;
    state.prev_sides = prev_sides;
}

fn fmt_time(t: f32) -> String {
    let m = (t / 60.0).floor() as u32;
    let s = t - m as f32 * 60.0;
    format!("{m}:{s:04.1}")
}

fn race_logic(
    time: Res<Time>,
    input: Res<DroneInput>,
    mut state: ResMut<RaceState>,
    mut stats: ResMut<FlightStats>,
    mut flash: ResMut<Flash>,
    drone_q: Query<(&Drone, &Transform)>,
    mut visuals: Query<(&RaceGate, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut last_next: Local<Option<usize>>,
) {
    // R restarts the current lap attempt.
    if input.reset {
        state.next_gate = 0;
        state.lap_time = 0.0;
        state.dist_next = 0.0;
        flash.set("RACE RESET", 1.2);
    }

    let Ok((drone, tf)) = drone_q.single() else { return };
    let pos = tf.translation;

    if drone.armed {
        state.lap_time += time.delta_secs();
    }

    // Gate crossing detection.
    if state.gates.is_empty() {
        return;
    }
    if state.prev_sides.len() != state.gates.len() {
        state.prev_sides = vec![0.0; state.gates.len()];
    }
    let armed = drone.armed;
    let mut passed: Option<usize> = None;
    for i in 0..state.gates.len() {
        let g = state.gates[i];
        let side = (pos - g.pos).dot(g.normal);
        let prev = state.prev_sides[i];
        if armed && i == state.next_gate && prev < 0.0 && side >= 0.0 {
            let lateral = (pos - g.pos) - side * g.normal;
            if lateral.length() < g.radius {
                passed = Some(i);
            }
        }
        state.prev_sides[i] = side;
    }

    if let Some(i) = passed {
        if i + 1 >= state.gates.len() {
            // Lap complete.
            let lap = state.lap_time;
            state.laps += 1;
            stats.laps += 1;
            let new_best = state.best_lap == 0.0 || lap < state.best_lap;
            if new_best {
                state.best_lap = lap;
                flash.set(&format!("NEW BEST  {}", fmt_time(lap)), 2.5);
            } else {
                flash.set(&format!("LAP {}   BEST {}", fmt_time(lap), fmt_time(state.best_lap)), 2.5);
            }
            state.next_gate = 0;
            state.lap_time = 0.0;
            crate::stats::write_stats(&stats);
        } else {
            state.next_gate = i + 1;
            flash.set(
                &format!("GATE {}/{}", state.next_gate + 1, state.gates.len()),
                1.0,
            );
        }
    }

    // Distance to the next gate (for the HUD).
    state.dist_next = if state.next_gate < state.gates.len() {
        pos.distance(state.gates[state.next_gate].pos)
    } else {
        0.0
    };

    // Highlight the next gate.
    if *last_next != Some(state.next_gate) {
        *last_next = Some(state.next_gate);
        for (gate, mat) in visuals.iter_mut() {
            if let Some(mut m) = materials.get_mut(&mat.0) {
                if gate.index == state.next_gate {
                    m.base_color = theme::accent();
                    m.emissive = LinearRgba::from(theme::accent()) * 1.2;
                } else {
                    m.base_color = theme::accent_dim();
                    m.emissive = LinearRgba::from(theme::accent_dim()) * 0.6;
                }
            }
        }
    }
}
