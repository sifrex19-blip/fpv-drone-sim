// UI: launcher with sidebar navigation, pages.

use bevy::prelude::*;
use bevy::image::Image;
use bevy::input::keyboard::KeyboardInput;
use bevy::input::ButtonState;
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::picking::hover::HoverMap;
use bevy::ui::{ComputedNode, Display, OverflowAxis, ScrollPosition};
use bevy::ui::widget::Button as UiButton;
use crate::AppState;
use crate::settings::GameSettings;
use crate::input::DroneInput;
use crate::drone::Drone;
use crate::controllers::{controller_database, ControllerEntry};
use crate::live_input::LiveInput;
use crate::calibration::{CalibrationState, CalibrateBtn, CalibrationPhase};
use crate::mapping::{InputMapping, AxisSource};

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
#[derive(Component)] struct LiveNameText;
#[derive(Component)] struct LiveAxisText(usize);
#[derive(Component)] struct LiveAxisFill(usize);
#[derive(Component)] struct LiveButtonsText;
#[derive(Component)] struct CalibStatusText;
#[derive(Component)] struct MappingSummaryText;
#[derive(Component)] struct MapCycleAxis(usize);
#[derive(Component)] struct MapToggleReverse(usize);
#[derive(Component)] struct SensDown;
#[derive(Component)] struct SensUp;
#[derive(Component)] struct SearchBoxText;
#[derive(Component)] struct ControllerListContainer;
#[derive(Component, Debug)] pub struct ControllerImage;
#[derive(Component)] struct ControllerPreviewLabel;
#[derive(Component)] struct SelectedControllerText;
#[derive(Component)] struct ConnStatusText;
#[derive(Resource, Default)] pub struct ControllerSearch { pub query: String }

pub struct UiPlugin;
impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ControllerSearch>()
            .add_systems(OnEnter(AppState::Home), spawn_home)
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
            .add_systems(Update, send_scroll_events)
            .add_observer(on_scroll_handler)
            .add_systems(Update, (nav_buttons, controller_select, quit_button, update_ui_text, pause_on_esc.run_if(in_state(AppState::Play)), mapping_controls.run_if(in_state(AppState::Controller)), controller_search_input.run_if(in_state(AppState::Controller)), filter_controller_list.run_if(in_state(AppState::Controller)), update_controller_preview.run_if(in_state(AppState::Controller)), update_controller_status.run_if(in_state(AppState::Controller))));
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
fn spawn_controller_page(mut commands: Commands, settings: Res<GameSettings>, live: Res<LiveInput>, mapping: Res<InputMapping>, cal: Res<CalibrationState>, asset_server: Res<AssetServer>, search: Res<ControllerSearch>) {
    let db = controller_database();
    commands.spawn((Node { width: Val::Percent(100.0), height: Val::Percent(100.0), flex_direction: FlexDirection::Row, ..default() }, BackgroundColor(Color::srgb(0.06,0.06,0.08)), PageRoot))
        .with_children(|root| {
            sidebar(root);
            // Left panel: search + grouped controller list
            root.spawn((Node { width: Val::Percent(34.0), height: Val::Percent(100.0), flex_direction: FlexDirection::Column, padding: UiRect::all(Val::Px(14.0)), row_gap: Val::Px(8.0), ..default() }, BackgroundColor(Color::srgb(0.07,0.07,0.1))))
                .with_children(|left| {
                    left.spawn(text("Controllers", 20.0, Color::WHITE));
                    left.spawn((Node { width: Val::Percent(100.0), height: Val::Px(34.0), align_items: AlignItems::Center, padding: UiRect::horizontal(Val::Px(10.0)), border_radius: BorderRadius::all(Val::Px(8.0)), ..default() }, BackgroundColor(Color::srgb(0.12,0.12,0.16))))
                        .with_children(|sb| {
                            sb.spawn((Text::new(if search.query.is_empty() { "Search controllers...".to_string() } else { search.query.clone() }), TextFont { font_size: bevy::text::FontSize::Px(13.0), ..default() }, TextColor(Color::srgb(0.7,0.7,0.7)), SearchBoxText));
                        });
                    left.spawn((Node { width: Val::Percent(100.0), flex_grow: 1.0, overflow: Overflow::scroll_y(), ..default() }, BackgroundColor(Color::srgb(0.09,0.09,0.12)), ScrollPosition::default(), ControllerListContainer))
                        .with_children(|sc| {
                            let qstr = search.query.to_lowercase();
                            let filtered: Vec<ControllerEntry> = db.iter().filter(|c| qstr.is_empty() || c.name.to_lowercase().contains(&qstr) || c.vendor.to_lowercase().contains(&qstr)).cloned().collect();
                            let mut vendors: Vec<&str> = Vec::new();
                            for c in &filtered { if !vendors.contains(&c.vendor) { vendors.push(c.vendor); } }
                            for v in vendors {
                                sc.spawn(text(v, 12.0, Color::srgb(0.6,0.4,1.0)));
                                for c in filtered.iter().filter(|c| c.vendor == v) {
                                    let selected = settings.selected_controller == c.name;
                                    sc.spawn((Node { width: Val::Percent(100.0), height: Val::Px(30.0), align_items: AlignItems::Center, padding: UiRect::left(Val::Px(10.0)), border_radius: BorderRadius::all(Val::Px(6.0)), ..default() }, BackgroundColor(if selected { Color::srgb(0.5,0.2,0.8) } else { Color::srgb(0.12,0.12,0.16) }), ControllerItem(c.name.to_string()), Interaction::default()))
                                        .with_children(|r| { r.spawn(text(&format!("{}", c.name), 12.0, Color::WHITE)); });
                                }
                            }
                        });
                });
            // Right panel
            root.spawn((Node { width: Val::Percent(66.0), height: Val::Percent(100.0), flex_direction: FlexDirection::Column, padding: UiRect::all(Val::Px(14.0)), row_gap: Val::Px(10.0), overflow: Overflow::scroll_y(), ..default() }, BackgroundColor(Color::srgb(0.07,0.07,0.1))))
                .with_children(|right| {
                    // Preview card
                    right.spawn((Node { width: Val::Percent(100.0), flex_direction: FlexDirection::Row, column_gap: Val::Px(16.0), padding: UiRect::all(Val::Px(12.0)), border_radius: BorderRadius::all(Val::Px(12.0)), ..default() }, BackgroundColor(Color::srgb(0.1,0.1,0.14))))
                        .with_children(|card| {
                            let color = controller_color(&settings.selected_controller);
                            let handle = controller_image_handle(&asset_server, &settings.selected_controller);
                            card.spawn((Node { width: Val::Px(220.0), height: Val::Px(140.0), border_radius: BorderRadius::all(Val::Px(12.0)), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() }, BackgroundColor(color), ImageNode { image: handle, ..default() }, ControllerImage))
                                .with_children(|p| { p.spawn((Text::new(settings.selected_controller.clone()), TextFont { font_size: bevy::text::FontSize::Px(13.0), ..default() }, TextColor(Color::WHITE), ControllerPreviewLabel)); });
                            card.spawn((Node { flex_grow: 1.0, flex_direction: FlexDirection::Column, row_gap: Val::Px(6.0), ..default() }))
                                .with_children(|info| {
                                    info.spawn(text("Selected Controller", 16.0, Color::srgb(0.6,0.4,1.0)));
                                    info.spawn((Text::new(settings.selected_controller.clone()), TextFont { font_size: bevy::text::FontSize::Px(18.0), ..default() }, TextColor(Color::WHITE), SelectedControllerText));
                                    info.spawn(text("Connection", 16.0, Color::srgb(0.6,0.4,1.0)));
                                    info.spawn((Node { flex_direction: FlexDirection::Row, align_items: AlignItems::Center, column_gap: Val::Px(8.0), ..default() }))
                                        .with_children(|r| {
                                            r.spawn((Node { width: Val::Px(14.0), height: Val::Px(14.0), border_radius: BorderRadius::all(Val::Px(7.0)), ..default() }, BackgroundColor(if live.connected { Color::srgb(0.2,0.9,0.3) } else { Color::srgb(0.5,0.5,0.5) }), ConnStatus));
                                            r.spawn((Text::new(if live.connected { "Connected".to_string() } else { "Disconnected".to_string() }), TextFont { font_size: bevy::text::FontSize::Px(14.0), ..default() }, TextColor(Color::WHITE), ConnStatusText));
                                        });
                                    info.spawn((Text::new(format!("Live: {}", if live.connected { live.name.clone() } else { "none".to_string() })), TextFont { font_size: bevy::text::FontSize::Px(13.0), ..default() }, TextColor(Color::srgb(0.8,0.8,0.8)), LiveNameText));
                                });
                        });

                    // Live Input card
                    right.spawn((Node { width: Val::Percent(100.0), flex_direction: FlexDirection::Column, padding: UiRect::all(Val::Px(12.0)), row_gap: Val::Px(8.0), border_radius: BorderRadius::all(Val::Px(10.0)), ..default() }, BackgroundColor(Color::srgb(0.1,0.1,0.14))))
                        .with_children(|card| {
                            card.spawn(text("Live Input", 18.0, Color::srgb(0.6,0.4,1.0)));
                            for i in 0..4 {
                                let label = ["Roll","Pitch","Yaw","Throttle"][i];
                                let val = [live.roll,live.pitch,live.yaw,live.throttle][i];
                                card.spawn((Node { width: Val::Percent(100.0), height: Val::Px(24.0), flex_direction: FlexDirection::Row, align_items: AlignItems::Center, column_gap: Val::Px(8.0), ..default() }, BackgroundColor(Color::srgb(0.08,0.08,0.11))))
                                    .with_children(|r| {
                                        r.spawn(text(label, 13.0, Color::WHITE));
                                        r.spawn((Node { width: Val::Px(180.0), height: Val::Px(12.0), border_radius: BorderRadius::all(Val::Px(6.0)), ..default() }, BackgroundColor(Color::srgb(0.05,0.05,0.08))))
                                            .with_children(|bar| {
                                                bar.spawn((Node { width: Val::Percent(((val+1.0)/2.0*100.0).clamp(0.0,100.0)), height: Val::Percent(100.0), border_radius: BorderRadius::all(Val::Px(6.0)), ..default() }, BackgroundColor(Color::srgb(0.5,0.2,0.8)), LiveAxisFill(i)));
                                            });
                                        r.spawn((Text::new(format!("{:.2}", val)), TextFont { font_size: bevy::text::FontSize::Px(13.0), ..default() }, TextColor(Color::srgb(0.6,0.4,1.0)), LiveAxisText(i)));
                                    });
                            }
                            card.spawn((Text::new(format!("Buttons: {}", if live.buttons.is_empty() { "None".to_string() } else { live.buttons.join(", ") })), TextFont { font_size: bevy::text::FontSize::Px(13.0), ..default() }, TextColor(Color::srgb(0.8,0.8,0.8)), LiveButtonsText));
                        });

                    // Calibration card
                    right.spawn((Node { width: Val::Percent(100.0), flex_direction: FlexDirection::Column, padding: UiRect::all(Val::Px(12.0)), row_gap: Val::Px(8.0), border_radius: BorderRadius::all(Val::Px(10.0)), ..default() }, BackgroundColor(Color::srgb(0.1,0.1,0.14))))
                        .with_children(|card| {
                            card.spawn(text("Calibration", 18.0, Color::srgb(0.6,0.4,1.0)));
                            let status = match cal.phase {
                                CalibrationPhase::Idle => if cal.calibrated { "Calibrated".to_string() } else { "Idle".to_string() },
                                CalibrationPhase::Center => format!("Calibrating center... ({}/90)", cal.samples),
                                CalibrationPhase::MinMax => format!("Move sticks to extremes... ({}/240)", cal.samples),
                            };
                            card.spawn((Text::new(status), TextFont { font_size: bevy::text::FontSize::Px(13.0), ..default() }, TextColor(Color::WHITE), CalibStatusText));
                            card.spawn((Node { width: Val::Px(160.0), height: Val::Px(36.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border_radius: BorderRadius::all(Val::Px(8.0)), ..default() }, BackgroundColor(Color::srgb(0.5,0.2,0.8)), CalibrateBtn, Interaction::default()))
                                .with_children(|b| { b.spawn(text("Calibrate", 14.0, Color::WHITE)); });
                        });

                    // Mapping card
                    right.spawn((Node { width: Val::Percent(100.0), flex_direction: FlexDirection::Column, padding: UiRect::all(Val::Px(12.0)), row_gap: Val::Px(8.0), border_radius: BorderRadius::all(Val::Px(10.0)), ..default() }, BackgroundColor(Color::srgb(0.1,0.1,0.14))))
                        .with_children(|card| {
                            card.spawn(text("Input Mapping", 18.0, Color::srgb(0.6,0.4,1.0)));
                            card.spawn((Text::new(mapping_summary(mapping.clone())), TextFont { font_size: bevy::text::FontSize::Px(12.0), ..default() }, TextColor(Color::WHITE), MappingSummaryText));
                            for i in 0..4 {
                                let label = ["Roll","Pitch","Yaw","Throttle"][i];
                                card.spawn((Node { width: Val::Percent(100.0), height: Val::Px(30.0), flex_direction: FlexDirection::Row, align_items: AlignItems::Center, column_gap: Val::Px(6.0), ..default() }, BackgroundColor(Color::srgb(0.08,0.08,0.11))))
                                    .with_children(|r| {
                                        r.spawn(text(label, 12.0, Color::WHITE));
                                        r.spawn((Node { width: Val::Px(90.0), height: Val::Px(24.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border_radius: BorderRadius::all(Val::Px(6.0)), ..default() }, BackgroundColor(Color::srgb(0.2,0.2,0.25)), MapCycleAxis(i), Interaction::default()))
                                            .with_children(|b| { b.spawn(text("Cycle Axis", 11.0, Color::WHITE)); });
                                        r.spawn((Node { width: Val::Px(90.0), height: Val::Px(24.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border_radius: BorderRadius::all(Val::Px(6.0)), ..default() }, BackgroundColor(Color::srgb(0.2,0.2,0.25)), MapToggleReverse(i), Interaction::default()))
                                            .with_children(|b| { b.spawn(text("Toggle Rev", 11.0, Color::WHITE)); });
                                    });
                            }
                            card.spawn((Node { width: Val::Percent(100.0), height: Val::Px(30.0), flex_direction: FlexDirection::Row, align_items: AlignItems::Center, column_gap: Val::Px(6.0), ..default() }, BackgroundColor(Color::srgb(0.08,0.08,0.11))))
                                .with_children(|r| {
                                    r.spawn(text("Sensitivity", 12.0, Color::WHITE));
                                    r.spawn((Node { width: Val::Px(60.0), height: Val::Px(24.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border_radius: BorderRadius::all(Val::Px(6.0)), ..default() }, BackgroundColor(Color::srgb(0.2,0.2,0.25)), SensDown, Interaction::default()))
                                        .with_children(|b| { b.spawn(text("-", 14.0, Color::WHITE)); });
                                    r.spawn((Node { width: Val::Px(60.0), height: Val::Px(24.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border_radius: BorderRadius::all(Val::Px(6.0)), ..default() }, BackgroundColor(Color::srgb(0.2,0.2,0.25)), SensUp, Interaction::default()))
                                        .with_children(|b| { b.spawn(text("+", 14.0, Color::WHITE)); });
                                });
                        });
                });
        });
}

fn controller_color(name: &str) -> Color {
    let mut h: u32 = 5381;
    for b in name.bytes() { h = h.wrapping_mul(33).wrapping_add(b as u32); }
    let r = ((h & 0xFF) as f32 / 255.0).clamp(0.2, 0.9);
    let g = (((h >> 8) & 0xFF) as f32 / 255.0).clamp(0.2, 0.9);
    let b = (((h >> 16) & 0xFF) as f32 / 255.0).clamp(0.2, 0.9);
    Color::srgb(r, g, b)
}

fn controller_image_handle(asset_server: &AssetServer, name: &str) -> Handle<Image> {
    let path = format!("controllers/{}.png", name);
    if std::path::Path::new("assets").join(&path).exists() {
        asset_server.load(path)
    } else {
        default()
    }
}

fn controller_search_input(
    mut reader: MessageReader<KeyboardInput>,
    mut search: ResMut<ControllerSearch>,
    mut q: Query<&mut Text, With<SearchBoxText>>,
) {
    for ev in reader.read() {
        if ev.state != ButtonState::Pressed { continue; }
        if ev.key_code == KeyCode::Backspace {
            search.query.pop();
        } else if let Some(t) = &ev.text {
            for ch in t.chars() {
                if !ch.is_control() { search.query.push(ch); }
            }
        }
    }
    for mut txt in q.iter_mut() {
        txt.0 = if search.query.is_empty() { "Search controllers...".to_string() } else { search.query.clone() };
    }
}

fn filter_controller_list(search: Res<ControllerSearch>, mut q: Query<(&ControllerItem, &mut Node)>) {
    if !search.is_changed() { return; }
    let qstr = search.query.to_lowercase();
    for (item, mut node) in q.iter_mut() {
        let m = qstr.is_empty() || item.0.to_lowercase().contains(&qstr);
        node.display = if m { Display::Flex } else { Display::None };
    }
}

fn update_controller_preview(
    settings: Res<GameSettings>,
    asset_server: Res<AssetServer>,
    mut imgs: Query<&mut ImageNode, With<ControllerImage>>,
    mut labels: Query<&mut Text, With<ControllerPreviewLabel>>,
    mut selected: Query<&mut Text, With<SelectedControllerText>>,
) {
    if !settings.is_changed() { return; }
    let handle = controller_image_handle(&asset_server, &settings.selected_controller);
    for mut img in imgs.iter_mut() { img.image = handle.clone(); }
    for mut t in labels.iter_mut() { t.0 = settings.selected_controller.clone(); }
    for mut t in selected.iter_mut() { t.0 = settings.selected_controller.clone(); }
}

const LINE_HEIGHT: f32 = 21.0;

fn send_scroll_events(
    mut mouse_wheel_reader: MessageReader<MouseWheel>,
    hover_map: Res<HoverMap>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
) {
    for mouse_wheel in mouse_wheel_reader.read() {
        let mut delta = -Vec2::new(mouse_wheel.x, mouse_wheel.y);
        if mouse_wheel.unit == MouseScrollUnit::Line {
            delta *= LINE_HEIGHT;
        }
        if keyboard_input.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]) {
            std::mem::swap(&mut delta.x, &mut delta.y);
        }
        for pointer_map in hover_map.values() {
            for entity in pointer_map.keys().copied() {
                commands.trigger(Scroll { entity, delta });
            }
        }
    }
}

#[derive(EntityEvent, Debug)]
#[entity_event(propagate, auto_propagate)]
struct Scroll {
    entity: Entity,
    delta: Vec2,
}

fn on_scroll_handler(
    mut scroll: On<Scroll>,
    mut query: Query<(&mut ScrollPosition, &Node, &ComputedNode)>,
) {
    let Ok((mut scroll_position, node, computed)) = query.get_mut(scroll.entity) else {
        return;
    };
    let max_offset = (computed.content_size() - computed.size()) * computed.inverse_scale_factor();
    let delta = &mut scroll.delta;
    if node.overflow.x == OverflowAxis::Scroll && delta.x != 0.0 {
        let max = if delta.x > 0.0 { scroll_position.x >= max_offset.x } else { scroll_position.x <= 0.0 };
        if !max { scroll_position.x += delta.x; delta.x = 0.0; }
    }
    if node.overflow.y == OverflowAxis::Scroll && delta.y != 0.0 {
        let max = if delta.y > 0.0 { scroll_position.y >= max_offset.y } else { scroll_position.y <= 0.0 };
        if !max { scroll_position.y += delta.y; delta.y = 0.0; }
    }
    if *delta == Vec2::ZERO { scroll.propagate(false); }
}

fn update_controller_status(
    settings: Res<GameSettings>,
    live: Res<LiveInput>,
    mut params: ParamSet<(
        Query<&mut BackgroundColor, With<ConnStatus>>,
        Query<&mut Text, With<ConnStatusText>>,
        Query<&mut Text, With<SelectedControllerText>>,
    )>,
) {
    for mut bg in params.p0().iter_mut() {
        bg.0 = if live.connected { Color::srgb(0.2,0.9,0.3) } else { Color::srgb(0.5,0.5,0.5) };
    }
    for mut t in params.p1().iter_mut() {
        t.0 = if live.connected { "Connected".to_string() } else { "Disconnected".to_string() };
    }
    for mut t in params.p2().iter_mut() {
        t.0 = settings.selected_controller.clone();
    }
}

fn mapping_summary(m: InputMapping) -> String {
    format!("Roll:{}{} Pitch:{}{} Yaw:{}{} Thr:{}{} Sens:{:.2}",
        m.roll.label(), if m.roll_reversed{" (rev)"}else{""},
        m.pitch.label(), if m.pitch_reversed{" (rev)"}else{""},
        m.yaw.label(), if m.yaw_reversed{" (rev)"}else{""},
        m.throttle.label(), if m.throttle_reversed{" (rev)"}else{""},
        m.sensitivity)
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

fn pause_on_esc(mut next: ResMut<NextState<AppState>>, keys: Res<ButtonInput<KeyCode>>) {
    if keys.just_pressed(KeyCode::Escape) { next.set(AppState::Home); }
}

fn update_ui_text(
    input: Res<DroneInput>,
    drone_q: Query<&Drone>,
    live: Res<LiveInput>,
    cal: Res<CalibrationState>,
    mapping: Res<InputMapping>,
    mut params: ParamSet<(
        Query<&mut Text, With<HudText>>,
        Query<&mut Text, With<LiveNameText>>,
        Query<(&mut Text, &LiveAxisText)>,
        Query<&mut Text, With<LiveButtonsText>>,
        Query<&mut Text, With<CalibStatusText>>,
        Query<&mut Text, With<MappingSummaryText>>,
        Query<(&mut Node, &LiveAxisFill)>,
    )>,
) {
    if let Ok(mut t) = params.p0().single_mut() {
        let armed = drone_q.single().map(|d| d.armed).unwrap_or(false);
        t.0 = format!("Armed: {} | T:{:.2} P:{:.2} R:{:.2} Y:{:.2}", if armed{"YES"}else{"NO"}, input.throttle, input.pitch, input.roll, input.yaw);
    }
    let vals = [live.roll, live.pitch, live.yaw, live.throttle];
    for mut t in params.p1().iter_mut() {
        t.0 = format!("Live: {}", if live.connected { live.name.clone() } else { "none".to_string() });
    }
    for (mut t, marker) in params.p2().iter_mut() {
        t.0 = format!("{:.2}", vals[marker.0]);
    }
    for (mut node, marker) in params.p6().iter_mut() {
        let v = vals[marker.0];
        node.width = Val::Percent(((v + 1.0) / 2.0 * 100.0).clamp(0.0, 100.0));
    }
    for mut t in params.p3().iter_mut() {
        t.0 = format!("Buttons: {}", if live.buttons.is_empty() { "None".to_string() } else { live.buttons.join(", ") });
    }
    for mut t in params.p4().iter_mut() {
        t.0 = match cal.phase {
            CalibrationPhase::Idle => if cal.calibrated { "Calibrated".to_string() } else { "Idle".to_string() },
            CalibrationPhase::Center => format!("Calibrating center... ({}/90)", cal.samples),
            CalibrationPhase::MinMax => format!("Move sticks to extremes... ({}/240)", cal.samples),
        };
    }
    for mut t in params.p5().iter_mut() {
        t.0 = mapping_summary(mapping.clone());
    }
}



fn mapping_controls(
    mut mapping: ResMut<InputMapping>,
    q_cycle: Query<(&Interaction, &MapCycleAxis), Changed<Interaction>>,
    q_toggle: Query<(&Interaction, &MapToggleReverse), Changed<Interaction>>,
    q_sens_down: Query<&Interaction, (Changed<Interaction>, With<SensDown>)>,
    q_sens_up: Query<&Interaction, (Changed<Interaction>, With<SensUp>)>,
) {
    for (i, marker) in q_cycle.iter() {
        if *i == Interaction::Pressed {
            let axis = marker.0;
            match axis {
                0 => mapping.roll = mapping.roll.next(),
                1 => mapping.pitch = mapping.pitch.next(),
                2 => mapping.yaw = mapping.yaw.next(),
                3 => mapping.throttle = mapping.throttle.next(),
                _ => {}
            }
        }
    }
    for (i, marker) in q_toggle.iter() {
        if *i == Interaction::Pressed {
            let axis = marker.0;
            match axis {
                0 => mapping.roll_reversed = !mapping.roll_reversed,
                1 => mapping.pitch_reversed = !mapping.pitch_reversed,
                2 => mapping.yaw_reversed = !mapping.yaw_reversed,
                3 => mapping.throttle_reversed = !mapping.throttle_reversed,
                _ => {}
            }
        }
    }
    for i in q_sens_down.iter() {
        if *i == Interaction::Pressed {
            mapping.sensitivity = (mapping.sensitivity - 0.1).max(0.1);
        }
    }
    for i in q_sens_up.iter() {
        if *i == Interaction::Pressed {
            mapping.sensitivity = (mapping.sensitivity + 0.1).min(3.0);
        }
    }
}
