// UI: main menu, in-game HUD, settings panel. Driven by AppState.

use bevy::prelude::*;
use crate::AppState;
use crate::settings::GameSettings;
use crate::input::DroneInput;
use crate::drone::Drone;

#[derive(Component)] struct MenuRoot;
#[derive(Component)] struct SettingsRoot;
#[derive(Component)] struct HudRoot;
#[derive(Component)] struct PlayBtn;
#[derive(Component)] struct SettingsBtn;
#[derive(Component)] struct QuitBtn;
#[derive(Component)] struct BackBtn;

pub struct UiPlugin;
impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Menu), spawn_menu)
            .add_systems(OnExit(AppState::Menu), despawn::<MenuRoot>)
            .add_systems(OnEnter(AppState::Settings), spawn_settings)
            .add_systems(OnExit(AppState::Settings), despawn::<SettingsRoot>)
            .add_systems(OnEnter(AppState::InGame), spawn_hud)
            .add_systems(OnExit(AppState::InGame), despawn::<HudRoot>)
            .add_systems(Update, (menu_buttons, settings_buttons, update_hud.run_if(in_state(AppState::InGame)), pause_on_esc.run_if(in_state(AppState::InGame))));
    }
}

fn spawn_menu(mut commands: Commands) {
    commands.spawn((Node { width: Val::Percent(100.0), height: Val::Percent(100.0), flex_direction: FlexDirection::Column, align_items: AlignItems::Center, justify_content: JustifyContent::Center, row_gap: Val::Px(20.0), ..default() }, BackgroundColor(Color::srgb(0.05,0.05,0.1)), MenuRoot))
        .with_children(|p| {
            p.spawn((Text::new("FPV DRONE SIM"), TextFont { font_size: bevy::text::FontSize::Px(48.0), ..default() }, TextColor(Color::WHITE)));
            p.spawn((Node { width: Val::Px(200.0), height: Val::Px(50.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() }, BackgroundColor(Color::srgb(0.2,0.4,0.8)), PlayBtn, Interaction::None)).with_children(|p| { p.spawn((Text::new("PLAY"), TextFont{font_size: bevy::text::FontSize::Px(24.0),..default()}, TextColor(Color::WHITE))); });
            p.spawn((Node { width: Val::Px(200.0), height: Val::Px(50.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() }, BackgroundColor(Color::srgb(0.3,0.3,0.3)), SettingsBtn, Interaction::None)).with_children(|p| { p.spawn((Text::new("SETTINGS"), TextFont{font_size: bevy::text::FontSize::Px(24.0),..default()}, TextColor(Color::WHITE))); });
            p.spawn((Node { width: Val::Px(200.0), height: Val::Px(50.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() }, BackgroundColor(Color::srgb(0.5,0.1,0.1)), QuitBtn, Interaction::None)).with_children(|p| { p.spawn((Text::new("QUIT"), TextFont{font_size: bevy::text::FontSize::Px(24.0),..default()}, TextColor(Color::WHITE))); });
        });
}

fn spawn_settings(mut commands: Commands, s: Res<GameSettings>) {
    commands.spawn((Node { width: Val::Percent(100.0), height: Val::Percent(100.0), flex_direction: FlexDirection::Column, align_items: AlignItems::Center, justify_content: JustifyContent::Center, row_gap: Val::Px(12.0), ..default() }, BackgroundColor(Color::srgb(0.08,0.08,0.12)), SettingsRoot))
        .with_children(|p| {
            p.spawn((Text::new("SETTINGS"), TextFont { font_size: bevy::text::FontSize::Px(36.0), ..default() }, TextColor(Color::WHITE)));
            p.spawn((Text::new(format!("Master Volume: {:.0}%", s.master_volume*100.0)), TextFont{font_size: bevy::text::FontSize::Px(20.0),..default()}, TextColor(Color::WHITE)));
            p.spawn((Text::new(format!("Graphics Quality: {}", ["Low","Med","High","Ultra"][s.graphics_quality as usize])), TextFont{font_size: bevy::text::FontSize::Px(20.0),..default()}, TextColor(Color::WHITE)));
            p.spawn((Text::new(format!("Controller Sensitivity: {:.2}", s.controller_sensitivity)), TextFont{font_size: bevy::text::FontSize::Px(20.0),..default()}, TextColor(Color::WHITE)));
            p.spawn((Text::new(format!("Invert Pitch: {}", s.invert_pitch)), TextFont{font_size: bevy::text::FontSize::Px(20.0),..default()}, TextColor(Color::WHITE)));
            p.spawn((Text::new(format!("FOV: {:.0}", s.fov)), TextFont{font_size: bevy::text::FontSize::Px(20.0),..default()}, TextColor(Color::WHITE)));
            p.spawn((Node { width: Val::Px(200.0), height: Val::Px(50.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() }, BackgroundColor(Color::srgb(0.3,0.3,0.3)), BackBtn, Interaction::None)).with_children(|p| { p.spawn((Text::new("BACK"), TextFont{font_size: bevy::text::FontSize::Px(24.0),..default()}, TextColor(Color::WHITE))); });
        });
}

fn spawn_hud(mut commands: Commands) {
    commands.spawn((Node { position_type: PositionType::Absolute, top: Val::Px(10.0), left: Val::Px(10.0), ..default() }, HudRoot))
        .with_children(|p| { p.spawn((Text::new("HUD"), TextFont{font_size: bevy::text::FontSize::Px(16.0),..default()}, TextColor(Color::WHITE), HudText)); });
}
#[derive(Component)] struct HudText;

fn update_hud(input: Res<DroneInput>, drone_q: Query<&Drone>, mut text_q: Query<&mut Text, With<HudText>>) {
    if let Ok(mut t) = text_q.single_mut() {
        let armed = drone_q.single().map(|d| d.armed).unwrap_or(false);
        t.0 = format!("Armed: {} | T:{:.2} P:{:.2} R:{:.2} Y:{:.2}", if armed{"YES"}else{"NO"}, input.throttle, input.pitch, input.roll, input.yaw);
    }
}

fn menu_buttons(mut next: ResMut<NextState<AppState>>, q: Query<(&Interaction, &PlayBtn), Changed<Interaction>>, q2: Query<(&Interaction,&SettingsBtn), Changed<Interaction>>, q3: Query<(&Interaction,&QuitBtn), Changed<Interaction>>) {
    for (i,_) in q.iter() { if *i == Interaction::Pressed { next.set(AppState::InGame); } }
    for (i,_) in q2.iter() { if *i == Interaction::Pressed { next.set(AppState::Settings); } }
    for (i,_) in q3.iter() { if *i == Interaction::Pressed { std::process::exit(0); } }
}

fn settings_buttons(mut next: ResMut<NextState<AppState>>, q: Query<(&Interaction,&BackBtn), Changed<Interaction>>) {
    for (i,_) in q.iter() { if *i == Interaction::Pressed { next.set(AppState::Menu); } }
}

fn pause_on_esc(keys: Res<ButtonInput<KeyCode>>, mut next: ResMut<NextState<AppState>>) {
    if keys.just_pressed(KeyCode::Escape) { next.set(AppState::Menu); }
}

fn despawn<T: Component>(mut commands: Commands, q: Query<Entity, With<T>>) {
    for e in q.iter() { commands.entity(e).despawn(); }
}
