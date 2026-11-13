// Launcher UI: dark + purple theme, sidebar navigation, pages.
// Controller page follows the reference design: Controller Database with
// search/thumbnails/kind badges, connected banner, info card with
// Calibrate Sticks, live Input Mapping bars + axis chips, Advanced Settings.

use bevy::input::keyboard::KeyboardInput;
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::input::ButtonState;
use bevy::picking::hover::HoverMap;
use bevy::prelude::*;
use bevy::ui::{ComputedNode, Display, OverflowAxis, ScrollPosition};
use bevy::image::Image;

use crate::calibration::{CalibrateBtn, CalibrationPhase, CalibrationState};
use crate::controllers::{controller_database, ControllerEntry};
use crate::live_input::LiveInput;
use crate::mapping::{AxisSource, InputMapping};
use crate::settings::GameSettings;
use crate::stats::FlightStats;
use crate::theme::{self, icon_box, page_header, text, text_bold, HoverTint};
use bevy::text::LetterSpacing;
use crate::AppState;

// ---- components ----
#[derive(Component)] struct PageRoot;
#[derive(Component)] struct NavBtn(AppState);
#[derive(Component)] struct QuitBtn;
#[derive(Component)] struct PlayBtn;
#[derive(Component)] struct ControllerItem { name: String, vendor: String }
#[derive(Component)] struct VendorLabel(String);
#[derive(Component)] struct SearchBoxText;
#[derive(Component)] struct ControllerListContainer;
#[derive(Component, Debug)] pub struct ControllerImage;
#[derive(Component)] struct SelectedControllerText;
#[derive(Component)] struct KindText;
#[derive(Component)] struct ConnStatus;
#[derive(Component)] struct ConnStatusText;
#[derive(Component)] struct InfoConnText;
#[derive(Component)] struct LiveNameText;
#[derive(Component)] struct LiveAxisText(usize);
#[derive(Component)] struct LiveAxisFill(usize);
#[derive(Component)] struct LiveButtonsText;
#[derive(Component)] struct CalibStatusText;
#[derive(Component)] struct MapAxisChip(usize);
#[derive(Component)] struct MapCycleAxis(usize);
#[derive(Component)] struct MapToggleReverse(usize);
#[derive(Component)] struct MapSensText;
#[derive(Component)] struct SensDown;
#[derive(Component)] struct SensUp;
#[derive(Component)] struct SettingBtn(&'static str, i32);
#[derive(Component)] struct SettingValue(&'static str);
#[derive(Component)] struct ResetStatsBtn;
#[derive(Component)] struct StatValue(&'static str);
#[derive(Component)] struct HomeLiveText;
#[derive(Component)] struct HomeStatsText;

#[derive(Resource, Default)]
pub struct ControllerSearch {
    pub query: String,
}

const NAV_ITEMS: [(&str, AppState, &str); 6] = [
    ("Home", AppState::Home, "home"),
    ("Controller", AppState::Controller, "controller"),
    ("Settings", AppState::Settings, "settings"),
    ("Addons", AppState::Addons, "addons"),
    ("Statistics", AppState::Statistics, "stats"),
    ("About", AppState::About, "about"),
];

pub struct UiPlugin;
impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ControllerSearch>()
            .add_systems(OnEnter(AppState::Home), spawn_home)
            .add_systems(OnExit(AppState::Home), despawn::<PageRoot>)
            .add_systems(OnEnter(AppState::Controller), spawn_controller_page)
            .add_systems(OnExit(AppState::Controller), despawn::<PageRoot>)
            .add_systems(OnEnter(AppState::Settings), spawn_settings_page)
            .add_systems(OnExit(AppState::Settings), despawn::<PageRoot>)
            .add_systems(OnEnter(AppState::Addons), spawn_addons_page)
            .add_systems(OnExit(AppState::Addons), despawn::<PageRoot>)
            .add_systems(OnEnter(AppState::Statistics), spawn_statistics_page)
            .add_systems(OnExit(AppState::Statistics), despawn::<PageRoot>)
            .add_systems(OnEnter(AppState::About), spawn_about_page)
            .add_systems(OnExit(AppState::About), despawn::<PageRoot>)
            .add_systems(Update, send_scroll_events)
            .add_observer(on_scroll_handler)
            .add_systems(
                Update,
                (
                    theme::hover_tints,
                    nav_buttons,
                    quit_button,
                    play_button,
                    update_home_texts.run_if(in_state(AppState::Home)),
                    (
                        controller_search_input,
                        filter_controller_list,
                        controller_select,
                        mapping_controls,
                        update_controller_preview,
                        update_controller_status,
                        update_live_panel,
                    )
                        .chain()
                        .run_if(in_state(AppState::Controller)),
                    settings_buttons.run_if(in_state(AppState::Settings)),
                    stats_buttons.run_if(in_state(AppState::Statistics)),
                ),
            )
            // After Bevy registers freshly loaded fonts so the family check is valid.
            .add_systems(
                PostUpdate,
                theme::apply_ui_font
                    .after(bevy::text::load_font_assets_into_font_collection),
            );
    }
}

// ---- shared shell ----

fn shell() -> Node {
    Node {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        flex_direction: FlexDirection::Row,
        ..default()
    }
}

fn main_area() -> Node {
    Node {
        flex_grow: 1.0,
        flex_direction: FlexDirection::Column,
        padding: UiRect::all(Val::Px(28.0)),
        row_gap: Val::Px(18.0),
        ..default()
    }
}

fn despawn<T: Component>(mut commands: Commands, q: Query<Entity, With<T>>) {
    for e in q.iter() {
        commands.entity(e).despawn();
    }
}

fn sidebar(parent: &mut ChildSpawnerCommands, active: AppState, asset_server: &AssetServer) {
    parent
        .spawn((
            Node {
                width: Val::Px(236.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(16.0)),
                row_gap: Val::Px(4.0),
                border: UiRect::right(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(theme::sidebar_bg()),
            BorderColor { right: theme::hairline(), ..BorderColor::DEFAULT },
        ))
        .with_children(|sb| {
            // logo block
            sb.spawn((
                Node {
                    column_gap: Val::Px(12.0),
                    align_items: AlignItems::Center,
                    margin: UiRect::bottom(Val::Px(22.0)),
                    ..default()
                },
            ))
            .with_children(|logo| {
                logo.spawn(icon_box(asset_server, "logo", 46.0));
                logo.spawn((
                    Node { flex_direction: FlexDirection::Column, row_gap: Val::Px(1.0), ..default() },
                ))
                .with_children(|titles| {
                    titles.spawn(text_bold("FPV SIM", 19.0, Color::WHITE));
                    titles.spawn(theme::label("Simulator", 9.0, theme::accent()));
                });
            });

            // nav items
            for (label, state, icon_name) in NAV_ITEMS {
                let is_active = state == active;
                let base = if is_active { theme::accent_dim() } else { theme::none() };
                let hover = if is_active { theme::accent_dim() } else { theme::row() };
                sb.spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(42.0),
                        column_gap: Val::Px(12.0),
                        align_items: AlignItems::Center,
                        padding: UiRect::left(Val::Px(if is_active { 9.0 } else { 12.0 })),
                        border_radius: BorderRadius::all(Val::Px(10.0)),
                        border: UiRect {
                            left: Val::Px(if is_active { 3.0 } else { 0.0 }),
                            ..default()
                        },
                        ..default()
                    },
                    BackgroundColor(base),
                    BorderColor {
                        left: if is_active { theme::accent() } else { Color::NONE },
                        ..BorderColor::DEFAULT
                    },
                    NavBtn(state),
                    Interaction::default(),
                    HoverTint { base, hover },
                ))
                .with_children(|item| {
                    item.spawn(icon_box(asset_server, icon_name, 20.0));
                    if is_active {
                        item.spawn(text_bold(label, 15.0, Color::WHITE));
                    } else {
                        item.spawn(text(label, 15.0, theme::text_dim()));
                    }
                });
            }

            // spacer pushes quit/version to the bottom
            sb.spawn((Node { flex_grow: 1.0, ..default() },));

            sb.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(42.0),
                    column_gap: Val::Px(12.0),
                    align_items: AlignItems::Center,
                    padding: UiRect::left(Val::Px(12.0)),
                    border_radius: BorderRadius::all(Val::Px(10.0)),
                    ..default()
                },
                BackgroundColor(theme::none()),
                QuitBtn,
                Interaction::default(),
                HoverTint { base: theme::none(), hover: Color::srgba(0.5, 0.12, 0.15, 0.5) },
            ))
            .with_children(|q| {
                q.spawn(icon_box(asset_server, "quit", 20.0));
                q.spawn(text("Quit", 15.0, theme::red()));
            });

            sb.spawn((
                Node { margin: UiRect::top(Val::Px(10.0)), ..default() },
            ))
            .with_children(|ver| {
                ver.spawn(text(
                    &format!("v{}", env!("CARGO_PKG_VERSION")),
                    11.0,
                    theme::text_dim(),
                ));
            });
        });
}

// ---- Home ----

fn spawn_home(
    mut commands: Commands,
    settings: Res<GameSettings>,
    live: Res<LiveInput>,
    stats: Res<FlightStats>,
    asset_server: Res<AssetServer>,
) {
    commands
        .spawn((shell(), BackgroundColor(theme::bg()), PageRoot))
        .with_children(|root| {
            sidebar(root, AppState::Home, &asset_server);
            root.spawn((main_area(), BackgroundColor(theme::bg())))
                .with_children(|content| {
                    page_header(
                        content,
                        &asset_server,
                        "home",
                        "Home",
                        "Welcome back, pilot — ready to fly?",
                    );

                    // hero start button
                    content
                        .spawn((
                            Node {
                                width: Val::Px(340.0),
                                height: Val::Px(62.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                column_gap: Val::Px(12.0),
                                border_radius: BorderRadius::all(Val::Px(14.0)),
                                ..default()
                            },
                            BackgroundColor(theme::accent()),
                            theme::glow_shadow(),
                            HoverTint { base: theme::accent(), hover: theme::accent_hover() },
                            PlayBtn,
                            Interaction::default(),
                        ))
                        .with_children(|b| {
                            b.spawn((
                                theme::text_bold("START FLYING", 18.0, Color::WHITE),
                                LetterSpacing::Px(2.0),
                            ));
                            b.spawn(theme::text_bold("→", 18.0, Color::WHITE.with_alpha(0.9)));
                        });
                    content.spawn(text(
                        "Acro + Angle modes · race course · freestyle map",
                        13.0,
                        theme::text_dim(),
                    ));

                    // info cards row
                    content
                        .spawn((Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(16.0), ..default() },))
                        .with_children(|row| {
                            // flight mode card
                            row.spawn((
                                Node {
                                    width: Val::Px(250.0),
                                    height: Val::Px(110.0),
                                    flex_direction: FlexDirection::Column,
                                    padding: UiRect::all(Val::Px(16.0)),
                                    row_gap: Val::Px(6.0),
                                    border_radius: BorderRadius::all(Val::Px(14.0)),
                                    border: UiRect::all(Val::Px(1.0)),
                                    ..default()
                                },
                                BackgroundColor(theme::card()),
                                theme::card_shadow(),
                                BorderColor::all(theme::hairline()),
                            ))
                            .with_children(|card| {
                                card.spawn(theme::label("Flight mode", 11.0, theme::text_dim()));
                                card.spawn(theme::text_bold(settings.flight_mode_label(), 24.0, theme::accent()));
                                card.spawn(text("Toggle with M while flying", 12.0, theme::text_dim()));
                            });

                            // controller card
                            row.spawn((
                                Node {
                                    width: Val::Px(280.0),
                                    height: Val::Px(110.0),
                                    flex_direction: FlexDirection::Column,
                                    padding: UiRect::all(Val::Px(16.0)),
                                    row_gap: Val::Px(6.0),
                                    border_radius: BorderRadius::all(Val::Px(14.0)),
                                    border: UiRect::all(Val::Px(1.0)),
                                    ..default()
                                },
                                BackgroundColor(theme::card()),
                                theme::card_shadow(),
                                BorderColor::all(theme::hairline()),
                            ))
                            .with_children(|card| {
                                card.spawn(theme::label("Controller", 11.0, theme::text_dim()));
                                card.spawn((
                                    theme::text_bold(
                                        if live.connected {
                                            live.name.as_str()
                                        } else {
                                            settings.selected_controller.as_str()
                                        },
                                        16.0,
                                        if live.connected { theme::green() } else { Color::WHITE },
                                    ),
                                    HomeLiveText,
                                ));
                                card.spawn(text("Configure on the Controller page", 12.0, theme::text_dim()));
                            });

                            // statistics card
                            row.spawn((
                                Node {
                                    width: Val::Px(280.0),
                                    height: Val::Px(110.0),
                                    flex_direction: FlexDirection::Column,
                                    padding: UiRect::all(Val::Px(16.0)),
                                    row_gap: Val::Px(6.0),
                                    border_radius: BorderRadius::all(Val::Px(14.0)),
                                    border: UiRect::all(Val::Px(1.0)),
                                    ..default()
                                },
                                BackgroundColor(theme::card()),
                                theme::card_shadow(),
                                BorderColor::all(theme::hairline()),
                            ))
                            .with_children(|card| {
                                card.spawn(theme::label("Statistics", 11.0, theme::text_dim()));
                                card.spawn((
                                    theme::text_bold(home_stats_value(&stats).as_str(), 16.0, Color::WHITE),
                                    HomeStatsText,
                                ));
                                card.spawn(text(
                                    format!("v{} · Engine Bevy 0.20", env!("CARGO_PKG_VERSION")).as_str(),
                                    12.0,
                                    theme::text_dim(),
                                ));
                            });
                        });

                    // controls card
                    content
                        .spawn((
                            Node {
                                width: Val::Px(546.0),
                                flex_direction: FlexDirection::Column,
                                padding: UiRect::all(Val::Px(16.0)),
                                row_gap: Val::Px(8.0),
                                border_radius: BorderRadius::all(Val::Px(14.0)),
                                border: UiRect::all(Val::Px(1.0)),
                                ..default()
                            },
                            BackgroundColor(theme::card()),
                            theme::card_shadow(),
                            BorderColor::all(theme::hairline()),
                        ))
                        .with_children(|card| {
                            card.spawn(theme::label("Controls", 11.0, theme::text_dim()));
                            for (keys, what) in [
                                ("W / S · A / D", "Pitch · Roll"),
                                ("Q / E", "Yaw"),
                                ("SPACE / SHIFT", "Throttle up / down"),
                                ("ENTER", "Arm motors"),
                                ("R", "Reset drone & race lap"),
                                ("M · C · ESC", "Flight mode · Camera · Pause"),
                            ] {
                                card.spawn((
                                    Node {
                                        width: Val::Percent(100.0),
                                        height: Val::Px(30.0),
                                        flex_direction: FlexDirection::Row,
                                        align_items: AlignItems::Center,
                                        column_gap: Val::Px(12.0),
                                        ..default()
                                    },
                                ))
                                .with_children(|r| {
                                    r.spawn((
                                        Node {
                                            width: Val::Px(150.0),
                                            height: Val::Px(24.0),
                                            justify_content: JustifyContent::Center,
                                            align_items: AlignItems::Center,
                                            border_radius: BorderRadius::all(Val::Px(6.0)),
                                            ..default()
                                        },
                                        BackgroundColor(theme::row()),
                                    ))
                                    .with_children(|k| {
                                        k.spawn(theme::text_bold(keys, 12.0, Color::WHITE));
                                    });
                                    r.spawn(text(what, 13.0, theme::text_dim()));
                                });
                            }
                        });
                });
        });
}

fn home_stats_value(stats: &FlightStats) -> String {
    if stats.flights == 0 {
        "No flights yet — take off!".to_string()
    } else {
        format!(
            "{} flights · best {}",
            stats.flights,
            if stats.best_lap > 0.0 { fmt_time(stats.best_lap) } else { "--:--.-".to_string() }
        )
    }
}

// ---- Controller page ----

fn controller_image(asset_server: &AssetServer, name: &str, kind: &str) -> Handle<Image> {
    let specific = format!("controllers/{}.png", crate::controllers::slug(name));
    if theme::asset_exists(&specific) {
        return asset_server.load(specific);
    }
    let kind_path = format!("controllers/_kind_{kind}.png");
    if theme::asset_exists(&kind_path) {
        return asset_server.load(kind_path);
    }
    default()
}

fn kind_of(db: &[ControllerEntry], name: &str) -> &'static str {
    db.iter()
        .find(|e| e.name == name)
        .map(|e| e.kind)
        .unwrap_or("Gamepad")
}

fn spawn_controller_page(
    mut commands: Commands,
    settings: Res<GameSettings>,
    live: Res<LiveInput>,
    mapping: Res<InputMapping>,
    cal: Res<CalibrationState>,
    asset_server: Res<AssetServer>,
    search: Res<ControllerSearch>,
) {
    let db = controller_database();
    commands
        .spawn((shell(), BackgroundColor(theme::bg()), PageRoot))
        .with_children(|root| {
            sidebar(root, AppState::Controller, &asset_server);
            root.spawn((main_area(), BackgroundColor(theme::bg())))
                .with_children(|page| {
                    page_header(
                        page,
                        &asset_server,
                        "controller",
                        "Controller",
                        "Configure your controller and input settings",
                    );

                    page.spawn((
                        Node {
                            width: Val::Percent(100.0),
                            flex_grow: 1.0,
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(16.0),
                            ..default()
                        },
                    ))
                    .with_children(|body| {
                        // ================= LEFT: database =================
                        body.spawn((
                            Node {
                                width: Val::Percent(36.0),
                                flex_direction: FlexDirection::Column,
                                ..default()
                            },
                        ))
                        .with_children(|left| {
                            left.spawn((
                                Node {
                                    flex_grow: 1.0,
                                    flex_direction: FlexDirection::Column,
                                    padding: UiRect::all(Val::Px(14.0)),
                                    row_gap: Val::Px(12.0),
                                    border_radius: BorderRadius::all(Val::Px(14.0)),
                                    border: UiRect::all(Val::Px(1.0)),
                                    ..default()
                                },
                                BackgroundColor(theme::card()),
                                theme::card_shadow(),
                                BorderColor::all(theme::hairline()),
                            ))
                            .with_children(|card| {
                                // title
                                card.spawn((
                                    Node {
                                        column_gap: Val::Px(8.0),
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },
                                ))
                                .with_children(|t| {
                                    t.spawn(icon_box(&asset_server, "controller", 18.0));
                                    t.spawn(theme::text_bold("Controller Database", 16.0, Color::WHITE));
                                    t.spawn((Node { flex_grow: 1.0, ..default() },));
                                    t.spawn((
                                        theme::label(&format!("{} devices", db.len()), 10.0, theme::text_dim()),
                                    ));
                                });

                                // search box
                                card.spawn((
                                    Node {
                                        width: Val::Percent(100.0),
                                        height: Val::Px(38.0),
                                        align_items: AlignItems::Center,
                                        column_gap: Val::Px(9.0),
                                        padding: UiRect::left(Val::Px(12.0)),
                                        border_radius: BorderRadius::all(Val::Px(10.0)),
                                        border: UiRect::all(Val::Px(1.0)),
                                        ..default()
                                    },
                                    BackgroundColor(theme::row()),
                                    BorderColor::all(theme::border()),
                                ))
                                .with_children(|sb| {
                                    sb.spawn(icon_box(&asset_server, "search", 14.0));
                                    sb.spawn((
                                        Text::new(
                                            if search.query.is_empty() {
                                                "Search controllers...".to_string()
                                            } else {
                                                search.query.clone()
                                            },
                                        ),
                                        TextFont { font_size: bevy::text::FontSize::Px(13.0), ..default() },
                                        TextColor(if search.query.is_empty() {
                                            theme::text_dim()
                                        } else {
                                            Color::WHITE
                                        }),
                                        SearchBoxText,
                                    ));
                                });

                                // scrollable list
                                card.spawn((
                                    Node {
                                        width: Val::Percent(100.0),
                                        flex_grow: 1.0,
                                        flex_direction: FlexDirection::Column,
                                        row_gap: Val::Px(4.0),
                                        overflow: Overflow::scroll_y(),
                                        ..default()
                                    },
                                    ScrollPosition::default(),
                                    ControllerListContainer,
                                ))
                                .with_children(|list| {
                                    let qstr = search.query.to_lowercase();
                                    let filtered: Vec<ControllerEntry> = db
                                        .iter()
                                        .filter(|c| {
                                            qstr.is_empty()
                                                || c.name.to_lowercase().contains(&qstr)
                                                || c.vendor.to_lowercase().contains(&qstr)
                                                || c.kind.to_lowercase().contains(&qstr)
                                        })
                                        .cloned()
                                        .collect();
                                    let mut vendors: Vec<&str> = Vec::new();
                                    for c in &filtered {
                                        if !vendors.contains(&c.vendor) {
                                            vendors.push(c.vendor);
                                        }
                                    }
                                    for v in vendors {
                                        // vendor group label (hidden live when a vendor has no matches)
                                        list.spawn((
                                            Node {
                                                margin: UiRect {
                                                    top: Val::Px(8.0),
                                                    bottom: Val::Px(4.0),
                                                    ..default()
                                                },
                                                ..default()
                                            },
                                            VendorLabel(v.to_string()),
                                        ))
                                        .with_children(|vh| {
                                            vh.spawn(theme::label(v, 10.0, theme::accent()));
                                        });

                                        for c in filtered.iter().filter(|c| c.vendor == v) {
                                            let selected =
                                                settings.selected_controller == c.name;
                                            list.spawn((
                                                Node {
                                                    width: Val::Percent(100.0),
                                                    height: Val::Px(54.0),
                                                    column_gap: Val::Px(12.0),
                                                    align_items: AlignItems::Center,
                                                    padding: UiRect::horizontal(Val::Px(10.0)),
                                                    border_radius: BorderRadius::all(Val::Px(10.0)),
                                                    border: if selected {
                                                        UiRect::all(Val::Px(1.0))
                                                    } else {
                                                        UiRect::all(Val::Px(0.0))
                                                    },
                                                    ..default()
                                                },
                                                BackgroundColor(if selected {
                                                    theme::accent_dim()
                                                } else {
                                                    theme::row()
                                                }),
                                                BorderColor::all(if selected {
                                                    theme::accent()
                                                } else {
                                                    theme::none()
                                                }),
                                                ControllerItem {
                                                    name: c.name.to_string(),
                                                    vendor: c.vendor.to_string(),
                                                },
                                                Interaction::default(),
                                            ))
                                            .with_children(|r| {
                                                r.spawn((
                                                    Node {
                                                        width: Val::Px(36.0),
                                                        height: Val::Px(36.0),
                                                        border_radius: BorderRadius::all(Val::Px(8.0)),
                                                        flex_shrink: 0.0,
                                                        border: UiRect::all(Val::Px(1.0)),
                                                        ..default()
                                                    },
                                                    BackgroundColor(theme::row_alt()),
                                                    BorderColor::all(theme::hairline()),
                                                    ImageNode {
                                                        image: controller_image(
                                                            &asset_server,
                                                            c.name,
                                                            c.kind,
                                                        ),
                                                        ..default()
                                                    },
                                                ));
                                                r.spawn((
                                                    Node {
                                                        flex_grow: 1.0,
                                                        flex_direction: FlexDirection::Column,
                                                        row_gap: Val::Px(2.0),
                                                        ..default()
                                                    },
                                                ))
                                                .with_children(|nm| {
                                                    nm.spawn(text(c.name, 14.0, Color::WHITE));
                                                    nm.spawn(text(c.vendor, 11.0, theme::text_dim()));
                                                });
                                                theme::badge(r, c.kind, theme::kind_color(c.kind));
                                            });
                                        }
                                    }
                                });
                            });
                        });

                        // ================= RIGHT: banner / info / mapping =================
                        body.spawn((
                            Node {
                                flex_grow: 1.0,
                                flex_direction: FlexDirection::Column,
                                row_gap: Val::Px(14.0),
                                overflow: Overflow::scroll_y(),
                                ..default()
                            },
                            ScrollPosition::default(),
                        ))
                        .with_children(|right| {
                            // ---- connected banner ----
                            right.spawn((
                                Node {
                                    width: Val::Percent(100.0),
                                    padding: UiRect::all(Val::Px(16.0)),
                                    flex_direction: FlexDirection::Row,
                                    align_items: AlignItems::Center,
                                    column_gap: Val::Px(12.0),
                                    border_radius: BorderRadius::all(Val::Px(14.0)),
                                    border: UiRect::all(Val::Px(1.0)),
                                    ..default()
                                },
                                BackgroundColor(theme::card()),
                                theme::card_shadow(),
                                BorderColor::all(if live.connected {
                                    Color::srgba(0.25, 0.85, 0.45, 0.4)
                                } else {
                                    theme::border()
                                }),
                            ))
                            .with_children(|bn| {
                                bn.spawn((
                                    Node {
                                        width: Val::Px(10.0),
                                        height: Val::Px(10.0),
                                        border_radius: BorderRadius::all(Val::Px(5.0)),
                                        flex_shrink: 0.0,
                                        ..default()
                                    },
                                    BackgroundColor(if live.connected { theme::green() } else { Color::srgb(0.5, 0.5, 0.5) }),
                                    BoxShadow::new(
                                        if live.connected {
                                            theme::green().with_alpha(0.6)
                                        } else {
                                            Color::NONE
                                        },
                                        Val::Px(0.0),
                                        Val::Px(0.0),
                                        Val::Px(1.0),
                                        Val::Px(8.0),
                                    ),
                                    ConnStatus,
                                ));
                                bn.spawn((
                                    Node {
                                        flex_grow: 1.0,
                                        flex_direction: FlexDirection::Column,
                                        row_gap: Val::Px(2.0),
                                        ..default()
                                    },
                                ))
                                .with_children(|tx| {
                                    tx.spawn((
                                        theme::text_bold(
                                            if live.connected { "Controller Connected" } else { "No Controller Detected" },
                                            15.0,
                                            if live.connected { theme::green() } else { theme::text_dim() },
                                        ),
                                        ConnStatusText,
                                    ));
                                    tx.spawn((
                                        Text::new(if live.connected {
                                            live.name.clone()
                                        } else {
                                            "Connect a radio or gamepad to see it here".to_string()
                                        }),
                                        TextFont { font_size: bevy::text::FontSize::Px(12.0), ..default() },
                                        TextColor(theme::text_dim()),
                                        LiveNameText,
                                    ));
                                });
                                // signal-strength decoration
                                bn.spawn((
                                    Node {
                                        flex_direction: FlexDirection::Row,
                                        column_gap: Val::Px(3.0),
                                        align_items: AlignItems::FlexEnd,
                                        margin: UiRect::right(Val::Px(4.0)),
                                        ..default()
                                    },
                                ))
                                .with_children(|bars| {
                                    for h in [6.0f32, 10.0, 14.0, 18.0] {
                                        bars.spawn((
                                            Node {
                                                width: Val::Px(4.0),
                                                height: Val::Px(h),
                                                border_radius: BorderRadius::all(Val::Px(2.0)),
                                                ..default()
                                            },
                                            BackgroundColor(theme::accent()),
                                        ));
                                    }
                                });
                            });

                            // ---- controller info + calibrate ----
                            right.spawn((
                                Node {
                                    width: Val::Percent(100.0),
                                    padding: UiRect::all(Val::Px(16.0)),
                                    flex_direction: FlexDirection::Row,
                                    column_gap: Val::Px(18.0),
                                    border_radius: BorderRadius::all(Val::Px(14.0)),
                                    border: UiRect::all(Val::Px(1.0)),
                                    ..default()
                                },
                                BackgroundColor(theme::card()),
                                theme::card_shadow(),
                                BorderColor::all(theme::hairline()),
                            ))
                            .with_children(|card| {
                                card.spawn((
                                    Node {
                                        width: Val::Px(210.0),
                                        height: Val::Px(150.0),
                                        border_radius: BorderRadius::all(Val::Px(12.0)),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        flex_shrink: 0.0,
                                        border: UiRect::all(Val::Px(1.0)),
                                        ..default()
                                    },
                                    BackgroundColor(theme::row_alt()),
                                    BorderColor::all(theme::hairline()),
                                    ImageNode {
                                        image: controller_image(
                                            &asset_server,
                                            &settings.selected_controller,
                                            kind_of(&db, &settings.selected_controller),
                                        ),
                                        ..default()
                                    },
                                    ControllerImage,
                                ));
                                card.spawn((
                                    Node {
                                        flex_grow: 1.0,
                                        flex_direction: FlexDirection::Column,
                                        row_gap: Val::Px(8.0),
                                        ..default()
                                    },
                                ))
                                .with_children(|info| {
                                    info.spawn(theme::text_bold("Controller Info", 16.0, Color::WHITE));

                                    info_row(
                                        info,
                                        "Name",
                                        &settings.selected_controller,
                                        SelectedControllerText,
                                        Color::WHITE,
                                    );
                                    info_row(
                                        info,
                                        "Type",
                                        kind_of(&db, &settings.selected_controller),
                                        KindText,
                                        theme::accent(),
                                    );
                                    info_row(
                                        info,
                                        "Connection",
                                        if live.connected { "Connected" } else { "Waiting for device" },
                                        InfoConnText,
                                        if live.connected { theme::green() } else { theme::text_dim() },
                                    );

                                    info.spawn((
                                        Text::new(calibration_status(&cal)),
                                        TextFont { font_size: bevy::text::FontSize::Px(12.0), ..default() },
                                        TextColor(theme::text_dim()),
                                        CalibStatusText,
                                    ));

                                    info.spawn((
                                        Node {
                                            width: Val::Px(200.0),
                                            height: Val::Px(40.0),
                                            justify_content: JustifyContent::Center,
                                            align_items: AlignItems::Center,
                                            border_radius: BorderRadius::all(Val::Px(10.0)),
                                            margin: UiRect::top(Val::Px(4.0)),
                                            ..default()
                                        },
                                        BackgroundColor(theme::accent()),
                                        theme::glow_shadow(),
                                        HoverTint { base: theme::accent(), hover: theme::accent_hover() },
                                        CalibrateBtn,
                                        Interaction::default(),
                                    ))
                                    .with_children(|b| {
                                        b.spawn(theme::text_bold("Calibrate Sticks", 14.0, Color::WHITE));
                                    });
                                });
                            });

                            // ---- input mapping ----
                            right.spawn((
                                Node {
                                    width: Val::Percent(100.0),
                                    padding: UiRect::all(Val::Px(16.0)),
                                    flex_direction: FlexDirection::Column,
                                    row_gap: Val::Px(10.0),
                                    border_radius: BorderRadius::all(Val::Px(14.0)),
                                    border: UiRect::all(Val::Px(1.0)),
                                    ..default()
                                },
                                BackgroundColor(theme::card()),
                                theme::card_shadow(),
                                BorderColor::all(theme::hairline()),
                            ))
                            .with_children(|card| {
                                // title + sensitivity
                                card.spawn((
                                    Node {
                                        width: Val::Percent(100.0),
                                        flex_direction: FlexDirection::Row,
                                        justify_content: JustifyContent::SpaceBetween,
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },
                                ))
                                .with_children(|tr| {
                                    tr.spawn(theme::text_bold("Input Mapping", 16.0, Color::WHITE));
                                    tr.spawn((
                                        Node {
                                            flex_direction: FlexDirection::Row,
                                            column_gap: Val::Px(8.0),
                                            align_items: AlignItems::Center,
                                            ..default()
                                        },
                                    ))
                                    .with_children(|g| {
                                        g.spawn(theme::label("Sensitivity", 10.0, theme::text_dim()));
                                        g.spawn((
                                            Node {
                                                width: Val::Px(26.0),
                                                height: Val::Px(26.0),
                                                justify_content: JustifyContent::Center,
                                                align_items: AlignItems::Center,
                                                border_radius: BorderRadius::all(Val::Px(6.0)),
                                                ..default()
                                            },
                                            BackgroundColor(theme::row()),
                                            HoverTint { base: theme::row(), hover: theme::row_hover() },
                                            SensDown,
                                            Interaction::default(),
                                        ))
                                        .with_children(|b| {
                                            b.spawn(text("−", 15.0, Color::WHITE));
                                        });
                                        g.spawn((
                                            Text::new(format!("{:.1}", mapping.sensitivity)),
                                            TextFont { font_size: bevy::text::FontSize::Px(13.0), ..default() },
                                            TextColor(theme::accent()),
                                            MapSensText,
                                        ));
                                        g.spawn((
                                            Node {
                                                width: Val::Px(26.0),
                                                height: Val::Px(26.0),
                                                justify_content: JustifyContent::Center,
                                                align_items: AlignItems::Center,
                                                border_radius: BorderRadius::all(Val::Px(6.0)),
                                                ..default()
                                            },
                                            BackgroundColor(theme::row()),
                                            HoverTint { base: theme::row(), hover: theme::row_hover() },
                                            SensUp,
                                            Interaction::default(),
                                        ))
                                        .with_children(|b| {
                                            b.spawn(text("+", 15.0, Color::WHITE));
                                        });
                                    });
                                });

                                let axes = [
                                    ("Roll", mapping.roll, mapping.roll_reversed, live.roll),
                                    ("Pitch", mapping.pitch, mapping.pitch_reversed, live.pitch),
                                    ("Yaw", mapping.yaw, mapping.yaw_reversed, live.yaw),
                                    ("Throttle", mapping.throttle, mapping.throttle_reversed, live.throttle),
                                ];
                                for (i, (label, axis, rev, val)) in axes.into_iter().enumerate() {
                                    mapping_row(card, i, label, axis, rev, val);
                                }

                                card.spawn(text(
                                    "Click an axis chip to cycle it · REV flips the direction",
                                    11.0,
                                    theme::text_dim(),
                                ));
                                card.spawn((
                                    Text::new("Buttons: none"),
                                    TextFont { font_size: bevy::text::FontSize::Px(11.0), ..default() },
                                    TextColor(theme::text_dim()),
                                    LiveButtonsText,
                                ));
                            });

                            // ---- advanced settings ----
                            right.spawn((
                                Node {
                                    width: Val::Percent(100.0),
                                    height: Val::Px(48.0),
                                    flex_direction: FlexDirection::Row,
                                    align_items: AlignItems::Center,
                                    column_gap: Val::Px(10.0),
                                    padding: UiRect::left(Val::Px(16.0)),
                                    border_radius: BorderRadius::all(Val::Px(10.0)),
                                    border: UiRect::all(Val::Px(1.0)),
                                    ..default()
                                },
                                BackgroundColor(theme::row()),
                                BorderColor::all(theme::hairline()),
                                NavBtn(AppState::Settings),
                                Interaction::default(),
                                HoverTint { base: theme::row(), hover: theme::row_hover() },
                            ))
                            .with_children(|adv| {
                                adv.spawn(icon_box(&asset_server, "settings", 18.0));
                                adv.spawn(theme::text_bold("Advanced Settings", 14.0, Color::WHITE));
                                adv.spawn((Node { flex_grow: 1.0, ..default() },));
                                adv.spawn(text("›", 18.0, theme::text_dim()));
                            });
                        });
                    });
                });
        });
}

fn info_row<M: Component>(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    value: &str,
    marker: M,
    color: Color,
) {
    parent
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(26.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(12.0),
                ..default()
            },
        ))
        .with_children(|r| {
            r.spawn((
                Node { width: Val::Px(92.0), flex_shrink: 0.0, ..default() },
            ))
            .with_children(|l| {
                l.spawn(theme::label(label, 10.0, theme::text_dim()));
            });
            r.spawn((
                Text::new(value.to_string()),
                TextFont { font_size: bevy::text::FontSize::Px(13.0), ..default() },
                TextColor(color),
                marker,
            ));
        });
}

fn mapping_row(
    card: &mut ChildSpawnerCommands,
    i: usize,
    label: &str,
    axis: AxisSource,
    reversed: bool,
    val: f32,
) {
    card.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(38.0),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(12.0),
            padding: UiRect::horizontal(Val::Px(12.0)),
            border_radius: BorderRadius::all(Val::Px(8.0)),
            ..default()
        },
        BackgroundColor(theme::row_alt()),
    ))
    .with_children(|r| {
        r.spawn((
            Node { width: Val::Px(66.0), flex_shrink: 0.0, ..default() },
        ))
        .with_children(|l| {
            l.spawn(theme::text_bold(label, 13.0, Color::WHITE));
        });

        // live bar (center-origin for sticks, left-origin for throttle)
        r.spawn((
            Node {
                flex_grow: 1.0,
                height: Val::Px(8.0),
                border_radius: BorderRadius::all(Val::Px(4.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.08)),
        ))
        .with_children(|bar| {
            let (left, width) = fill_pct(i, val);
            bar.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Percent(left),
                    width: Val::Percent(width),
                    height: Val::Percent(100.0),
                    border_radius: BorderRadius::all(Val::Px(4.0)),
                    ..default()
                },
                BackgroundColor(theme::accent()),
                LiveAxisFill(i),
            ));
        });

        r.spawn((
            Node { width: Val::Px(52.0), flex_shrink: 0.0, ..default() },
        ))
        .with_children(|v| {
            v.spawn((
                Text::new(format!("{val:+.2}")),
                TextFont { font_size: bevy::text::FontSize::Px(12.0), ..default() },
                TextColor(theme::text_dim()),
                LiveAxisText(i),
            ));
        });

        // axis chip (click cycles the source)
        r.spawn((
            Node {
                height: Val::Px(26.0),
                padding: UiRect::horizontal(Val::Px(10.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border_radius: BorderRadius::all(Val::Px(6.0)),
                flex_shrink: 0.0,
                ..default()
            },
            BackgroundColor(theme::accent_dim()),
            MapCycleAxis(i),
            Interaction::default(),
            HoverTint { base: theme::accent_dim(), hover: theme::accent_soft() },
        ))
        .with_children(|c| {
            c.spawn((
                Text::new(axis_label(axis, reversed)),
                TextFont { font_size: bevy::text::FontSize::Px(11.0), ..default() },
                TextColor(Color::WHITE),
                MapAxisChip(i),
            ));
        });

        // reverse toggle
        r.spawn((
            Node {
                width: Val::Px(36.0),
                height: Val::Px(26.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border_radius: BorderRadius::all(Val::Px(6.0)),
                flex_shrink: 0.0,
                ..default()
            },
            BackgroundColor(theme::row()),
            MapToggleReverse(i),
            Interaction::default(),
            HoverTint { base: theme::row(), hover: theme::row_hover() },
        ))
        .with_children(|b| {
            b.spawn(text("REV", 9.0, theme::text_dim()));
        });
    });
}

fn axis_label(axis: AxisSource, reversed: bool) -> String {
    if reversed {
        format!("{} ↓", axis.label())
    } else {
        axis.label().to_string()
    }
}

fn fill_pct(i: usize, v: f32) -> (f32, f32) {
    if i == 3 {
        (0.0, v.clamp(0.0, 1.0) * 100.0)
    } else if v >= 0.0 {
        (50.0, v * 50.0)
    } else {
        (50.0 + v * 50.0, -v * 50.0)
    }
}

fn calibration_status(cal: &CalibrationState) -> String {
    match cal.phase {
        CalibrationPhase::Idle => {
            if cal.calibrated {
                "Sticks calibrated".to_string()
            } else {
                "Not calibrated yet — recommended".to_string()
            }
        }
        CalibrationPhase::Center => format!("Capturing center… ({}/90)", cal.samples),
        CalibrationPhase::MinMax => format!("Move sticks to extremes… ({}/240)", cal.samples),
    }
}

// ---- Settings page ----

fn spawn_settings_page(mut commands: Commands, settings: Res<GameSettings>, asset_server: Res<AssetServer>) {
    commands
        .spawn((shell(), BackgroundColor(theme::bg()), PageRoot))
        .with_children(|root| {
            sidebar(root, AppState::Settings, &asset_server);
            root.spawn((
                Node {
                    flex_grow: 1.0,
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(28.0)),
                    row_gap: Val::Px(16.0),
                    overflow: Overflow::scroll_y(),
                    ..default()
                },
                BackgroundColor(theme::bg()),
                ScrollPosition::default(),
            ))
            .with_children(|page| {
                page_header(
                    page,
                    &asset_server,
                    "settings",
                    "Settings",
                    "Tune flight, camera and input to your style",
                );

                page.spawn((
                    Node {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(16.0),
                        align_items: AlignItems::FlexStart,
                        ..default()
                    },
                ))
                .with_children(|cols| {
                    // left column
                    cols.spawn((
                        Node {
                            flex_grow: 1.0,
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(16.0),
                            ..default()
                        },
                    ))
                    .with_children(|col| {
                        settings_card(col, &asset_server, "Flight", |card| {
                            slider_row(card, "Flight Mode (M)", "acro", settings_value_text("acro", &settings));
                            slider_row(card, "Rotation Profile", "rot", settings_value_text("rot", &settings));
                            slider_row(card, "Controller Sensitivity", "sens", settings_value_text("sens", &settings));
                            slider_row(card, "Stick Expo", "expo", settings_value_text("expo", &settings));
                            slider_row(card, "Deadzone", "dead", settings_value_text("dead", &settings));
                        });

                        settings_card(col, &asset_server, "Rates (Actual)", |card| {
                            slider_row(card, "Center Sensitivity", "rc", settings_value_text("rc", &settings));
                            slider_row(card, "Max Rate", "rm", settings_value_text("rm", &settings));
                            slider_row(card, "Angle Max Tilt", "mt", settings_value_text("mt", &settings));
                        });
                    });

                    // right column
                    cols.spawn((
                        Node {
                            flex_grow: 1.0,
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(16.0),
                            ..default()
                        },
                    ))
                    .with_children(|col| {
                        settings_card(col, &asset_server, "Camera", |card| {
                            slider_row(card, "Field of View", "fov", settings_value_text("fov", &settings));
                            slider_row(card, "Camera Tilt", "tilt", settings_value_text("tilt", &settings));
                            slider_row(card, "Camera", "chase", settings_value_text("chase", &settings));
                            slider_row(card, "Invert Pitch", "invert", settings_value_text("invert", &settings));
                        });

                        settings_card(col, &asset_server, "System", |card| {
                            slider_row(card, "Master Volume", "volume", settings_value_text("volume", &settings));
                            slider_row(card, "Graphics Quality", "quality", settings_value_text("quality", &settings));
                        });
                    });
                });
            });
        });
}

fn settings_card<F: FnOnce(&mut ChildSpawnerCommands)>(
    parent: &mut ChildSpawnerCommands,
    asset_server: &AssetServer,
    title: &'static str,
    fill: F,
) {
    parent
        .spawn((
            Node {
                width: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(14.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                border_radius: BorderRadius::all(Val::Px(14.0)),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(theme::card()),
            theme::card_shadow(),
            BorderColor::all(theme::hairline()),
        ))
        .with_children(|card| {
            card.spawn((
                Node {
                    column_gap: Val::Px(8.0),
                    align_items: AlignItems::Center,
                    margin: UiRect::bottom(Val::Px(4.0)),
                    ..default()
                },
            ))
            .with_children(|h| {
                h.spawn(icon_box(asset_server, "settings", 16.0));
                h.spawn(theme::label(title, 11.0, theme::accent()));
            });
            fill(card);
        });
}

fn slider_row(parent: &mut ChildSpawnerCommands, label: &str, key: &'static str, value: String) {
    parent
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(44.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: UiRect::horizontal(Val::Px(14.0)),
                border_radius: BorderRadius::all(Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(theme::row()),
        ))
        .with_children(|row| {
            row.spawn(text(label, 14.0, Color::WHITE));
            row.spawn((
                Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(8.0),
                    align_items: AlignItems::Center,
                    ..default()
                },
            ))
            .with_children(|g| {
                g.spawn((
                    Node {
                        width: Val::Px(28.0),
                        height: Val::Px(28.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border_radius: BorderRadius::all(Val::Px(7.0)),
                        ..default()
                    },
                    BackgroundColor(theme::row_hover()),
                    SettingBtn(key, -1),
                    Interaction::default(),
                    HoverTint { base: theme::row_hover(), hover: theme::accent_dim() },
                ))
                .with_children(|b| {
                    b.spawn(text("−", 14.0, Color::WHITE));
                });
                g.spawn((
                    Node {
                        min_width: Val::Px(92.0),
                        height: Val::Px(28.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        padding: UiRect::horizontal(Val::Px(8.0)),
                        border_radius: BorderRadius::all(Val::Px(7.0)),
                        ..default()
                    },
                    BackgroundColor(theme::accent_dim()),
                    SettingBtn(key, 0),
                    Interaction::default(),
                    HoverTint { base: theme::accent_dim(), hover: theme::accent_soft() },
                ))
                .with_children(|v| {
                    v.spawn((
                        Text::new(value),
                        TextFont { font_size: bevy::text::FontSize::Px(13.0), ..default() },
                        TextColor(Color::WHITE),
                        SettingValue(key),
                    ));
                });
                g.spawn((
                    Node {
                        width: Val::Px(28.0),
                        height: Val::Px(28.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border_radius: BorderRadius::all(Val::Px(7.0)),
                        ..default()
                    },
                    BackgroundColor(theme::row_hover()),
                    SettingBtn(key, 1),
                    Interaction::default(),
                    HoverTint { base: theme::row_hover(), hover: theme::accent_dim() },
                ))
                .with_children(|b| {
                    b.spawn(text("+", 14.0, Color::WHITE));
                });
            });
        });
}

fn settings_value_text(key: &str, s: &GameSettings) -> String {
    match key {
        "volume" => format!("{:.0}%", s.master_volume * 100.0),
        "quality" => ["Low", "Medium", "High", "Ultra"][s.graphics_quality as usize].to_string(),
        "sens" => format!("{:.2}", s.controller_sensitivity),
        "rot" => ["Slow", "Normal", "Fast"][s.rotation_profile as usize].to_string(),
        "dead" => format!("{:.2}", s.deadzone),
        "expo" => format!("{:.2}", s.expo),
        "invert" => (if s.invert_pitch { "On" } else { "Off" }).to_string(),
        "acro" => s.flight_mode_label().to_string(),
        "rc" => format!("{:.0} °/s", s.rate_center),
        "rm" => format!("{:.0} °/s", s.rate_max),
        "mt" => format!("{:.0}°", s.max_tilt),
        "fov" => format!("{:.0}°", s.fov),
        "tilt" => format!("{:.0}°", s.camera_tilt),
        "chase" => (if s.chase_cam { "Chase" } else { "FPV" }).to_string(),
        _ => "-".to_string(),
    }
}

fn apply_setting(s: &mut GameSettings, key: &str, dir: i32) {
    let d = dir as f32;
    match key {
        "volume" => s.master_volume = (s.master_volume + 0.05 * d).clamp(0.0, 1.0),
        "quality" => {
            s.graphics_quality = ((s.graphics_quality as i32 + dir).clamp(0, 3)) as u8;
        }
        "sens" => s.controller_sensitivity = (s.controller_sensitivity + 0.05 * d).clamp(0.2, 3.0),
        "rot" => {
            s.rotation_profile = ((s.rotation_profile as i32 + dir).clamp(0, 2)) as u8;
        }
        "dead" => s.deadzone = (s.deadzone + 0.01 * d).clamp(0.0, 0.4),
        "expo" => s.expo = (s.expo + 0.05 * d).clamp(0.0, 0.9),
        "invert" => s.invert_pitch = !s.invert_pitch,
        "acro" => s.flight_mode_acro = !s.flight_mode_acro,
        "rc" => s.rate_center = (s.rate_center + 10.0 * d).clamp(40.0, 700.0),
        "rm" => s.rate_max = (s.rate_max + 20.0 * d).clamp(200.0, 1600.0),
        "mt" => s.max_tilt = (s.max_tilt + 5.0 * d).clamp(15.0, 70.0),
        "fov" => s.fov = (s.fov + 5.0 * d).clamp(60.0, 130.0),
        "tilt" => s.camera_tilt = (s.camera_tilt + 1.0 * d).clamp(0.0, 45.0),
        "chase" => s.chase_cam = !s.chase_cam,
        _ => {}
    }
}

fn settings_buttons(
    mut settings: ResMut<GameSettings>,
    q: Query<(&Interaction, &SettingBtn), Changed<Interaction>>,
    mut vals: Query<(&SettingValue, &mut Text)>,
) {
    let mut changed = false;
    for (interaction, btn) in q.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }
        apply_setting(&mut settings, btn.0, btn.1);
        changed = true;
    }
    if changed {
        for (key, mut t) in vals.iter_mut() {
            t.0 = settings_value_text(key.0, &settings);
        }
    }
}

// ---- Addons page ----

fn spawn_addons_page(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn((shell(), BackgroundColor(theme::bg()), PageRoot))
        .with_children(|root| {
            sidebar(root, AppState::Addons, &asset_server);
            root.spawn((main_area(), BackgroundColor(theme::bg())))
                .with_children(|page| {
                    page_header(
                        page,
                        &asset_server,
                        "addons",
                        "Addons",
                        "Content packs that extend the simulator",
                    );
                    page.spawn((
                        Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(16.0),
                            align_items: AlignItems::FlexStart,
                            ..default()
                        },
                    ))
                    .with_children(|row| {
                        addon_card(
                            row,
                            &asset_server,
                            "Race Course",
                            "6-gate oval with lap timing, best-lap tracking and gate hints. Included in every session.",
                            "ENABLED",
                            theme::green(),
                        );
                        addon_card(
                            row,
                            &asset_server,
                            "Freestyle Park",
                            "Buildings, trees and open lines to practice gaps and dives. Included in every session.",
                            "ENABLED",
                            theme::green(),
                        );
                        addon_card(
                            row,
                            &asset_server,
                            "Night Mode",
                            "Low-light flying with gate LEDs and headlight beams.",
                            "SOON",
                            theme::accent(),
                        );
                    });
                });
        });
}

fn addon_card(
    parent: &mut ChildSpawnerCommands,
    asset_server: &AssetServer,
    title: &str,
    desc: &str,
    badge: &str,
    badge_color: Color,
) {
    parent
        .spawn((
            Node {
                width: Val::Px(280.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(16.0)),
                row_gap: Val::Px(8.0),
                border_radius: BorderRadius::all(Val::Px(14.0)),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(theme::card()),
            theme::card_shadow(),
            BorderColor::all(theme::hairline()),
        ))
        .with_children(|card| {
            card.spawn((
                Node {
                    column_gap: Val::Px(10.0),
                    align_items: AlignItems::Center,
                    ..default()
                },
            ))
            .with_children(|h| {
                h.spawn(icon_box(asset_server, "addons", 22.0));
                h.spawn(theme::text_bold(title, 16.0, Color::WHITE));
            });
            card.spawn((
                Node {
                    width: Val::Percent(100.0),
                    min_height: Val::Px(54.0),
                    ..default()
                },
            ))
            .with_children(|d| {
                d.spawn(text(desc, 12.5, theme::text_dim()));
            });
            theme::badge(card, badge, badge_color);
        });
}

// ---- Statistics page ----

fn spawn_statistics_page(mut commands: Commands, stats: Res<FlightStats>, asset_server: Res<AssetServer>) {
    commands
        .spawn((shell(), BackgroundColor(theme::bg()), PageRoot))
        .with_children(|root| {
            sidebar(root, AppState::Statistics, &asset_server);
            root.spawn((main_area(), BackgroundColor(theme::bg())))
                .with_children(|page| {
                    page_header(
                        page,
                        &asset_server,
                        "stats",
                        "Statistics",
                        "Your flying record, saved on this machine",
                    );

                    let rows: [&[(&'static str, &'static str)]; 2] = [
                        &[
                            ("flights", "FLIGHTS"),
                            ("time", "FLIGHT TIME"),
                            ("distance", "DISTANCE"),
                            ("top_speed", "TOP SPEED"),
                        ],
                        &[
                            ("crashes", "CRASHES"),
                            ("laps", "LAPS COMPLETED"),
                            ("best_lap", "BEST LAP"),
                        ],
                    ];
                    for keys in rows {
                        page.spawn((
                            Node {
                                flex_direction: FlexDirection::Row,
                                column_gap: Val::Px(16.0),
                                ..default()
                            },
                        ))
                        .with_children(|row| {
                            for (key, label) in keys {
                                stat_tile(row, key, label, stat_text(key, &stats));
                            }
                        });
                    }

                    // reset button
                    page.spawn((
                        Node {
                            width: Val::Px(190.0),
                            height: Val::Px(42.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border_radius: BorderRadius::all(Val::Px(10.0)),
                            margin: UiRect::top(Val::Px(6.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.5, 0.14, 0.17, 0.8)),
                        HoverTint {
                            base: Color::srgba(0.5, 0.14, 0.17, 0.8),
                            hover: theme::red_hover(),
                        },
                        ResetStatsBtn,
                        Interaction::default(),
                    ))
                    .with_children(|b| {
                        b.spawn((
                            theme::text_bold("RESET STATISTICS", 13.0, Color::WHITE),
                            LetterSpacing::Px(1.0),
                        ));
                    });
                });
        });
}

fn stat_tile(row: &mut ChildSpawnerCommands, key: &'static str, label: &str, value: String) {
    row.spawn((
        Node {
            width: Val::Px(200.0),
            height: Val::Px(96.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(16.0)),
            row_gap: Val::Px(6.0),
            border_radius: BorderRadius::all(Val::Px(14.0)),
            border: UiRect::all(Val::Px(1.0)),
            ..default()
        },
        BackgroundColor(theme::card()),
        theme::card_shadow(),
        BorderColor::all(theme::hairline()),
    ))
    .with_children(|tile| {
        tile.spawn(theme::label(label, 11.0, theme::text_dim()));
        tile.spawn((
            theme::text_bold(&value, 22.0, Color::WHITE),
            StatValue(key),
        ));
    });
}

fn fmt_time(t: f32) -> String {
    let m = (t / 60.0).floor() as u32;
    let s = t - m as f32 * 60.0;
    format!("{m}:{s:04.1}")
}

fn fmt_duration(secs: f32) -> String {
    let total = secs as u32;
    let h = total / 3600;
    let m = (total % 3600) / 60;
    if h > 0 {
        format!("{h}h {m}m")
    } else {
        format!("{m}m {}s", total % 60)
    }
}

fn stat_text(key: &str, s: &FlightStats) -> String {
    match key {
        "flights" => s.flights.to_string(),
        "time" => fmt_duration(s.flight_time),
        "distance" => {
            if s.distance >= 1000.0 {
                format!("{:.2} km", s.distance / 1000.0)
            } else {
                format!("{:.0} m", s.distance)
            }
        }
        "top_speed" => format!("{:.1} km/h", s.top_speed),
        "crashes" => s.crashes.to_string(),
        "laps" => s.laps.to_string(),
        "best_lap" => {
            if s.best_lap > 0.0 {
                fmt_time(s.best_lap)
            } else {
                "--:--.-".to_string()
            }
        }
        _ => "-".to_string(),
    }
}

fn stats_buttons(
    mut stats: ResMut<FlightStats>,
    q: Query<&Interaction, (Changed<Interaction>, With<ResetStatsBtn>)>,
    mut vals: Query<(&StatValue, &mut Text)>,
) {
    for interaction in q.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }
        *stats = FlightStats::default();
        crate::stats::write_stats(&stats);
        for (key, mut t) in vals.iter_mut() {
            t.0 = stat_text(key.0, &stats);
        }
    }
}

// ---- About page ----

fn spawn_about_page(mut commands: Commands, asset_server: Res<AssetServer>) {
    let version = env!("CARGO_PKG_VERSION");
    commands
        .spawn((shell(), BackgroundColor(theme::bg()), PageRoot))
        .with_children(|root| {
            sidebar(root, AppState::About, &asset_server);
            root.spawn((main_area(), BackgroundColor(theme::bg())))
                .with_children(|page| {
                    page_header(
                        page,
                        &asset_server,
                        "about",
                        "About",
                        "FPV Drone Sim — built for pilots, by pilots",
                    );

                    page.spawn((
                        Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(16.0),
                            align_items: AlignItems::FlexStart,
                            ..default()
                        },
                    ))
                    .with_children(|row| {
                        row.spawn((
                            Node {
                                width: Val::Px(360.0),
                                flex_direction: FlexDirection::Column,
                                padding: UiRect::all(Val::Px(18.0)),
                                row_gap: Val::Px(8.0),
                                border_radius: BorderRadius::all(Val::Px(14.0)),
                                border: UiRect::all(Val::Px(1.0)),
                                ..default()
                            },
                            BackgroundColor(theme::card()),
                            theme::card_shadow(),
                            BorderColor::all(theme::hairline()),
                        ))
                        .with_children(|card| {
                            card.spawn(theme::label("Version", 11.0, theme::text_dim()));
                            card.spawn(theme::text_bold(&format!("v{version}"), 22.0, theme::accent()));
                            card.spawn(text(
                                "Engine: Bevy (Rust)",
                                13.0,
                                theme::text_dim(),
                            ));
                            card.spawn(text(
                                "github.com/sifrex19-blip/fpv-drone-sim",
                                12.0,
                                theme::text_dim(),
                            ));
                        });

                        row.spawn((
                            Node {
                                width: Val::Px(440.0),
                                flex_direction: FlexDirection::Column,
                                padding: UiRect::all(Val::Px(18.0)),
                                row_gap: Val::Px(8.0),
                                border_radius: BorderRadius::all(Val::Px(14.0)),
                                border: UiRect::all(Val::Px(1.0)),
                                ..default()
                            },
                            BackgroundColor(theme::card()),
                            theme::card_shadow(),
                            BorderColor::all(theme::hairline()),
                        ))
                        .with_children(|card| {
                            card.spawn(theme::label("Features", 11.0, theme::text_dim()));
                            for feature in [
                                "Acro & Angle flight modes with Betaflight-style rates",
                                "Race gates with lap timer and best-lap tracking",
                                "Battery model with voltage sag + OSD-style HUD",
                                "Controller database, mapping, live input & calibration",
                                "Daytime map with runway, buildings, trees and fog",
                            ] {
                                card.spawn(text(&format!("•  {feature}"), 13.0, Color::WHITE));
                            }
                        });
                    });
                });
        });
}

// ---- search / filter / preview / status ----

fn controller_search_input(
    mut reader: MessageReader<KeyboardInput>,
    mut search: ResMut<ControllerSearch>,
    mut q: Query<(&mut Text, &mut TextColor), With<SearchBoxText>>,
) {
    for ev in reader.read() {
        if ev.state != ButtonState::Pressed {
            continue;
        }
        if ev.key_code == KeyCode::Backspace {
            search.query.pop();
        } else if let Some(t) = &ev.text {
            for ch in t.chars() {
                if !ch.is_control() {
                    search.query.push(ch);
                }
            }
        }
    }
    for (mut txt, mut color) in q.iter_mut() {
        if search.query.is_empty() {
            txt.0 = "Search controllers...".to_string();
            color.0 = theme::text_dim();
        } else {
            txt.0 = search.query.clone();
            color.0 = Color::WHITE;
        }
    }
}

fn filter_controller_list(
    search: Res<ControllerSearch>,
    mut params: ParamSet<(
        Query<(&ControllerItem, &mut Node)>,
        Query<(&VendorLabel, &mut Node)>,
    )>,
) {
    if !search.is_changed() {
        return;
    }
    let qstr = search.query.to_lowercase();
    let mut visible_vendors: std::collections::HashSet<String> = std::collections::HashSet::new();
    for (item, mut node) in params.p0().iter_mut() {
        let m = qstr.is_empty()
            || item.name.to_lowercase().contains(&qstr)
            || item.vendor.to_lowercase().contains(&qstr);
        node.display = if m { Display::Flex } else { Display::None };
        if m {
            visible_vendors.insert(item.vendor.clone());
        }
    }
    for (label, mut node) in params.p1().iter_mut() {
        node.display = if visible_vendors.contains(&label.0) {
            Display::Flex
        } else {
            Display::None
        };
    }
}

fn controller_select(
    mut settings: ResMut<GameSettings>,
    q: Query<(&Interaction, &ControllerItem), Changed<Interaction>>,
) {
    for (interaction, item) in q.iter() {
        if *interaction == Interaction::Pressed {
            settings.selected_controller = item.name.clone();
        }
    }
}

fn update_controller_preview(
    settings: Res<GameSettings>,
    asset_server: Res<AssetServer>,
    mut params: ParamSet<(
        Query<&mut ImageNode, With<ControllerImage>>,
        Query<&mut Text, With<SelectedControllerText>>,
        Query<&mut Text, With<KindText>>,
    )>,
) {
    if !settings.is_changed() {
        return;
    }
    let db = controller_database();
    let kind = kind_of(&db, &settings.selected_controller);
    let handle = controller_image(&asset_server, &settings.selected_controller, kind);
    for mut img in params.p0().iter_mut() {
        img.image = handle.clone();
    }
    for mut t in params.p1().iter_mut() {
        t.0 = settings.selected_controller.clone();
    }
    for mut t in params.p2().iter_mut() {
        t.0 = kind.to_string();
    }
}

fn update_controller_status(
    live: Res<LiveInput>,
    mut params: ParamSet<(
        Query<&mut BackgroundColor, With<ConnStatus>>,
        Query<(&mut Text, &mut TextColor), With<ConnStatusText>>,
        Query<(&mut Text, &mut TextColor), With<InfoConnText>>,
        Query<&mut Text, With<LiveNameText>>,
    )>,
) {
    for mut bg in params.p0().iter_mut() {
        bg.0 = if live.connected { theme::green() } else { Color::srgb(0.5, 0.5, 0.5) };
    }
    for (mut t, mut c) in params.p1().iter_mut() {
        if live.connected {
            t.0 = "Controller Connected".to_string();
            c.0 = theme::green();
        } else {
            t.0 = "No Controller Detected".to_string();
            c.0 = theme::text_dim();
        }
    }
    for (mut t, mut c) in params.p2().iter_mut() {
        if live.connected {
            t.0 = "Connected".to_string();
            c.0 = theme::green();
        } else {
            t.0 = "Waiting for device".to_string();
            c.0 = theme::text_dim();
        }
    }
    for mut t in params.p3().iter_mut() {
        t.0 = if live.connected {
            live.name.clone()
        } else {
            "Connect a radio or gamepad to see it here".to_string()
        };
    }
}

/// Live bars/values, sensitivity and calibration status on the Controller page.
#[allow(clippy::too_many_arguments)]
fn update_live_panel(
    live: Res<LiveInput>,
    cal: Res<CalibrationState>,
    mapping: Res<InputMapping>,
    mut params: ParamSet<(
        Query<(&mut Text, &LiveAxisText)>,
        Query<(&mut Node, &LiveAxisFill)>,
        Query<&mut Text, With<LiveButtonsText>>,
        Query<&mut Text, With<CalibStatusText>>,
        Query<&mut Text, With<MapSensText>>,
    )>,
) {
    let vals = [live.roll, live.pitch, live.yaw, live.throttle];
    for (mut t, marker) in params.p0().iter_mut() {
        t.0 = format!("{:+.2}", vals[marker.0]);
    }
    for (mut node, marker) in params.p1().iter_mut() {
        let (left, width) = fill_pct(marker.0, vals[marker.0]);
        node.left = Val::Percent(left);
        node.width = Val::Percent(width);
    }
    for mut t in params.p2().iter_mut() {
        t.0 = if live.buttons.is_empty() {
            "Buttons: none".to_string()
        } else {
            format!("Buttons: {}", live.buttons.join(", "))
        };
    }
    for mut t in params.p3().iter_mut() {
        t.0 = calibration_status(&cal);
    }
    for mut t in params.p4().iter_mut() {
        t.0 = format!("{:.1}", mapping.sensitivity);
    }
}

/// Live texts on the Home page.
fn update_home_texts(
    live: Res<LiveInput>,
    settings: Res<GameSettings>,
    stats: Res<FlightStats>,
    mut params: ParamSet<(
        Query<&mut Text, With<HomeLiveText>>,
        Query<&mut Text, With<HomeStatsText>>,
    )>,
) {
    for mut t in params.p0().iter_mut() {
        t.0 = if live.connected {
            live.name.clone()
        } else {
            settings.selected_controller.clone()
        };
    }
    for mut t in params.p1().iter_mut() {
        t.0 = home_stats_value(&stats);
    }
}

// ---- mapping controls ----

fn mapping_controls(
    mut mapping: ResMut<InputMapping>,
    q_cycle: Query<(&Interaction, &MapCycleAxis), Changed<Interaction>>,
    q_toggle: Query<(&Interaction, &MapToggleReverse), Changed<Interaction>>,
    q_sens_down: Query<&Interaction, (Changed<Interaction>, With<SensDown>)>,
    q_sens_up: Query<&Interaction, (Changed<Interaction>, With<SensUp>)>,
    mut chips: Query<(&MapAxisChip, &mut Text)>,
) {
    for (interaction, marker) in q_cycle.iter() {
        if *interaction == Interaction::Pressed {
            match marker.0 {
                0 => mapping.roll = mapping.roll.next(),
                1 => mapping.pitch = mapping.pitch.next(),
                2 => mapping.yaw = mapping.yaw.next(),
                3 => mapping.throttle = mapping.throttle.next(),
                _ => {}
            }
        }
    }
    for (interaction, marker) in q_toggle.iter() {
        if *interaction == Interaction::Pressed {
            match marker.0 {
                0 => mapping.roll_reversed = !mapping.roll_reversed,
                1 => mapping.pitch_reversed = !mapping.pitch_reversed,
                2 => mapping.yaw_reversed = !mapping.yaw_reversed,
                3 => mapping.throttle_reversed = !mapping.throttle_reversed,
                _ => {}
            }
        }
    }
    for interaction in q_sens_down.iter() {
        if *interaction == Interaction::Pressed {
            mapping.sensitivity = (mapping.sensitivity - 0.1).max(0.1);
        }
    }
    for interaction in q_sens_up.iter() {
        if *interaction == Interaction::Pressed {
            mapping.sensitivity = (mapping.sensitivity + 0.1).min(3.0);
        }
    }

    let axes = [mapping.roll, mapping.pitch, mapping.yaw, mapping.throttle];
    let revs = [
        mapping.roll_reversed,
        mapping.pitch_reversed,
        mapping.yaw_reversed,
        mapping.throttle_reversed,
    ];
    for (idx, mut t) in chips.iter_mut() {
        if idx.0 < 4 {
            t.0 = axis_label(axes[idx.0], revs[idx.0]);
        }
    }
}

// ---- nav / buttons ----

fn quit_button(q: Query<&Interaction, (Changed<Interaction>, With<QuitBtn>)>) {
    for interaction in q.iter() {
        if *interaction == Interaction::Pressed {
            std::process::exit(0);
        }
    }
}

fn play_button(mut next: ResMut<NextState<AppState>>, q: Query<&Interaction, (Changed<Interaction>, With<PlayBtn>)>) {
    for interaction in q.iter() {
        if *interaction == Interaction::Pressed {
            next.set(AppState::Play);
        }
    }
}

fn nav_buttons(mut next: ResMut<NextState<AppState>>, q: Query<(&Interaction, &NavBtn), Changed<Interaction>>) {
    for (interaction, nav) in q.iter() {
        if *interaction == Interaction::Pressed {
            next.set(nav.0);
        }
    }
}

// ---- scrolling ----

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
        let max = if delta.x > 0.0 {
            scroll_position.x >= max_offset.x
        } else {
            scroll_position.x <= 0.0
        };
        if !max {
            scroll_position.x += delta.x;
            delta.x = 0.0;
        }
    }
    if node.overflow.y == OverflowAxis::Scroll && delta.y != 0.0 {
        let max = if delta.y > 0.0 {
            scroll_position.y >= max_offset.y
        } else {
            scroll_position.y <= 0.0
        };
        if !max {
            scroll_position.y += delta.y;
            delta.y = 0.0;
        }
    }
    if *delta == Vec2::ZERO {
        scroll.propagate(false);
    }
}
