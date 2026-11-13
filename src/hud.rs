// In-game HUD (OSD-style) + pause menu.

use bevy::prelude::*;
use crate::AppState;
use crate::drone::{Battery, Drone};
use crate::input::DroneInput;
use crate::race::RaceState;
use crate::settings::GameSettings;
use crate::theme::{self, text};

#[derive(Resource, Default)]
pub struct Pause(pub bool);

pub fn not_paused(pause: Res<Pause>) -> bool {
    !pause.0
}

/// Short centered message shown in-game (crash, mode switch, lap events...).
#[derive(Resource, Default)]
pub struct Flash {
    pub text: String,
    pub timer: f32,
}

impl Flash {
    pub fn set(&mut self, text: &str, secs: f32) {
        self.text = text.to_string();
        self.timer = secs;
    }
}

#[derive(Component)] struct HudRoot;
#[derive(Component)] struct OsdText;
#[derive(Component)] struct ArmText;
#[derive(Component)] struct BatteryText;
#[derive(Component)] struct BatteryBarFill;
#[derive(Component)] struct RaceText;
#[derive(Component)] struct GateText;
#[derive(Component)] struct FlashText;
#[derive(Component)] struct PauseOverlay;
#[derive(Component)] struct ResumeBtn;
#[derive(Component)] struct PauseResetBtn;
#[derive(Component)] struct PauseQuitBtn;

pub struct HudPlugin;
impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Pause>()
            .init_resource::<Flash>()
            .add_systems(OnEnter(AppState::Play), (spawn_hud, reset_flash).chain())
            .add_systems(OnExit(AppState::Play), (despawn_hud, reset_pause).chain())
            .add_systems(
                Update,
                (toggle_pause, pause_buttons, update_hud)
                    .run_if(in_state(AppState::Play)),
            );
    }
}

fn reset_flash(mut flash: ResMut<Flash>) {
    flash.timer = 0.0;
    flash.text.clear();
}

fn reset_pause(mut pause: ResMut<Pause>) {
    pause.0 = false;
}

fn despawn_hud(mut commands: Commands, q: Query<Entity, With<HudRoot>>) {
    for e in q.iter() {
        commands.entity(e).despawn();
    }
}

fn fmt_time(t: f32) -> String {
    let m = (t / 60.0).floor() as u32;
    let s = t - m as f32 * 60.0;
    format!("{m}:{s:04.1}")
}

fn spawn_hud(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            HudRoot,
        ))
        .with_children(|root| {
            // ---- top-left OSD ----
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(16.0),
                    top: Val::Px(16.0),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect {
                        left: Val::Px(14.0),
                        right: Val::Px(14.0),
                        top: Val::Px(12.0),
                        bottom: Val::Px(12.0),
                    },
                    row_gap: Val::Px(6.0),
                    border_radius: BorderRadius::all(Val::Px(12.0)),
                    ..default()
                },
                BackgroundColor(theme::osd_bg()),
            ))
            .with_children(|osd| {
                // battery row
                osd.spawn((Node { column_gap: Val::Px(8.0), align_items: AlignItems::Center, ..default() },))
                    .with_children(|brow| {
                        brow.spawn((Node { width: Val::Px(9.0), height: Val::Px(9.0), border_radius: BorderRadius::all(Val::Px(5.0)), ..default() }, BackgroundColor(theme::green())));
                        brow.spawn(text("16.8 V", 13.0, theme::green())).insert(BatteryText);
                    });
                // battery bar
                osd.spawn((
                    Node {
                        width: Val::Px(150.0),
                        height: Val::Px(7.0),
                        border_radius: BorderRadius::all(Val::Px(4.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.12)),
                ))
                .with_children(|bar| {
                    bar.spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            border_radius: BorderRadius::all(Val::Px(4.0)),
                            ..default()
                        },
                        BackgroundColor(theme::green()),
                        BatteryBarFill,
                    ));
                });
                osd.spawn((
                    Text::new("SPEED  0.0 km/h\nALT    0.0 m\nCAM    FPV"),
                    TextFont { font_size: bevy::text::FontSize::Px(14.0), ..default() },
                    TextColor(Color::srgba(1.0, 1.0, 1.0, 0.92)),
                    OsdText,
                ));
                osd.spawn((
                    Text::new("• DISARMED"),
                    TextFont { font_size: bevy::text::FontSize::Px(14.0), ..default() },
                    TextColor(theme::text_dim()),
                    ArmText,
                ));
            });

            // ---- top-center race bar ----
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(16.0),
                    width: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
            ))
            .with_children(|wrap| {
                wrap.spawn((
                    Node {
                        padding: UiRect {
                            left: Val::Px(18.0),
                            right: Val::Px(18.0),
                            top: Val::Px(9.0),
                            bottom: Val::Px(9.0),
                        },
                        border_radius: BorderRadius::all(Val::Px(10.0)),
                        column_gap: Val::Px(16.0),
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(theme::osd_bg()),
                ))
                .with_children(|card| {
                    card.spawn((
                        Text::new("LAP 0:00.0"),
                        TextFont { font_size: bevy::text::FontSize::Px(15.0), ..default() },
                        TextColor(Color::WHITE),
                        RaceText,
                    ));
                    card.spawn((
                        Text::new("GATE 1/1"),
                        TextFont { font_size: bevy::text::FontSize::Px(13.0), ..default() },
                        TextColor(theme::accent()),
                        GateText,
                    ));
                });
            });

            // ---- crosshair ----
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
            ))
            .with_children(|wrap| {
                wrap.spawn((Node { width: Val::Px(26.0), height: Val::Px(26.0), position_type: PositionType::Relative, ..default() },))
                    .with_children(|cross| {
                        cross.spawn((
                            Node {
                                position_type: PositionType::Absolute,
                                left: Val::Px(0.0),
                                top: Val::Px(12.0),
                                width: Val::Px(26.0),
                                height: Val::Px(2.0),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.7)),
                        ));
                        cross.spawn((
                            Node {
                                position_type: PositionType::Absolute,
                                left: Val::Px(12.0),
                                top: Val::Px(0.0),
                                width: Val::Px(2.0),
                                height: Val::Px(26.0),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.7)),
                        ));
                    });
            });

            // ---- flash message ----
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    bottom: Val::Px(96.0),
                    width: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
            ))
            .with_children(|wrap| {
                wrap.spawn((
                    text("", 26.0, theme::accent()).0,
                    TextFont { font_size: bevy::text::FontSize::Px(26.0), ..default() },
                    TextColor(theme::accent()),
                    FlashText,
                    Visibility::Hidden,
                ));
            });

            // ---- controls hint ----
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(16.0),
                    bottom: Val::Px(12.0),
                    ..default()
                },
            ))
            .with_children(|hint| {
                hint.spawn(text(
                    "ENTER arm · R reset · M mode · C camera · ESC pause",
                    12.0,
                    Color::srgba(1.0, 1.0, 1.0, 0.45),
                ));
            });

            // ---- pause overlay ----
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(Color::srgba(0.02, 0.02, 0.04, 0.6)),
                PauseOverlay,
                Visibility::Hidden,
            ))
            .with_children(|overlay| {
                overlay
                    .spawn((
                        Node {
                            flex_direction: FlexDirection::Column,
                            padding: UiRect::all(Val::Px(30.0)),
                            row_gap: Val::Px(16.0),
                            border_radius: BorderRadius::all(Val::Px(16.0)),
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(theme::card()),
                    ))
                    .with_children(|panel| {
                        panel.spawn(text("PAUSED", 30.0, Color::WHITE));
                        panel.spawn(text(
                            "ENTER arm · R reset · M flight mode · C camera",
                            13.0,
                            theme::text_dim(),
                        ));
                        // buttons row
                        panel
                            .spawn((Node { column_gap: Val::Px(12.0), ..default() },))
                            .with_children(|row| {
                                row.spawn((
                                    Node {
                                        width: Val::Px(170.0),
                                        height: Val::Px(44.0),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        border_radius: BorderRadius::all(Val::Px(10.0)),
                                        ..default()
                                    },
                                    BackgroundColor(theme::accent()),
                                    theme::HoverTint { base: theme::accent(), hover: theme::accent_hover() },
                                    ResumeBtn,
                                    Interaction::default(),
                                ))
                                .with_children(|b| {
                                    b.spawn(text("RESUME", 15.0, Color::WHITE));
                                });
                                row.spawn((
                                    Node {
                                        width: Val::Px(170.0),
                                        height: Val::Px(44.0),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        border_radius: BorderRadius::all(Val::Px(10.0)),
                                        ..default()
                                    },
                                    BackgroundColor(theme::row()),
                                    theme::HoverTint { base: theme::row(), hover: theme::row_hover() },
                                    PauseResetBtn,
                                    Interaction::default(),
                                ))
                                .with_children(|b| {
                                    b.spawn(text("RESET DRONE", 15.0, Color::WHITE));
                                });
                                row.spawn((
                                    Node {
                                        width: Val::Px(170.0),
                                        height: Val::Px(44.0),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        border_radius: BorderRadius::all(Val::Px(10.0)),
                                        ..default()
                                    },
                                    BackgroundColor(theme::red()),
                                    theme::HoverTint { base: theme::red(), hover: theme::red_hover() },
                                    PauseQuitBtn,
                                    Interaction::default(),
                                ))
                                .with_children(|b| {
                                    b.spawn(text("QUIT TO MENU", 15.0, Color::WHITE));
                                });
                            });
                    });
            });
        });
}

fn toggle_pause(
    keys: Res<ButtonInput<KeyCode>>,
    mut pause: ResMut<Pause>,
    mut overlay_q: Query<&mut Visibility, With<PauseOverlay>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        pause.0 = !pause.0;
        for mut v in overlay_q.iter_mut() {
            *v = if pause.0 { Visibility::Visible } else { Visibility::Hidden };
        }
    }
}

fn set_overlay_visible(pause: bool, overlay_q: &mut Query<&mut Visibility, With<PauseOverlay>>) {
    for mut v in overlay_q.iter_mut() {
        *v = if pause { Visibility::Visible } else { Visibility::Hidden };
    }
}

fn pause_buttons(
    mut pause: ResMut<Pause>,
    mut overlay_q: Query<&mut Visibility, With<PauseOverlay>>,
    mut input: ResMut<DroneInput>,
    mut next: ResMut<NextState<AppState>>,
    q: Query<
        (
            &Interaction,
            Option<&ResumeBtn>,
            Option<&PauseResetBtn>,
            Option<&PauseQuitBtn>,
        ),
        Changed<Interaction>,
    >,
) {
    for (interaction, resume, reset, quit) in q.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }
        if resume.is_some() {
            pause.0 = false;
            set_overlay_visible(false, &mut overlay_q);
        } else if reset.is_some() {
            input.reset = true;
            pause.0 = false;
            set_overlay_visible(false, &mut overlay_q);
        } else if quit.is_some() {
            pause.0 = false;
            next.set(AppState::Home);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn update_hud(
    time: Res<Time>,
    settings: Res<GameSettings>,
    battery: Res<Battery>,
    race: Res<RaceState>,
    drone_q: Query<(&Drone, &Transform)>,
    mut flash: ResMut<Flash>,
    mut params: ParamSet<(
        Query<&mut Text, With<OsdText>>,
        Query<(&mut Text, &mut TextColor), With<ArmText>>,
        Query<&mut Text, With<BatteryText>>,
        Query<&mut Node, With<BatteryBarFill>>,
        Query<&mut Text, With<RaceText>>,
        Query<(&mut Text, &mut Visibility), With<FlashText>>,
    )>,
) {
    // Flash decay
    if flash.timer > 0.0 {
        flash.timer -= time.delta_secs();
        for (mut t, mut vis) in params.p5().iter_mut() {
            t.0 = flash.text.clone();
            *vis = Visibility::Visible;
        }
        if flash.timer <= 0.0 {
            for (_, mut vis) in params.p5().iter_mut() {
                *vis = Visibility::Hidden;
            }
        }
    } else {
        for (_, mut vis) in params.p5().iter_mut() {
            *vis = Visibility::Hidden;
        }
    }

    let fake_drone = Drone { velocity: Vec3::ZERO, angular_velocity: Vec3::ZERO, armed: false };
    let fake_tf = Transform::IDENTITY;
    let (drone, tf) = drone_q.single().unwrap_or((&fake_drone, &fake_tf));

    let speed = drone.velocity.length() * 3.6;
    let alt = tf.translation.y;
    let cam = if settings.chase_cam { "CHASE" } else { "FPV" };
    for mut t in params.p0().iter_mut() {
        t.0 = format!(
            "SPEED  {speed:>6.1} km/h\nALT    {alt:>6.1} m\nMODE   {} · {cam}",
            settings.flight_mode_label()
        );
    }

    for (mut t, mut c) in params.p1().iter_mut() {
        if drone.armed {
            t.0 = "• ARMED".to_string();
            c.0 = theme::green();
        } else {
            t.0 = "• DISARMED".to_string();
            c.0 = theme::text_dim();
        }
    }

    for mut t in params.p2().iter_mut() {
        t.0 = format!("{:.1} V  {:>3.0}%", battery.volts, battery.pct() * 100.0);
    }
    for mut n in params.p3().iter_mut() {
        n.width = Val::Percent(battery.pct() * 100.0);
    }

    for mut t in params.p4().iter_mut() {
        let best = if race.best_lap > 0.0 {
            fmt_time(race.best_lap)
        } else {
            "--:--.-".to_string()
        };
        t.0 = format!(
            "LAP {}   BEST {}   GATE {}/{}   → {:.0} m",
            fmt_time(race.lap_time),
            best,
            race.next_gate + 1,
            race.gates.len().max(1),
            race.dist_next
        );
    }
}
