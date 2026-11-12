// UI: launcher with sidebar navigation, pages.

use bevy::prelude::*;
use bevy::ui::widget::Button as UiButton;
use crate::AppState;
use crate::settings::GameSettings;
use crate::input::DroneInput;
use crate::drone::Drone;
use crate::controllers::{controller_database, ControllerEntry};

// Components
#[derive(Component)] struct Sidebar;
#[derive(Component)] struct PageRoot;
#[derive(Component)] struct NavBtn(AppState);
#[derive(Component)] struct QuitBtn;
#[derive(Component)] struct PlayBtn;
#[derive(Component)] struct ControllerItem(String);
#[derive(Component)] struct ControllerPreview;
#[derive(Component)] struct ConnStatus;
#[derive(Component)] struct HudText;
#[derive(Component)] struct MapRow(String, String); // setting name, action

pub struct UiPlugin;
impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Home), spawn_home)
            .add_systems(OnExit(AppState::Home), despawn::<PageRoot>)
            .add_systems(OnEnter(AppState::Play), spawn_hud)
            .add_systems(OnExit(AppState::Play), despawn::<HudRoot>)
            .add_systems(OnEnter(AppState::Controller), spawn_controller_page)
            .add_systems(OnExit(AppState::Controller), despawn::<PageRoot>)
            .add_systems(OnEnter(AppState::Settings), spawn_settings_page)
            .add_systems(OnExit(AppState::Settings), despawn::<PageRoot>)
            .add_systems(OnEnter(AppState::Addons), spawn_simple_page("Addons"))
            .add_systems(OnExit(AppState::Addons), despawn::<PageRoot>)
            .add_systems(OnEnter(AppState::Statistics), spawn_simple_page("Statistics"))
            .add_systems(OnExit(AppState::Statistics), despawn::<PageRoot>)
            .add_systems(OnEnter(AppState::About), spawn_simple_page("About"))
            .add_systems(OnExit(AppState::About), despawn::<PageRoot>)
            .add_systems(Update, (nav_buttons, controller_select, quit_button, update_hud.run_if(in_state(AppState::Play)), pause_on_esc.run_if(in_state(AppState::Play))));
    }
}
#[derive(Component)] struct HudRoot;

// ---- helpers ----
fn despawn<T: Component>(mut commands: Commands, q: Query<Entity, With<T>>) {
    for e in q.iter() { commands.entity(e).despawn(); }
}

fn text(s: &str, size: f32, color: Color) -> (Text, TextFont, TextColor) {
    (Text::new(s.to_string()), TextFont { font_size: bevy::text::FontSize::Px(size), ..default() }, TextColor(color))
}

// ---- Home page with sidebar ----
fn spawn_home(mut commands: Commands, settings: Res<GameSettings>) {
    commands.spawn((Node { width: Val::Percent(100.0), height: Val::Percent(100.0), flex_direction: FlexDirection::Row, ..default() }, BackgroundColor(Color::srgb(0.06,0.06,0.08)), PageRoot))
        .with_children(|root| {
            // Sidebar
            root.spawn((Node { width: Val::Px(220.0), height: Val::Percent(100.0), flex_direction: FlexDirection::Column, padding: UiRect::all(Val::Px(12.0)), row_gap: Val::Px(6.0), ..default() }, BackgroundColor(Color::srgb(0.09,0.09,0.11)), Sidebar))
                .with_children(|sb| {
                    sb.spawn(text("FPV SIM", 20.0, Color::srgb(0.6,0.4,1.0)));
                    for (label, state) in [("Home", AppState::Home),("Play", AppState::Play),("Controller", AppState::Controller),("Settings", AppState::Settings),("Addons", AppState::Addons),("Statistics", AppState::Statistics),("About", AppState::About)] {
                        sb.spawn((Node { width: Val::Percent(100.0), height: Val::Px(40.0), align_items: AlignItems::Center, padding: UiRect::left(Val::Px(12.0)), border_radius: BorderRadius::all(Val::Px(8.0)), ..default() }, BackgroundColor(Color::srgb(0.12,0.12,0.15)), NavBtn(state), Interaction::default()))
                            .with_children(|b| { b.spawn(text(label, 16.0, Color::WHITE)); });
                    }
                    sb.spawn((Node { width: Val::Percent(100.0), height: Val::Px(40.0), align_items: AlignItems::Center, padding: UiRect::left(Val::Px(12.0)), border_radius: BorderRadius::all(Val::Px(8.0)), ..default() }, BackgroundColor(Color::srgb(0.3,0.1,0.1)), QuitBtn, Interaction::default()))
                        .with_children(|b| { b.spawn(text("Quit", 16.0, Color::WHITE)); });
                });
            // Main content
            root.spawn((Node { flex_grow: 1.0, flex_direction: FlexDirection::Column, padding: UiRect::all(Val::Px(32.0)), row_gap: Val::Px(20.0), ..default() }, BackgroundColor(Color::srgb(0.06,0.06,0.08))))
                .with_children(|mc| {
                    mc.spawn(text("Welcome to FPV Drone Sim", 32.0, Color::WHITE));
                    mc.spawn(text("Start flying and configure your controller", 16.0, Color::srgb(0.7,0.7,0.7)));
                    // Start Flying button
                    mc.spawn((Node { width: Val::Px(220.0), height: Val::Px(56.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border_radius: BorderRadius::all(Val::Px(12.0)), ..default() }, BackgroundColor(Color::srgb(0.5,0.2,0.8)), PlayBtn, Interaction::default()))
                        .with_children(|b| { b.spawn(text("START FLYING", 18.0, Color::WHITE)); });
                    // Info cards
                    mc.spawn((Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(16.0), ..default() }))
                        .with_children(|row| {
                            card(row, "Recent Sessions", "None yet");
                            card(row, "Last Played Map", "Default Field");
                            card(row, format!("Version: {}", env!("CARGO_PKG_VERSION")).as_str(), settings.selected_controller.as_str());
                        });
                });
        });
}

#[derive(Component)] struct Card;
fn card(parent: &mut ChildSpawnerCommands, title: &str, body: &str) {
    parent.spawn((Node { width: Val::Px(220.0), height: Val::Px(120.0), flex_direction: FlexDirection::Column, padding: UiRect::all(Val::Px(12.0)), row_gap: Val::Px(8.0), border_radius: BorderRadius::all(Val::Px(10.0)), ..default() }, BackgroundColor(Color::srgb(0.1,0.1,0.14)), Card))
        .with_children(|c| {
            c.spawn(text(title, 14.0, Color::srgb(0.6,0.4,1.0)));
            c.spawn(text(body, 13.0, Color::srgb(0.8,0.8,0.8)));
        });
}

// ---- Controller page ----
fn spawn_controller_page(mut commands: Commands, settings: Res<GameSettings>) {
    let db = controller_database();
    commands.spawn((Node { width: Val::Percent(100.0), height: Val::Percent(100.0), flex_direction: FlexDirection::Row, ..default() }, BackgroundColor(Color::srgb(0.06,0.06,0.08)), PageRoot))
        .with_children(|root| {
            // Sidebar
            sidebar(root);
            // Left: database
            root.spawn((Node { width: Val::Percent(50.0), height: Val::Percent(100.0), flex_direction: FlexDirection::Column, padding: UiRect::all(Val::Px(16.0)), row_gap: Val::Px(8.0), ..default() }, BackgroundColor(Color::srgb(0.07,0.07,0.1))))
                .with_children(|left| {
                    left.spawn(text("Controller Database", 22.0, Color::WHITE));
                    left.spawn((Node { width: Val::Percent(100.0), height: Val::Px(420.0), overflow: Overflow::scroll_y(), ..default() }, BackgroundColor(Color::srgb(0.09,0.09,0.12))))
                        .with_children(|sc| {
                            for c in db {
                                sc.spawn((Node { width: Val::Percent(100.0), height: Val::Px(36.0), align_items: AlignItems::Center, padding: UiRect::left(Val::Px(10.0)), border_radius: BorderRadius::all(Val::Px(6.0)), ..default() }, BackgroundColor(Color::srgb(0.12,0.12,0.16)), ControllerItem(c.name.to_string()), Interaction::default()))
                                    .with_children(|r| { r.spawn(text(&format!("{}  —  {}", c.name, c.vendor), 13.0, Color::WHITE)); });
                            }
                        });
                });
            // Right: status + preview
            root.spawn((Node { width: Val::Percent(50.0), height: Val::Percent(100.0), flex_direction: FlexDirection::Column, padding: UiRect::all(Val::Px(16.0)), row_gap: Val::Px(12.0), align_items: AlignItems::Center, ..default() }, BackgroundColor(Color::srgb(0.07,0.07,0.1))))
                .with_children(|right| {
                    right.spawn(text("Connection Status", 22.0, Color::WHITE));
                    right.spawn((Node { width: Val::Px(16.0), height: Val::Px(16.0), border_radius: BorderRadius::all(Val::Px(8.0)), ..default() }, BackgroundColor(Color::srgb(0.5,0.5,0.5)), ConnStatus));
                    right.spawn(text(&format!("Selected: {}", settings.selected_controller), 14.0, Color::WHITE));
                    right.spawn(text("Preview", 18.0, Color::srgb(0.6,0.4,1.0)));
                    right.spawn((Node { width: Val::Px(300.0), height: Val::Px(200.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border_radius: BorderRadius::all(Val::Px(12.0)), ..default() }, BackgroundColor(Color::srgb(0.12,0.12,0.16)), ControllerPreview))
                        .with_children(|p| { p.spawn(text("[Controller Preview]", 14.0, Color::srgb(0.6,0.6,0.6))); });
                    right.spawn(text("Live input will appear here when connected.", 12.0, Color::srgb(0.6,0.6,0.6)));
                });
        });
}

// ---- Settings page ----
fn spawn_settings_page(mut commands: Commands, settings: Res<GameSettings>) {
    commands.spawn((Node { width: Val::Percent(100.0), height: Val::Percent(100.0), flex_direction: FlexDirection::Row, ..default() }, BackgroundColor(Color::srgb(0.06,0.06,0.08)), PageRoot))
        .with_children(|root| {
            sidebar(root);
            root.spawn((Node { flex_grow: 1.0, flex_direction: FlexDirection::Column, padding: UiRect::all(Val::Px(24.0)), row_gap: Val::Px(12.0), ..default() }, BackgroundColor(Color::srgb(0.06,0.06,0.08))))
                .with_children(|mc| {
                    mc.spawn(text("Settings", 28.0, Color::WHITE));
                    row(mc, "Master Volume", &format!("{:.0}%", settings.master_volume*100.0));
                    row(mc, "Graphics Quality", &["Low","Med","High","Ultra"][settings.graphics_quality as usize]);
                    row(mc, "Controller Sensitivity", &format!("{:.2}", settings.controller_sensitivity));
                    row(mc, "Rotation Profile", &["Slow","Normal","Fast"][settings.rotation_profile as usize]);
                    row(mc, "Deadzone", &format!("{:.2}", settings.deadzone));
                    row(mc, "Expo", &format!("{:.2}", settings.expo));
                    row(mc, "Invert Pitch", if settings.invert_pitch {"On"} else {"Off"});
                    row(mc, "Selected Controller", &settings.selected_controller);
                });
        });
}

fn row(parent: &mut ChildSpawnerCommands, name: &str, value: &str) {
    parent.spawn((Node { width: Val::Percent(100.0), height: Val::Px(44.0), flex_direction: FlexDirection::Row, justify_content: JustifyContent::SpaceBetween, align_items: AlignItems::Center, padding: UiRect::horizontal(Val::Px(12.0)), border_radius: BorderRadius::all(Val::Px(8.0)), ..default() }, BackgroundColor(Color::srgb(0.1,0.1,0.13))))
        .with_children(|r| {
            r.spawn(text(name, 16.0, Color::WHITE));
            r.spawn(text(value, 16.0, Color::srgb(0.6,0.4,1.0)));
        });
}

// ---- simple pages ----
fn spawn_simple_page(title: &'static str) -> impl FnMut(Commands) {
    move |mut commands: Commands| {
        commands.spawn((Node { width: Val::Percent(100.0), height: Val::Percent(100.0), flex_direction: FlexDirection::Row, ..default() }, BackgroundColor(Color::srgb(0.06,0.06,0.08)), PageRoot))
            .with_children(|root| {
                sidebar(root);
                root.spawn((Node { flex_grow: 1.0, justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() }, BackgroundColor(Color::srgb(0.06,0.06,0.08))))
                    .with_children(|mc| { mc.spawn(text(title, 32.0, Color::WHITE)); });
            });
    }
}

// ---- sidebar helper ----
fn sidebar(parent: &mut ChildSpawnerCommands) {
    parent.spawn((Node { width: Val::Px(200.0), height: Val::Percent(100.0), flex_direction: FlexDirection::Column, padding: UiRect::all(Val::Px(12.0)), row_gap: Val::Px(6.0), ..default() }, BackgroundColor(Color::srgb(0.09,0.09,0.11))))
        .with_children(|sb| {
            sb.spawn(text("FPV SIM", 18.0, Color::srgb(0.6,0.4,1.0)));
            for (label, state) in [("Home", AppState::Home),("Play", AppState::Play),("Controller", AppState::Controller),("Settings", AppState::Settings),("Addons", AppState::Addons),("Statistics", AppState::Statistics),("About", AppState::About)] {
                sb.spawn((Node { width: Val::Percent(100.0), height: Val::Px(40.0), align_items: AlignItems::Center, padding: UiRect::left(Val::Px(12.0)), border_radius: BorderRadius::all(Val::Px(8.0)), ..default() }, BackgroundColor(Color::srgb(0.12,0.12,0.15)), NavBtn(state), Interaction::default()))
                    .with_children(|b| { b.spawn(text(label, 15.0, Color::WHITE)); });
            }
        });
}

// ---- nav handling ----
fn quit_button(q: Query<&Interaction, (Changed<Interaction>, With<QuitBtn>)>) {
    for i in q.iter() { if *i == Interaction::Pressed { std::process::exit(0); } }
}

fn nav_buttons(mut next: ResMut<NextState<AppState>>, q: Query<(&Interaction, &NavBtn), Changed<Interaction>>) {
    for (i, nav) in q.iter() {
        if *i == Interaction::Pressed { next.set(nav.0); }
    }
}

fn controller_select(mut settings: ResMut<GameSettings>, q: Query<(&Interaction, &ControllerItem), Changed<Interaction>>) {
    for (i, item) in q.iter() {
        if *i == Interaction::Pressed {
            settings.selected_controller = item.0.clone();
        }
    }
}

// ---- HUD ----
fn spawn_hud(mut commands: Commands) {
    commands.spawn((Node { position_type: PositionType::Absolute, top: Val::Px(10.0), left: Val::Px(10.0), ..default() }, HudRoot))
        .with_children(|p| { p.spawn((Text::new("HUD"), TextFont{font_size: bevy::text::FontSize::Px(16.0),..default()}, TextColor(Color::WHITE), HudText)); });
}

fn update_hud(input: Res<DroneInput>, drone_q: Query<&Drone>, mut text_q: Query<&mut Text, With<HudText>>) {
    if let Ok(mut t) = text_q.single_mut() {
        let armed = drone_q.single().map(|d| d.armed).unwrap_or(false);
        t.0 = format!("Armed: {} | T:{:.2} P:{:.2} R:{:.2} Y:{:.2}", if armed{"YES"}else{"NO"}, input.throttle, input.pitch, input.roll, input.yaw);
    }
}

fn pause_on_esc(mut next: ResMut<NextState<AppState>>, keys: Res<ButtonInput<KeyCode>>) {
    if keys.just_pressed(KeyCode::Escape) { next.set(AppState::Home); }
}
