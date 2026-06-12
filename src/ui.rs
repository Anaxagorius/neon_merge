use bevy::prelude::*;

use crate::components::{AuraDisplay, RateDisplay, TokenDisplay, UpgradeButton, UpgradeKind, UpgradeLabel};
use crate::resources::{fmt_aura, AuraPool, SpawnTokens, UpgradeState};

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

    // Upgrade panel at bottom
    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(0.0),
            left: Val::Px(0.0),
            right: Val::Px(0.0),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceEvenly,
            align_items: AlignItems::Center,
            padding: UiRect::axes(Val::Px(6.0), Val::Px(8.0)),
            height: Val::Px(95.0),
            ..default()
        })
        .insert(BackgroundColor(Color::srgba(0.03, 0.01, 0.12, 0.92)))
        .with_children(|parent| {
            spawn_upgrade_button(
                parent,
                UpgradeKind::AuraMultiplier,
                "⚡ Aura Rate\nLv 0 | x1.0→x1.5\nCost: 50.0⚡",
            );
            spawn_upgrade_button(
                parent,
                UpgradeKind::TokenCapacity,
                "📦 Token Cap\nLv 0 | 10→15\nCost: 25.0⚡",
            );
            spawn_upgrade_button(
                parent,
                UpgradeKind::MergeSpeed,
                "⏩ Merge Spd\nLv 0 | 0.35s→0.27s\nCost: 30.0⚡",
            );
        });
}

fn spawn_upgrade_button(parent: &mut ChildBuilder, kind: UpgradeKind, initial_text: &str) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(148.0),
                height: Val::Px(75.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.07, 0.04, 0.16)),
            UpgradeButton(kind),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(initial_text),
                TextFont {
                    font_size: 11.5,
                    ..default()
                },
                TextColor(Color::srgb(0.85, 0.85, 1.00)),
                TextLayout::new_with_justify(JustifyText::Center),
                UpgradeLabel(kind),
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
        **text = format!("⚡ Aura: {}", fmt_aura(aura.total));
    }
    for mut text in rate_q.iter_mut() {
        **text = format!("+{}/s", fmt_aura(aura.rate));
    }
    for mut text in tok_q.iter_mut() {
        **text = format!(
            "Spawn tokens: {} / {}",
            tokens.current as u32,
            tokens.max as u32
        );
    }
}

// ── Upgrade panel update ──────────────────────────────────────────────────────

pub fn update_upgrade_panel(
    upgrades: Res<UpgradeState>,
    aura: Res<AuraPool>,
    mut labels_q: Query<(&UpgradeLabel, &mut Text)>,
    mut buttons_q: Query<(&UpgradeButton, &mut BackgroundColor)>,
) {
    for (label, mut text) in labels_q.iter_mut() {
        match label.0 {
            UpgradeKind::AuraMultiplier => {
                let cost = upgrades.aura_multi_cost();
                **text = format!(
                    "⚡ Aura Rate\nLv {} | x{:.1}→x{:.1}\n{}⚡",
                    upgrades.aura_multi_level,
                    upgrades.aura_multiplier(),
                    upgrades.aura_multiplier() + 0.5,
                    fmt_aura(cost)
                );
            }
            UpgradeKind::TokenCapacity => {
                let cost = upgrades.token_cap_cost();
                **text = format!(
                    "📦 Token Cap\nLv {} | {}→{}\n{}⚡",
                    upgrades.token_cap_level,
                    upgrades.token_capacity() as u32,
                    upgrades.token_capacity() as u32 + 5,
                    fmt_aura(cost)
                );
            }
            UpgradeKind::MergeSpeed => {
                let cost = upgrades.merge_speed_cost();
                **text = format!(
                    "⏩ Merge Spd\nLv {} | {:.2}s→{:.2}s\n{}⚡",
                    upgrades.merge_speed_level,
                    upgrades.merge_interval(),
                    UpgradeState::merge_interval_at(upgrades.merge_speed_level + 1),
                    fmt_aura(cost)
                );
            }
        }
    }
    for (btn, mut bg) in buttons_q.iter_mut() {
        let cost = match btn.0 {
            UpgradeKind::AuraMultiplier => upgrades.aura_multi_cost(),
            UpgradeKind::TokenCapacity => upgrades.token_cap_cost(),
            UpgradeKind::MergeSpeed => upgrades.merge_speed_cost(),
        };
        *bg = if aura.total >= cost {
            BackgroundColor(Color::srgb(0.15, 0.08, 0.30))
        } else {
            BackgroundColor(Color::srgb(0.07, 0.04, 0.16))
        };
    }
}
