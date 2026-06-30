use bevy::prelude::*;

use crate::components::{
    BuyQuantityButton, GoldDisplay, ParagonButton, ParagonKind, ParagonLabel, RateDisplay,
    RebirthButton, RebirthButtonLabel, RebirthDisplay, ShopButton, ShopLabel, UpgradeButton,
    UpgradeKind, UpgradeLabel,
};
use crate::resources::{fmt_gold, BuyQuantity, GoldPool, RebirthState, ShopState, UpgradeState};
use crate::systems::shape_name;

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
            // Gold total
            parent.spawn((
                Text::new("💰 Gold: 10.0"),
                TextFont {
                    font_size: 22.0,
                    ..default()
                },
                TextColor(Color::srgb(1.00, 0.84, 0.00)),
                GoldDisplay,
            ));
            // Gold rate
            parent.spawn((
                Text::new("+0.00 / sec"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.90, 0.75, 0.40)),
                RateDisplay,
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
            // Buy quantity selector row
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(4.0),
                    margin: UiRect::top(Val::Px(4.0)),
                    ..default()
                })
                .with_children(|row| {
                    for qty in BuyQuantity::ALL {
                        spawn_buy_quantity_button(row, qty, qty == BuyQuantity::One);
                    }
                });
        });

    // Bottom panel: three rows — shop on top, session upgrades middle, rebirth & paragon bottom.
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
            // ── Row 1: shop system ────────────────────────────────────────
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceEvenly,
                    align_items: AlignItems::Center,
                    height: Val::Px(65.0),
                    ..default()
                })
                .with_children(|row| {
                    for i in 0..3 {
                        spawn_shop_button(row, i);
                    }
                });
            
            // ── Row 2: session upgrades ───────────────────────────────────
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
                        "💰 Gold Rate\nLv 0 | x1.0→x1.5\nCost: 50.0💰",
                    );
                    spawn_upgrade_button(
                        row,
                        UpgradeKind::ClickGold,
                        "👆 Per Click\nLv 0 | 0→5/click\nCost: 25.0💰",
                    );
                });

            // ── Row 3: rebirth + paragon ──────────────────────────────────
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceEvenly,
                    align_items: AlignItems::Center,
                    height: Val::Px(75.0),
                    ..default()
                })
                .with_children(|row| {
                    spawn_rebirth_button(row);
                    spawn_paragon_button(row, ParagonKind::AuraBoost, "🌟 Paragon\nGold Boost");
                });
        });
}

// ── Buy quantity button ───────────────────────────────────────────────────────

const BUY_QTY_BG_ACTIVE: Color     = Color::srgb(0.40, 0.20, 0.60);
const BUY_QTY_BG_INACTIVE: Color   = Color::srgb(0.15, 0.10, 0.25);
const BUY_QTY_BORDER_ACTIVE: Color   = Color::srgb(1.00, 0.80, 1.00);
const BUY_QTY_BORDER_INACTIVE: Color = Color::srgb(0.60, 0.40, 1.00);

fn spawn_buy_quantity_button(parent: &mut ChildBuilder, qty: BuyQuantity, active: bool) {
    let bg     = if active { BUY_QTY_BG_ACTIVE }     else { BUY_QTY_BG_INACTIVE };
    let border = if active { BUY_QTY_BORDER_ACTIVE } else { BUY_QTY_BORDER_INACTIVE };
    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(52.0),
                height: Val::Px(26.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(1.5)),
                ..default()
            },
            BackgroundColor(bg),
            BorderColor(border),
            BuyQuantityButton(qty),
        ))
        .with_children(|button| {
            button.spawn((
                Text::new(qty.label()),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

/// Highlight the currently active buy-quantity button.
pub fn update_buy_quantity_ui(
    buy_qty: Res<BuyQuantity>,
    mut buttons_q: Query<(&mut BackgroundColor, &mut BorderColor, &BuyQuantityButton)>,
) {
    if !buy_qty.is_changed() {
        return;
    }
    for (mut bg, mut border, btn) in buttons_q.iter_mut() {
        if btn.0 == *buy_qty {
            *bg     = BackgroundColor(BUY_QTY_BG_ACTIVE);
            *border = BorderColor(BUY_QTY_BORDER_ACTIVE);
        } else {
            *bg     = BackgroundColor(BUY_QTY_BG_INACTIVE);
            *border = BorderColor(BUY_QTY_BORDER_INACTIVE);
        }
    }
}

// ── Shop button ───────────────────────────────────────────────────────────────

fn spawn_shop_button(parent: &mut ChildBuilder, slot: usize) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(140.0),
                height: Val::Px(60.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.15, 0.10, 0.25)),
            BorderColor(Color::srgb(0.60, 0.40, 1.00)),
            ShopButton(slot),
        ))
        .with_children(|button| {
            button.spawn((
                Text::new("Buy Circle\n10💰"),
                TextFont {
                    font_size: 13.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                ShopLabel(slot),
            ));
        });
}

// ── Upgrade button ────────────────────────────────────────────────────────────

fn spawn_upgrade_button(parent: &mut ChildBuilder, kind: UpgradeKind, initial_text: &str) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(200.0),
                height: Val::Px(70.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.15, 0.10, 0.25)),
            BorderColor(Color::srgb(0.60, 0.40, 1.00)),
            UpgradeButton(kind),
        ))
        .with_children(|button| {
            button.spawn((
                Text::new(initial_text),
                TextFont {
                    font_size: 13.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                TextLayout::new_with_justify(JustifyText::Center),
                UpgradeLabel(kind),
            ));
        });
}

// ── Rebirth button ────────────────────────────────────────────────────────────

fn spawn_rebirth_button(parent: &mut ChildBuilder) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(200.0),
                height: Val::Px(70.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.20, 0.08, 0.10)),
            BorderColor(Color::srgb(1.00, 0.30, 0.40)),
            RebirthButton,
        ))
        .with_children(|button| {
            button.spawn((
                Text::new("🔄 Rebirth\nLevel 0\nEarn +0 PP"),
                TextFont {
                    font_size: 13.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                TextLayout::new_with_justify(JustifyText::Center),
                RebirthButtonLabel,
            ));
        });
}

// ── Paragon button ────────────────────────────────────────────────────────────

fn spawn_paragon_button(parent: &mut ChildBuilder, kind: ParagonKind, initial_text: &str) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(200.0),
                height: Val::Px(70.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.12, 0.08, 0.18)),
            BorderColor(Color::srgb(1.00, 0.75, 0.20)),
            ParagonButton(kind),
        ))
        .with_children(|button| {
            button.spawn((
                Text::new(initial_text),
                TextFont {
                    font_size: 13.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                TextLayout::new_with_justify(JustifyText::Center),
                ParagonLabel(kind),
            ));
        });
}

// ── Update systems ────────────────────────────────────────────────────────────

/// Update the top HUD with current gold, rate, and rebirth info.
pub fn update_hud(
    gold: Res<GoldPool>,
    rebirth: Res<RebirthState>,
    mut gold_q: Query<&mut Text, (With<GoldDisplay>, Without<RateDisplay>, Without<RebirthDisplay>)>,
    mut rate_q: Query<&mut Text, (With<RateDisplay>, Without<GoldDisplay>, Without<RebirthDisplay>)>,
    mut rebirth_q: Query<&mut Text, (With<RebirthDisplay>, Without<GoldDisplay>, Without<RateDisplay>)>,
) {
    for mut text in gold_q.iter_mut() {
        **text = format!("💰 Gold: {}", fmt_gold(gold.total));
    }
    for mut text in rate_q.iter_mut() {
        **text = format!("+{} / sec", fmt_gold(gold.rate));
    }
    for mut text in rebirth_q.iter_mut() {
        let perm_mult = rebirth.rebirth_gold_multiplier();
        **text = format!(
            "🔄 Rebirth: {} | x{:.1} perm  |  PP: {}",
            rebirth.rebirth_count, perm_mult, rebirth.paragon_points
        );
    }
}

/// Update shop button labels with current available shapes and costs.
pub fn update_shop_ui(
    shop: Res<ShopState>,
    mut labels_q: Query<(&mut Text, &ShopLabel)>,
) {
    for (mut text, label) in labels_q.iter_mut() {
        let slot = label.0;
        if slot < shop.available_levels.len() {
            let level = shop.available_levels[slot];
            let cost = crate::systems::shape_shop_cost(level);
            **text = format!("Buy {}\n{}💰", shape_name(level), fmt_gold(cost));
        }
    }
}

/// Update the session-upgrade button labels (only gold rate now).
pub fn update_upgrade_ui(
    gold: Res<GoldPool>,
    upgrades: Res<UpgradeState>,
    mut labels_q: Query<(&mut Text, &mut BackgroundColor, &UpgradeLabel)>,
) {
    for (mut text, mut bg_color, label) in labels_q.iter_mut() {
        match label.0 {
            UpgradeKind::AuraMultiplier => {
                let lv = upgrades.aura_multi_level;
                let current_mult = upgrades.aura_multiplier();
                let next_mult = UpgradeState {
                    aura_multi_level: lv + 1,
                    ..default()
                }
                .aura_multiplier();
                let cost = upgrades.aura_multi_cost();
                let affordable = gold.total >= cost;
                *bg_color = if affordable {
                    BackgroundColor(Color::srgb(0.18, 0.13, 0.28))
                } else {
                    BackgroundColor(Color::srgb(0.15, 0.10, 0.25))
                };
                **text = format!(
                    "💰 Gold Rate\nLv {lv} | x{current_mult:.1}→x{next_mult:.1}\nCost: {:.1}💰",
                    cost
                );
            }
            UpgradeKind::TokenCapacity | UpgradeKind::MergeSpeed => {
                // Legacy upgrades - display as disabled
                *bg_color = BackgroundColor(Color::srgb(0.10, 0.08, 0.12));
                **text = "(Disabled)".to_string();
            }
            UpgradeKind::ClickGold => {
                let lv = upgrades.click_gold_level;
                let current = upgrades.click_gold_per_click();
                let next = upgrades.next_click_gold_per_click();
                let cost = upgrades.click_gold_cost();
                let affordable = gold.total >= cost;
                *bg_color = if affordable {
                    BackgroundColor(Color::srgb(0.18, 0.13, 0.28))
                } else {
                    BackgroundColor(Color::srgb(0.15, 0.10, 0.25))
                };
                **text = format!(
                    "👆 Per Click\nLv {lv} | {current:.0}→{next:.0}/click\nCost: {}💰",
                    fmt_gold(cost)
                );
            }
        }
    }
}

/// Update the Rebirth button label and color based on total earned gold and max level.
pub fn update_rebirth_ui(
    gold: Res<GoldPool>,
    rebirth: Res<RebirthState>,
    shapes_q: Query<&crate::components::ShapeLevel>,
    mut button_q: Query<(&mut BackgroundColor, &mut BorderColor), With<RebirthButton>>,
    mut label_q: Query<&mut Text, With<RebirthButtonLabel>>,
) {
    let max_level = shapes_q.iter().map(|sl| sl.0).max().unwrap_or(0);
    let pp_gain = crate::resources::RebirthState::pp_earned(max_level);
    let threshold = rebirth.rebirth_gold_requirement();
    let affordable = gold.total_gold_earned >= threshold;

    for (mut bg, mut border) in button_q.iter_mut() {
        if affordable {
            *bg = BackgroundColor(Color::srgb(0.30, 0.12, 0.15));
            *border = BorderColor(Color::srgb(1.00, 0.40, 0.50));
        } else {
            *bg = BackgroundColor(Color::srgb(0.20, 0.08, 0.10));
            *border = BorderColor(Color::srgb(1.00, 0.30, 0.40));
        }
    }

    for mut text in label_q.iter_mut() {
        if affordable {
            **text = format!(
                "🔄 Rebirth\nLevel {max_level}\nEarn +{pp_gain} PP"
            );
        } else {
            **text = format!(
                "🔄 Rebirth\nNeed: {}💰\n(Lifetime: {}💰)",
                fmt_gold(threshold),
                fmt_gold(gold.total_gold_earned)
            );
        }
    }
}

/// Update Paragon upgrade button labels.
pub fn update_paragon_ui(
    rebirth: Res<RebirthState>,
    mut labels_q: Query<(&mut Text, &mut BackgroundColor, &ParagonLabel)>,
) {
    for (mut text, mut bg_color, label) in labels_q.iter_mut() {
        match label.0 {
            ParagonKind::AuraBoost => {
                let lv = rebirth.paragon_aura_level;
                let current_mult = rebirth.paragon_aura_multiplier();
                let next_mult = crate::resources::RebirthState {
                    paragon_aura_level: lv + 1,
                    ..default()
                }
                .paragon_aura_multiplier();
                let cost = rebirth.paragon_aura_cost();
                let affordable = rebirth.paragon_points >= cost;
                *bg_color = if affordable {
                    BackgroundColor(Color::srgb(0.18, 0.12, 0.24))
                } else {
                    BackgroundColor(Color::srgb(0.12, 0.08, 0.18))
                };
                **text = format!(
                    "🌟 Paragon Gold\nLv {lv} | x{current_mult:.1}→x{next_mult:.1}\nCost: {cost} PP"
                );
            }
        }
    }
}

/// Visual feedback: highlight buttons when hovered.
pub fn button_interaction(
    mut interaction_q: Query<(&Interaction, &mut BorderColor), (Changed<Interaction>, With<Button>)>,
) {
    for (interaction, mut border) in interaction_q.iter_mut() {
        match interaction {
            Interaction::Hovered => {
                *border = BorderColor(Color::srgb(1.00, 1.00, 1.00));
            }
            Interaction::None => {
                *border = BorderColor(Color::srgb(0.60, 0.40, 1.00));
            }
            _ => {}
        }
    }
}
