use bevy::prelude::*;

use crate::components::{AuraDisplay, RateDisplay, TokenDisplay};
use crate::resources::{AuraPool, SpawnTokens};

// ── HUD setup ─────────────────────────────────────────────────────────────────

pub fn setup_hud(mut commands: Commands) {
    // Root container anchored to the top of the screen
    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            top: Val::Px(0.0),
            left: Val::Px(0.0),
            right: Val::Px(0.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            padding: UiRect::axes(Val::Px(12.0), Val::Px(10.0)),
            row_gap: Val::Px(4.0),
            ..default()
        })
        .insert(BackgroundColor(Color::srgba(0.03, 0.01, 0.12, 0.85)))
        .with_children(|parent| {
            // Aura total
            parent.spawn((
                Text::new("⚡ Aura: 10.0"),
                TextFont {
                    font_size: 22.0,
                    ..default()
                },
                TextColor(Color::srgb(0.45, 0.85, 1.00)),
                AuraDisplay,
            ));
            // Aura rate
            parent.spawn((
                Text::new("+0.00 / sec"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.50, 0.70, 0.90)),
                RateDisplay,
            ));
            // Spawn tokens
            parent.spawn((
                Text::new("Spawn tokens: 5 / 10"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.70, 0.55, 1.00)),
                TokenDisplay,
            ));
        });

    // Hint bar at bottom
    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(0.0),
            left: Val::Px(0.0),
            right: Val::Px(0.0),
            justify_content: JustifyContent::Center,
            padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
            ..default()
        })
        .insert(BackgroundColor(Color::srgba(0.03, 0.01, 0.12, 0.85)))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Click empty cell to spawn  •  shapes auto-merge when adjacent"),
                TextFont {
                    font_size: 13.0,
                    ..default()
                },
                TextColor(Color::srgb(0.45, 0.45, 0.60)),
            ));
        });
}

// ── HUD update ────────────────────────────────────────────────────────────────

pub fn update_hud(
    aura: Res<AuraPool>,
    tokens: Res<SpawnTokens>,
    mut aura_q: Query<&mut Text, (With<AuraDisplay>, Without<RateDisplay>, Without<TokenDisplay>)>,
    mut rate_q: Query<&mut Text, (With<RateDisplay>, Without<AuraDisplay>, Without<TokenDisplay>)>,
    mut tok_q: Query<&mut Text, (With<TokenDisplay>, Without<AuraDisplay>, Without<RateDisplay>)>,
) {
    for mut text in aura_q.iter_mut() {
        **text = format!("⚡ Aura: {:.1}", aura.total);
    }
    for mut text in rate_q.iter_mut() {
        **text = format!("+{:.2} / sec", aura.rate);
    }
    for mut text in tok_q.iter_mut() {
        **text = format!(
            "Spawn tokens: {} / {}",
            tokens.current as u32,
            tokens.max as u32
        );
    }
}
