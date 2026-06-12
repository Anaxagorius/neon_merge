use bevy::prelude::*;

use crate::components::{
    AuraDisplay, ParagonButton, ParagonKind, ParagonLabel, ParagonPointsDisplay, RateDisplay,
    RebirthButton, RebirthDisplay, TokenDisplay, UpgradeButton, UpgradeKind, UpgradeLabel,
};
use crate::resources::{fmt_aura, AuraPool, RebirthState, SpawnTokens, UpgradeState, REBIRTH_THRESHOLD};

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
            // Rebirth info
            parent.spawn((
                Text::new("🔄 Rebirth: 0 | x1.0 perm  |  PP: 0"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(1.00, 0.75, 0.20)),
                RebirthDisplay,
            ));
        });

    // Bottom panel: two rows — session upgrades on top, rebirth & paragon below.
    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(0.0),
            left: Val::Px(0.0),
            right: Val::Px(0.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Stretch,
            padding: UiRect::axes(Val::Px(6.0), Val::Px(6.0)),
            row_gap: Val::Px(6.0),
            ..default()
        })
        .insert(BackgroundColor(Color::srgba(0.03, 0.01, 0.12, 0.92)))
        .with_children(|parent| {
            // ── Row 1: session upgrades ───────────────────────────────────
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceEvenly,
                    align_items: AlignItems::Center,
                    height: Val::Px(75.0),
                    ..default()
                })
                .with_children(|row| {
                    spawn_upgrade_button(
                        row,
                        UpgradeKind::AuraMultiplier,
                        "⚡ Aura Rate\nLv 0 | x1.0→x1.5\nCost: 50.0⚡",
                    );
                    spawn_upgrade_button(
                        row,
                        UpgradeKind::TokenCapacity,
                        "📦 Token Cap\nLv 0 | 10→15\nCost: 25.0⚡",
                    );
                    spawn_upgrade_button(
                        row,
                        UpgradeKind::MergeSpeed,
                        "⏩ Merge Spd\nLv 0 | 0.35s→0.27s\nCost: 30.0⚡",
                    );
                });

            // ── Row 2: rebirth + paragon upgrades ────────────────────────
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceEvenly,
                    align_items: AlignItems::Center,
                    height: Val::Px(75.0),
                    ..default()
                })
                .with_children(|row| {
                    // Rebirth button
                    row.spawn((
                        Button,
                        Node {
                            width: Val::Px(148.0),
                            height: Val::Px(68.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.10, 0.04, 0.04)),
                        RebirthButton,
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new(format!(
                                "🔄 Rebirth\nNeeds Lv {REBIRTH_THRESHOLD}+\n+20% perm / run"
                            )),
                            TextFont {
                                font_size: 11.5,
                                ..default()
                            },
                            TextColor(Color::srgb(1.00, 0.75, 0.20)),
                            TextLayout::new_with_justify(JustifyText::Center),
                            // reuse ParagonPointsDisplay as the rebirth button label
                            ParagonPointsDisplay,
                        ));
                    });

                    // Paragon: Aura Boost button
                    spawn_paragon_button(
                        row,
                        ParagonKind::AuraBoost,
                        "⭐ PP: Aura\nLv 0 | x1.0→x1.3\nCost: 1 PP",
                    );

                    // Paragon: Token Regen button
                    spawn_paragon_button(
                        row,
                        ParagonKind::TokenRegen,
                        "⭐ PP: Regen\nLv 0 | +0.0→+0.2/s\nCost: 1 PP",
                    );
                });
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

fn spawn_paragon_button(parent: &mut ChildBuilder, kind: ParagonKind, initial_text: &str) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(148.0),
                height: Val::Px(68.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.06, 0.06, 0.14)),
            ParagonButton(kind),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(initial_text),
                TextFont {
                    font_size: 11.5,
                    ..default()
                },
                TextColor(Color::srgb(1.00, 0.90, 0.50)),
                TextLayout::new_with_justify(JustifyText::Center),
                ParagonLabel(kind),
            ));
        });
}

// ── HUD update ────────────────────────────────────────────────────────────────

pub fn update_hud(
    aura: Res<AuraPool>,
    tokens: Res<SpawnTokens>,
    rebirth: Res<RebirthState>,
    mut aura_q: Query<&mut Text, (With<AuraDisplay>, Without<RateDisplay>, Without<TokenDisplay>, Without<RebirthDisplay>)>,
    mut rate_q: Query<&mut Text, (With<RateDisplay>, Without<AuraDisplay>, Without<TokenDisplay>, Without<RebirthDisplay>)>,
    mut tok_q: Query<&mut Text, (With<TokenDisplay>, Without<AuraDisplay>, Without<RateDisplay>, Without<RebirthDisplay>)>,
    mut reb_q: Query<&mut Text, (With<RebirthDisplay>, Without<AuraDisplay>, Without<RateDisplay>, Without<TokenDisplay>)>,
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
    let perm_multi = rebirth.rebirth_aura_multiplier() * rebirth.paragon_aura_multiplier();
    for mut text in reb_q.iter_mut() {
        **text = format!(
            "🔄 Rebirth: {}  |  x{:.2} perm  |  PP: {}",
            rebirth.rebirth_count,
            perm_multi,
            rebirth.paragon_points,
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
                    upgrades.next_aura_multiplier(),
                    fmt_aura(cost)
                );
            }
            UpgradeKind::TokenCapacity => {
                let cost = upgrades.token_cap_cost();
                **text = format!(
                    "📦 Token Cap\nLv {} | {}→{}\n{}⚡",
                    upgrades.token_cap_level,
                    upgrades.token_capacity() as u32,
                    upgrades.next_token_capacity() as u32,
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

// ── Rebirth & Paragon panel update ────────────────────────────────────────────

/// Updates the Rebirth button colour and label to reflect availability.
pub fn update_rebirth_panel(
    rebirth: Res<RebirthState>,
    shapes_q: Query<&crate::components::ShapeLevel>,
    mut buttons_q: Query<&mut BackgroundColor, With<RebirthButton>>,
    mut labels_q: Query<&mut Text, With<ParagonPointsDisplay>>,
) {
    let max_level = shapes_q.iter().map(|sl| sl.0).max().unwrap_or(0);
    let available = max_level >= REBIRTH_THRESHOLD;
    let pp_preview = if available {
        RebirthState::pp_earned(max_level)
    } else {
        0
    };

    for mut bg in buttons_q.iter_mut() {
        *bg = if available {
            BackgroundColor(Color::srgb(0.30, 0.10, 0.05))
        } else {
            BackgroundColor(Color::srgb(0.10, 0.04, 0.04))
        };
    }
    for mut text in labels_q.iter_mut() {
        if available {
            **text = format!(
                "🔄 Rebirth\n+{} PP  (+{:.0}% perm)\nLv {}+ ✓",
                pp_preview,
                20.0 * (rebirth.rebirth_count + 1) as f32,
                REBIRTH_THRESHOLD,
            );
        } else {
            **text = format!(
                "🔄 Rebirth\nNeeds Lv {}+\n(you have Lv {})",
                REBIRTH_THRESHOLD,
                max_level,
            );
        }
    }
}

/// Updates the Paragon upgrade button labels and colours.
pub fn update_paragon_panel(
    rebirth: Res<RebirthState>,
    mut labels_q: Query<(&ParagonLabel, &mut Text)>,
    mut buttons_q: Query<(&ParagonButton, &mut BackgroundColor)>,
) {
    for (label, mut text) in labels_q.iter_mut() {
        match label.0 {
            ParagonKind::AuraBoost => {
                let cost = rebirth.paragon_aura_cost();
                **text = format!(
                    "⭐ PP: Aura\nLv {} | x{:.1}→x{:.1}\nCost: {} PP",
                    rebirth.paragon_aura_level,
                    rebirth.paragon_aura_multiplier(),
                    rebirth.next_paragon_aura_multiplier(),
                    cost,
                );
            }
            ParagonKind::TokenRegen => {
                let cost = rebirth.paragon_regen_cost();
                **text = format!(
                    "⭐ PP: Regen\nLv {} | +{:.1}→+{:.1}/s\nCost: {} PP",
                    rebirth.paragon_regen_level,
                    rebirth.paragon_regen_bonus(),
                    rebirth.next_paragon_regen_bonus(),
                    cost,
                );
            }
        }
    }
    for (btn, mut bg) in buttons_q.iter_mut() {
        let cost = match btn.0 {
            ParagonKind::AuraBoost => rebirth.paragon_aura_cost(),
            ParagonKind::TokenRegen => rebirth.paragon_regen_cost(),
        };
        *bg = if rebirth.paragon_points >= cost {
            BackgroundColor(Color::srgb(0.12, 0.10, 0.22))
        } else {
            BackgroundColor(Color::srgb(0.06, 0.06, 0.14))
        };
    }
}
