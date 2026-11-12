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
#[derive(Component)] struct ControllerItem(String);
#[derive(Component)] struct RotProfileRow;
#[derive(Component)] struct DeadzoneRow;
#[derive(Component)] struct ExpoRow;

pub struct UiPlugin;
impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Menu), spawn_menu)
            .add_systems(OnExit(AppState::Menu), despawn::<MenuRoot>)
            .add_systems(OnEnter(AppState::Settings), spawn_settings)
            .add_systems(OnExit(AppState::Settings), despawn::<SettingsRoot>)
            .add_systems(OnEnter(AppState::InGame), spawn_hud)
            .add_systems(OnExit(AppState::InGame), despawn::<HudRoot>)
            .add_systems(Update, (menu_buttons, menu_controller_select, settings_buttons, update_hud.run_if(in_state(AppState::InGame)), pause_on_esc.run_if(in_state(AppState::InGame))));
    }
}

fn spawn_menu(mut commands: Commands, settings: Res<GameSettings>) {
    use crate::controllers::controller_database;
    commands.spawn((Node { width: Val::Percent(100.0), height: Val::Percent(100.0), flex_direction: FlexDirection::Row, ..default() }, BackgroundColor(Color::srgb(0.02,0.03,0.08)), MenuRoot))
        .with_children(|root| {
            // Left: title + buttons
            root.spawn((Node { width: Val::Percent(60.0), height: Val::Percent(100.0), flex_direction: FlexDirection::Column, align_items: AlignItems::Center, justify_content: JustifyContent::Center, row_gap: Val::Px(24.0), ..default() }, BackgroundColor(Color::srgb(0.02,0.04,0.12))))
                .with_children(|p| {
                    p.spawn((Text::new("FPV DRONE SIM"), TextFont { font_size: bevy::text::FontSize::Px(64.0), ..default() }, TextColor(Color::srgb(0.0,0.9,1.0))));
                    // Play button glowing
                    p.spawn((Node { width: Val::Px(260.0), height: Val::Px(70.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border_radius: BorderRadius::all(Val::Px(16.0)), ..default() }, BackgroundColor(Color::srgb(0.0,0.7,0.9)), PlayBtn, Interaction::None)).with_children(|p| { p.spawn((Text::new("PLAY"), TextFont{font_size: bevy::text::FontSize::Px(30.0),..default()}, TextColor(Color::WHITE))); });
                    p.spawn((Node { width: Val::Px(200.0), height: Val::Px(50.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border_radius: BorderRadius::all(Val::Px(12.0)), ..default() }, BackgroundColor(Color::srgb(0.25,0.25,0.35)), SettingsBtn, Interaction::None)).with_children(|p| { p.spawn((Text::new("SETTINGS"), TextFont{font_size: bevy::text::FontSize::Px(22.0),..default()}, TextColor(Color::WHITE))); });
                    p.spawn((Node { width: Val::Px(200.0), height: Val::Px(50.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border_radius: BorderRadius::all(Val::Px(12.0)), ..default() }, BackgroundColor(Color::srgb(0.45,0.1,0.1)), QuitBtn, Interaction::None)).with_children(|p| { p.spawn((Text::new("QUIT"), TextFont{font_size: bevy::text::FontSize::Px(22.0),..default()}, TextColor(Color::WHITE))); });
                });
            // Right side panel: quick settings + controller browser
            root.spawn((Node { width: Val::Percent(40.0), height: Val::Percent(100.0), flex_direction: FlexDirection::Column, padding: UiRect::all(Val::Px(16.0)), row_gap: Val::Px(12.0), ..default() }, BackgroundColor(Color::srgb(0.05,0.06,0.12))))
                .with_children(|p| {
                    p.spawn((Text::new("Quick Settings"), TextFont { font_size: bevy::text::FontSize::Px(24.0), ..default() }, TextColor(Color::srgb(0.0,0.9,1.0))));
                    p.spawn((Text::new(format!("Selected Controller: {}", settings.selected_controller)), TextFont{font_size: bevy::text::FontSize::Px(16.0),..default()}, TextColor(Color::WHITE)));
                    p.spawn((Text::new("Controller Browser"), TextFont { font_size: bevy::text::FontSize::Px(20.0), ..default() }, TextColor(Color::WHITE)));
                    // scroll container
                    p.spawn((Node { width: Val::Percent(100.0), height: Val::Px(400.0), overflow: Overflow::scroll_y(), ..default() }, BackgroundColor(Color::srgb(0.08,0.09,0.15))))
                        .with_children(|sc| {
                            for c in controller_database() {
                                sc.spawn((Node { width: Val::Percent(100.0), height: Val::Px(32.0), align_items: AlignItems::Center, padding: UiRect::left(Val::Px(8.0)), ..default() }, BackgroundColor(Color::srgb(0.1,0.12,0.2)), ControllerItem(c.name.to_string()), Interaction::None))
                                    .with_children(|r| {
                                        r.spawn((Text::new(format!("{} ({})", c.name, c.vendor)), TextFont{font_size: bevy::text::FontSize::Px(14.0),..default()}, TextColor(Color::WHITE)));
                                    });
                            }
                        });
                });
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
            // clickable rows
            p.spawn((Node { width: Val::Px(320.0), height: Val::Px(30.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() }, BackgroundColor(Color::srgb(0.2,0.2,0.3)), RotProfileRow, Interaction::None)).with_children(|p| { p.spawn((Text::new(format!("Rotation Profile: {}", ["Slow","Normal","Fast"][s.rotation_profile as usize])), TextFont{font_size: bevy::text::FontSize::Px(18.0),..default()}, TextColor(Color::WHITE))); });
            p.spawn((Node { width: Val::Px(320.0), height: Val::Px(30.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() }, BackgroundColor(Color::srgb(0.2,0.2,0.3)), DeadzoneRow, Interaction::None)).with_children(|p| { p.spawn((Text::new(format!("Deadzone: {:.2}", s.deadzone)), TextFont{font_size: bevy::text::FontSize::Px(18.0),..default()}, TextColor(Color::WHITE))); });
            p.spawn((Node { width: Val::Px(320.0), height: Val::Px(30.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() }, BackgroundColor(Color::srgb(0.2,0.2,0.3)), ExpoRow, Interaction::None)).with_children(|p| { p.spawn((Text::new(format!("Expo: {:.2}", s.expo)), TextFont{font_size: bevy::text::FontSize::Px(18.0),..default()}, TextColor(Color::WHITE))); });
            // Controls section
            p.spawn((Text::new("CONTROLS"), TextFont { font_size: bevy::text::FontSize::Px(24.0), ..default() }, TextColor(Color::WHITE)));
            p.spawn((Text::new("Controller: gamepad supported (South=arm, Start=reset)"), TextFont{font_size: bevy::text::FontSize::Px(16.0),..default()}, TextColor(Color::WHITE)));
            p.spawn((Text::new("Keyboard: W/S pitch, A/D roll, Q/E yaw, Space/Shift throttle, Enter arm, R reset"), TextFont{font_size: bevy::text::FontSize::Px(16.0),..default()}, TextColor(Color::WHITE)));
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

fn settings_buttons(
    mut next: ResMut<NextState<AppState>>,
    mut settings: ResMut<GameSettings>,
    back_q: Query<(&Interaction,&BackBtn), Changed<Interaction>>,
    rot_q: Query<(&Interaction, &Children), (Changed<Interaction>, With<RotProfileRow>)>,
    dz_q: Query<(&Interaction, &Children), (Changed<Interaction>, With<DeadzoneRow>)>,
    expo_q: Query<(&Interaction, &Children), (Changed<Interaction>, With<ExpoRow>)>,
    mut text_q: Query<&mut Text>,
) {
    for (i,_) in back_q.iter() { if *i == Interaction::Pressed { next.set(AppState::Menu); } }
    for (i, children) in rot_q.iter() {
        if *i == Interaction::Pressed {
            settings.rotation_profile = (settings.rotation_profile + 1) % 3;
            for c in children.iter() { if let Ok(mut t) = text_q.get_mut(c) { t.0 = format!("Rotation Profile: {}", ["Slow","Normal","Fast"][settings.rotation_profile as usize]); } }
        }
    }
    for (i, children) in dz_q.iter() {
        if *i == Interaction::Pressed {
            settings.deadzone = (settings.deadzone + 0.05).min(0.3);
            if settings.deadzone > 0.29 { settings.deadzone = 0.0; }
            for c in children.iter() { if let Ok(mut t) = text_q.get_mut(c) { t.0 = format!("Deadzone: {:.2}", settings.deadzone); } }
        }
    }
    for (i, children) in expo_q.iter() {
        if *i == Interaction::Pressed {
            settings.expo = (settings.expo + 0.1).min(1.0);
            if settings.expo > 0.99 { settings.expo = 0.0; }
            for c in children.iter() { if let Ok(mut t) = text_q.get_mut(c) { t.0 = format!("Expo: {:.2}", settings.expo); } }
        }
    }
}

fn menu_controller_select(mut settings: ResMut<GameSettings>, q: Query<(&Interaction, &ControllerItem), Changed<Interaction>>) {
    for (i, item) in q.iter() {
        if *i == Interaction::Pressed {
            settings.selected_controller = item.0.clone();
        }
    }
}

fn pause_on_esc(keys: Res<ButtonInput<KeyCode>>, mut next: ResMut<NextState<AppState>>) {
    if keys.just_pressed(KeyCode::Escape) { next.set(AppState::Menu); }
}

fn despawn<T: Component>(mut commands: Commands, q: Query<Entity, With<T>>) {
    for e in q.iter() { commands.entity(e).despawn(); }
}
