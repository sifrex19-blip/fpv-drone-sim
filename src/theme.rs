// Shared visual theme: colors, text/icon helpers, layout pieces.
// Matches the dark + purple launcher design.

use bevy::prelude::*;
use bevy::text::LetterSpacing;

// ---- palette ----
pub fn bg() -> Color {
    Color::srgb(0.055, 0.055, 0.075)
}
pub fn sidebar_bg() -> Color {
    Color::srgb(0.078, 0.078, 0.10)
}
pub fn card() -> Color {
    Color::srgb(0.088, 0.088, 0.113)
}
pub fn row() -> Color {
    Color::srgb(0.105, 0.105, 0.130)
}
pub fn row_alt() -> Color {
    Color::srgb(0.075, 0.075, 0.098)
}
pub fn accent() -> Color {
    Color::srgb(0.49, 0.36, 0.99)
}
pub fn accent_soft() -> Color {
    Color::srgb(0.30, 0.22, 0.62)
}
pub fn accent_dim() -> Color {
    Color::srgb(0.20, 0.16, 0.36)
}
pub fn green() -> Color {
    Color::srgb(0.25, 0.85, 0.45)
}
pub fn red() -> Color {
    Color::srgb(0.95, 0.30, 0.35)
}
pub fn text_dim() -> Color {
    Color::srgb(0.58, 0.59, 0.65)
}
pub fn border() -> Color {
    Color::srgb(0.22, 0.22, 0.30)
}
pub fn osd_bg() -> Color {
    Color::srgba(0.04, 0.04, 0.06, 0.72)
}
pub fn accent_hover() -> Color {
    Color::srgb(0.58, 0.45, 1.0)
}
pub fn row_hover() -> Color {
    Color::srgb(0.145, 0.145, 0.180)
}
pub fn red_hover() -> Color {
    Color::srgb(1.0, 0.42, 0.47)
}
pub fn blue() -> Color {
    Color::srgb(0.28, 0.46, 0.88)
}
pub fn none() -> Color {
    Color::NONE
}

/// Badge / accent color for a controller kind ("Radio", "Gamepad", ...).
pub fn kind_color(kind: &str) -> Color {
    match kind {
        "Radio" => accent(),
        "Gamepad" => blue(),
        _ => Color::srgb(0.26, 0.28, 0.34),
    }
}

// ---- hover tinting ----

/// Marks a UI button: background tints to `hover` while hovered/pressed.
#[derive(Component)]
pub struct HoverTint {
    pub base: Color,
    pub hover: Color,
}

pub fn hover_tints(
    mut q: Query<(&Interaction, &HoverTint, &mut BackgroundColor), Changed<Interaction>>,
) {
    for (interaction, tint, mut bg) in q.iter_mut() {
        bg.0 = match interaction {
            Interaction::None => tint.base,
            _ => tint.hover,
        };
    }
}

/// The full UI font (the Bevy default is a mono subset without · — ↓ ° etc.).
///
/// Phase machine, run in `PostUpdate` after Bevy registers newly loaded fonts:
/// 0 = kick off asset loads, 1 = wait for registration, 2 = family-based source
/// (real bold face), 3 = single-handle fallback (bold degraded to regular),
/// 4 = fonts never appeared — keep Bevy's default so text stays visible.
pub fn apply_ui_font(
    server: Res<AssetServer>,
    mut font_cx: ResMut<bevy::text::FontCx>,
    mut phase: Local<u8>,
    mut frames: Local<u32>,
    mut seen_at: Local<u32>,
    mut reg: Local<Option<Handle<Font>>>,
    mut bold: Local<Option<Handle<Font>>>,
    mut texts: Query<&mut TextFont>,
) {
    if !asset_exists("fonts/FiraSans-Regular.ttf") {
        return;
    }
    match *phase {
        0 => {
            *reg = Some(server.load("fonts/FiraSans-Regular.ttf"));
            *bold = if asset_exists("fonts/FiraSans-Bold.ttf") {
                Some(server.load("fonts/FiraSans-Bold.ttf"))
            } else {
                None
            };
            *phase = 1;
            *frames = 0;
            *seen_at = 0;
        }
        1 => {
            *frames += 1;
            let Some(reg_h) = reg.as_ref() else {
                *phase = 4;
                return;
            };
            let reg_loaded = server.is_loaded_with_dependencies(reg_h.id());
            let bold_ready = match bold.as_ref() {
                Some(b) => server.is_loaded_with_dependencies(b.id()),
                None => true,
            };
            let ready = reg_loaded && bold_ready;
            if ready && *seen_at == 0 {
                *seen_at = *frames;
            }
            // Check a couple of frames after both loads complete so Bevy's
            // registration pass (PostUpdate) has definitely run.
            if ready && *frames >= *seen_at + 2 {
                let names: Vec<String> = font_cx
                    .collection
                    .family_names()
                    .take(24)
                    .map(|s| s.to_string())
                    .collect();
                if font_cx.collection.family_id("Fira Sans").is_some() {
                    info!(
                        "UI-FONTS: family 'Fira Sans' resolved; families: {names:?}"
                    );
                    *phase = 2;
                } else {
                    warn!(
                        "UI-FONTS: family 'Fira Sans' NOT registered; \
                         falling back to single handle; families: {names:?}"
                    );
                    *phase = 3;
                }
            } else if !ready && *frames > 600 {
                warn!(
                    "UI-FONTS: font assets failed to load (reg_loaded={reg_loaded}); \
                     keeping default font"
                );
                *phase = 4;
            }
        }
        _ => {}
    }
    let source = match *phase {
        2 => bevy::text::FontSource::families("Fira Sans"),
        3 => match reg.as_ref() {
            Some(h) => bevy::text::FontSource::Handle(h.clone()),
            None => return,
        },
        _ => return,
    };
    for mut tf in texts.iter_mut() {
        if *phase == 3 && tf.weight != bevy::text::FontWeight::NORMAL {
            // A lone handle has no bold face to pick; degrade instead of blanking.
            tf.weight = bevy::text::FontWeight::NORMAL;
        }
        if tf.font != source {
            tf.font = source.clone();
        }
    }
}

// ---- helpers ----
pub fn text(s: &str, size: f32, color: Color) -> (Text, TextFont, TextColor) {
    (
        Text::new(s.to_string()),
        TextFont {
            font_size: bevy::text::FontSize::Px(size),
            ..default()
        },
        TextColor(color),
    )
}

/// Bold text (headlines, values, buttons).
pub fn text_bold(s: &str, size: f32, color: Color) -> (Text, TextFont, TextColor) {
    (
        Text::new(s.to_string()),
        TextFont {
            font_size: bevy::text::FontSize::Px(size),
            weight: bevy::text::FontWeight::BOLD,
            ..default()
        },
        TextColor(color),
    )
}

/// Uppercase micro-label with letter spacing — section headers, vendor groups.
pub fn label(s: &str, size: f32, color: Color) -> (Text, TextFont, TextColor, LetterSpacing) {
    (
        Text::new(s.to_uppercase()),
        TextFont {
            font_size: bevy::text::FontSize::Px(size),
            ..default()
        },
        TextColor(color),
        LetterSpacing::Px(1.2),
    )
}

// ---- depth ----

/// Soft drop shadow under cards.
pub fn card_shadow() -> BoxShadow {
    BoxShadow::new(
        Color::srgba(0.0, 0.0, 0.0, 0.32),
        Val::Px(0.0),
        Val::Px(6.0),
        Val::Px(0.0),
        Val::Px(16.0),
    )
}

/// Purple glow + drop shadow for primary buttons.
pub fn glow_shadow() -> BoxShadow {
    BoxShadow(vec![
        ShadowStyle {
            color: accent().with_alpha(0.38),
            x_offset: Val::Px(0.0),
            y_offset: Val::Px(8.0),
            spread_radius: Val::Px(2.0),
            blur_radius: Val::Px(26.0),
        },
        ShadowStyle {
            color: Color::srgba(0.0, 0.0, 0.0, 0.35),
            x_offset: Val::Px(0.0),
            y_offset: Val::Px(10.0),
            spread_radius: Val::Px(0.0),
            blur_radius: Val::Px(18.0),
        },
    ])
}

/// Subtle light hairline for card edges.
pub fn hairline() -> Color {
    Color::srgba(1.0, 1.0, 1.0, 0.06)
}

/// True when an asset file exists relative to the active asset root.
pub fn asset_exists(rel: &str) -> bool {
    let root = crate::asset_root();
    root.join(rel).is_dir() || root.join(rel).is_file()
}

/// Load a UI icon if present; otherwise a default (empty) handle.
pub fn icon(asset_server: &AssetServer, name: &str) -> ImageNode {
    let rel = format!("ui/{name}.png");
    if asset_exists(&rel) {
        ImageNode {
            image: asset_server.load(rel),
            ..default()
        }
    } else {
        default()
    }
}

/// Square node that holds an icon image.
pub fn icon_box(asset_server: &AssetServer, name: &str, size: f32) -> (Node, ImageNode) {
    (
        Node {
            width: Val::Px(size),
            height: Val::Px(size),
            ..default()
        },
        icon(asset_server, name),
    )
}

// ---- layout pieces ----

/// Page header: rounded icon tile + title + subtitle (as in the reference launcher).
pub fn page_header(
    parent: &mut ChildSpawnerCommands,
    asset_server: &AssetServer,
    icon_name: &str,
    title: &str,
    subtitle: &str,
) {
    parent
        .spawn((Node {
            column_gap: Val::Px(16.0),
            align_items: AlignItems::Center,
            ..default()
        },))
        .with_children(|h| {
            h.spawn((
                Node {
                    width: Val::Px(46.0),
                    height: Val::Px(46.0),
                    border_radius: BorderRadius::all(Val::Px(12.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(accent().with_alpha(0.14)),
                BoxShadow::new(
                    accent().with_alpha(0.18),
                    Val::Px(0.0),
                    Val::Px(4.0),
                    Val::Px(0.0),
                    Val::Px(14.0),
                ),
            ))
            .with_children(|t| {
                t.spawn(icon_box(asset_server, icon_name, 26.0));
            });
            h.spawn((Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(2.0),
                ..default()
            },))
            .with_children(|titles| {
                titles.spawn(text_bold(title, 30.0, Color::WHITE));
                titles.spawn(text(subtitle, 14.0, text_dim()));
            });
        });
}

/// Small pill badge (Radio / Gamepad / ...).
pub fn badge(parent: &mut ChildSpawnerCommands, label: &str, bg: Color) {
    parent
        .spawn((
            Node {
                padding: UiRect {
                    left: Val::Px(10.0),
                    right: Val::Px(10.0),
                    top: Val::Px(3.0),
                    bottom: Val::Px(3.0),
                },
                border_radius: BorderRadius::all(Val::Px(9.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(bg),
        ))
        .with_children(|b| {
            b.spawn(text(label, 11.0, Color::WHITE));
        });
}
