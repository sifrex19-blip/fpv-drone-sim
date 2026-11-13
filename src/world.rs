// World scenery for the Play state: runway, pad, buildings, trees, hills, clouds.
// Also switches the clear color to a daytime sky (and back on exit).

use bevy::prelude::*;
use crate::AppState;

#[derive(Component)]
struct WorldProp;

pub struct WorldPlugin;
impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Play), (setup_world, set_sky).chain())
            .add_systems(OnExit(AppState::Play), (despawn_world, restore_sky).chain());
    }
}

fn set_sky(mut clear: ResMut<ClearColor>) {
    clear.0 = Color::srgb(0.40, 0.58, 0.85);
}

fn restore_sky(mut clear: ResMut<ClearColor>) {
    clear.0 = Color::srgb(0.05, 0.05, 0.08);
}

fn despawn_world(mut commands: Commands, q: Query<Entity, With<WorldProp>>) {
    for e in q.iter() {
        commands.entity(e).despawn();
    }
}

/// Tiny deterministic PRNG so the scenery is identical every run.
struct Rng(u32);
impl Rng {
    fn next(&mut self) -> f32 {
        self.0 = self.0.wrapping_mul(1664525).wrapping_add(1013904223);
        ((self.0 >> 8) as f32) / ((1u32 << 24) as f32)
    }
    fn range(&mut self, lo: f32, hi: f32) -> f32 {
        lo + (hi - lo) * self.next()
    }
}

fn setup_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // ---- shared assets (unit meshes scaled per prop) ----
    let unit_box = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let unit_cyl = meshes.add(Cylinder::new(1.0, 1.0));
    let unit_cone = meshes.add(Cone::new(1.0, 1.0));
    let unit_sphere = meshes.add(Sphere::new(1.0));
    let unit_plane = meshes.add(Plane3d::default().mesh().size(1.0, 1.0));
    let pad_ring = meshes.add(Torus { major_radius: 3.4, minor_radius: 0.2 });

    let runway_mat = materials.add(StandardMaterial { base_color: Color::srgb(0.16, 0.16, 0.18), perceptual_roughness: 0.85, ..default() });
    let paint_mat = materials.add(StandardMaterial { base_color: Color::srgb(0.9, 0.9, 0.92), ..default() });
    let pad_mat = materials.add(StandardMaterial { base_color: Color::srgb(0.22, 0.22, 0.27), perceptual_roughness: 0.8, ..default() });
    let accent_mat = materials.add(StandardMaterial {
        base_color: crate::theme::accent(),
        emissive: LinearRgba::from(crate::theme::accent()) * 0.9,
        ..default()
    });
    let building_mats = [
        materials.add(StandardMaterial { base_color: Color::srgb(0.34, 0.36, 0.44), perceptual_roughness: 0.7, ..default() }),
        materials.add(StandardMaterial { base_color: Color::srgb(0.42, 0.38, 0.40), perceptual_roughness: 0.7, ..default() }),
        materials.add(StandardMaterial { base_color: Color::srgb(0.28, 0.34, 0.46), perceptual_roughness: 0.7, ..default() }),
        materials.add(StandardMaterial { base_color: Color::srgb(0.46, 0.44, 0.38), perceptual_roughness: 0.7, ..default() }),
    ];
    let trunk_mat = materials.add(StandardMaterial { base_color: Color::srgb(0.32, 0.22, 0.14), perceptual_roughness: 0.95, ..default() });
    let leaf_mat = materials.add(StandardMaterial { base_color: Color::srgb(0.13, 0.38, 0.18), perceptual_roughness: 0.9, ..default() });
    let leaf_mat2 = materials.add(StandardMaterial { base_color: Color::srgb(0.17, 0.45, 0.22), perceptual_roughness: 0.9, ..default() });
    let hill_mat = materials.add(StandardMaterial { base_color: Color::srgb(0.20, 0.34, 0.26), perceptual_roughness: 1.0, ..default() });
    let cloud_mat = materials.add(StandardMaterial { base_color: Color::srgb(0.95, 0.96, 1.0), unlit: true, ..default() });

    fn prop(mut e: EntityCommands) -> EntityCommands {
        e.insert(WorldProp);
        e
    }

    // ---- runway along -Z with centerline dashes and edge lines ----
    prop(commands.spawn((
        Mesh3d(unit_plane.clone()),
        MeshMaterial3d(runway_mat.clone()),
        Transform::from_xyz(0.0, 0.02, -30.0).with_scale(Vec3::new(12.0, 1.0, 90.0)),
    )));
    for i in 0..8 {
        prop(commands.spawn((
            Mesh3d(unit_plane.clone()),
            MeshMaterial3d(paint_mat.clone()),
            Transform::from_xyz(0.0, 0.035, 10.0 - i as f32 * 11.0).with_scale(Vec3::new(0.5, 1.0, 4.0)),
        )));
    }
    for x in [-5.5f32, 5.5] {
        prop(commands.spawn((
            Mesh3d(unit_plane.clone()),
            MeshMaterial3d(paint_mat.clone()),
            Transform::from_xyz(x, 0.035, -30.0).with_scale(Vec3::new(0.3, 1.0, 90.0)),
        )));
    }

    // ---- landing / start pad east of the runway ----
    prop(commands.spawn((
        Mesh3d(unit_cyl.clone()),
        MeshMaterial3d(pad_mat.clone()),
        Transform::from_xyz(9.0, 0.03, 6.0).with_scale(Vec3::new(4.2, 0.06, 4.2)),
    )));
    prop(commands.spawn((
        Mesh3d(pad_ring.clone()),
        MeshMaterial3d(accent_mat.clone()),
        Transform::from_xyz(9.0, 0.05, 6.0),
    )));

    // ---- buildings (boxes with accent roof trims) ----
    let buildings: [(f32, f32, f32, f32, f32); 11] = [
        (-55.0, -45.0, 10.0, 22.0, 10.0),
        (-70.0, -20.0, 8.0, 14.0, 8.0),
        (-48.0, -70.0, 12.0, 30.0, 9.0),
        (-85.0, -55.0, 9.0, 18.0, 9.0),
        (-35.0, -52.0, 7.0, 12.0, 7.0),
        (75.0, -55.0, 11.0, 25.0, 11.0),
        (92.0, -28.0, 8.0, 15.0, 8.0),
        (70.0, -82.0, 10.0, 20.0, 10.0),
        (-62.0, 14.0, 9.0, 16.0, 9.0),
        (-46.0, 38.0, 8.0, 11.0, 8.0),
        (84.0, 24.0, 10.0, 18.0, 10.0),
    ];
    for (i, (x, z, w, h, d)) in buildings.iter().enumerate() {
        let mat = building_mats[i % building_mats.len()].clone();
        prop(commands.spawn((
            Mesh3d(unit_box.clone()),
            MeshMaterial3d(mat),
            Transform::from_xyz(*x, h / 2.0, *z).with_scale(Vec3::new(*w, *h, *d)),
        )));
        // accent roof trim
        prop(commands.spawn((
            Mesh3d(unit_box.clone()),
            MeshMaterial3d(accent_mat.clone()),
            Transform::from_xyz(*x, h + 0.15, *z).with_scale(Vec3::new(w * 1.03, 0.3, d * 1.03)),
        )));
    }

    // ---- trees scattered outside the flight line ----
    let mut rng = Rng(0x00C0FFEE);
    let mut placed = 0;
    let mut attempts = 0;
    while placed < 26 && attempts < 300 {
        attempts += 1;
        let ang = rng.range(0.0, std::f32::consts::TAU);
        let dist = rng.range(42.0, 130.0);
        let x = ang.cos() * dist;
        let z = ang.sin() * dist;
        // keep clear of the runway strip
        if x.abs() < 12.0 && z > -85.0 && z < 18.0 {
            continue;
        }
        let s = rng.range(0.8, 1.4);
        prop(commands.spawn((
            Mesh3d(unit_cyl.clone()),
            MeshMaterial3d(trunk_mat.clone()),
            Transform::from_xyz(x, 1.1 * s, z).with_scale(Vec3::new(0.5 * s, 2.2 * s, 0.5 * s)),
        )));
        prop(commands.spawn((
            Mesh3d(unit_cone.clone()),
            MeshMaterial3d(leaf_mat.clone()),
            Transform::from_xyz(x, 4.4 * s, z).with_scale(Vec3::new(2.6 * s, 5.2 * s, 2.6 * s)),
        )));
        prop(commands.spawn((
            Mesh3d(unit_cone.clone()),
            MeshMaterial3d(leaf_mat2.clone()),
            Transform::from_xyz(x, 7.4 * s, z).with_scale(Vec3::new(1.7 * s, 3.6 * s, 1.7 * s)),
        )));
        placed += 1;
    }

    // ---- horizon hills ----
    let mut hills_rng = Rng(0xBEEF1234);
    for _ in 0..8 {
        let ang = hills_rng.range(0.0, std::f32::consts::TAU);
        let dist = hills_rng.range(170.0, 240.0);
        let r = hills_rng.range(22.0, 42.0);
        prop(commands.spawn((
            Mesh3d(unit_sphere.clone()),
            MeshMaterial3d(hill_mat.clone()),
            Transform::from_xyz(ang.cos() * dist, -r * 0.45, ang.sin() * dist)
                .with_scale(Vec3::new(r * 1.7, r, r * 1.4)),
        )));
    }

    // ---- clouds ----
    let mut cloud_rng = Rng(0x1CEC100D);
    for _ in 0..7 {
        let x = cloud_rng.range(-160.0, 160.0);
        let z = cloud_rng.range(-160.0, 160.0);
        let y = cloud_rng.range(70.0, 105.0);
        prop(commands.spawn((
            Mesh3d(unit_sphere.clone()),
            MeshMaterial3d(cloud_mat.clone()),
            Transform::from_xyz(x, y, z).with_scale(Vec3::new(16.0, 3.5, 9.0)),
        )));
    }
}
