use bevy::prelude::*;

use crate::components::{GridPos, LabelEntity, ParagonButton, ParagonKind, RebirthButton, ShapeLabel, ShapeLevel, UpgradeButton, UpgradeKind};
use crate::resources::{
    grid_to_world, world_to_grid, AuraPool, Grid, MergeTimer, RebirthState, SpawnTokens,
    UpgradeState, CELL_SIZE, GRID_COLS, GRID_ROWS, REBIRTH_THRESHOLD,
};

// ── Shape catalogue ───────────────────────────────────────────────────────────

pub const MAX_LEVEL: u32 = 25;

/// Human-readable name for each shape level (1–25).
pub fn shape_name(level: u32) -> &'static str {
    match level {
        1  => "Spark",
        2  => "Crystal",
        3  => "Prism",
        4  => "Nova",
        5  => "Singularity",
        6  => "Nebula",
        7  => "Pulsar",
        8  => "Quasar",
        9  => "Vortex",
        10 => "Zenith",
        11 => "Rift",
        12 => "Eclipse",
        13 => "Aurora",
        14 => "Inferno",
        15 => "Cascade",
        16 => "Apex",
        17 => "Phantom",
        18 => "Specter",
        19 => "Radiance",
        20 => "Celestial",
        21 => "Ethereal",
        22 => "Transcend",
        23 => "Infinity",
        24 => "Nexus",
        25 => "Void",
        _  => "Unknown",
    }
}

/// Neon colour for each shape level (1–25).
pub fn shape_color(level: u32) -> Color {
    match level {
        1  => Color::srgb(0.10, 0.55, 1.00), // electric blue
        2  => Color::srgb(0.00, 0.95, 0.90), // cyan
        3  => Color::srgb(0.65, 0.10, 1.00), // deep purple
        4  => Color::srgb(1.00, 0.10, 0.65), // hot pink
        5  => Color::srgb(1.00, 0.82, 0.00), // gold
        6  => Color::srgb(0.20, 1.00, 0.40), // neon green
        7  => Color::srgb(1.00, 0.45, 0.00), // neon orange
        8  => Color::srgb(0.50, 0.00, 1.00), // violet
        9  => Color::srgb(0.00, 0.70, 1.00), // sky blue
        10 => Color::srgb(1.00, 1.00, 0.30), // bright yellow
        11 => Color::srgb(1.00, 0.20, 0.20), // red-orange
        12 => Color::srgb(0.30, 1.00, 0.80), // aquamarine
        13 => Color::srgb(0.80, 0.00, 0.80), // magenta
        14 => Color::srgb(0.60, 1.00, 0.00), // chartreuse
        15 => Color::srgb(0.00, 0.40, 1.00), // royal blue
        16 => Color::srgb(1.00, 0.60, 0.80), // light pink
        17 => Color::srgb(0.40, 0.80, 0.00), // olive green
        18 => Color::srgb(1.00, 0.30, 0.30), // coral
        19 => Color::srgb(0.20, 0.80, 1.00), // ice blue
        20 => Color::srgb(0.90, 0.50, 1.00), // lavender
        21 => Color::srgb(1.00, 0.70, 0.10), // amber
        22 => Color::srgb(0.00, 1.00, 0.60), // mint
        23 => Color::srgb(0.70, 0.20, 1.00), // indigo
        24 => Color::srgb(1.00, 0.95, 0.70), // warm white
        25 => Color::srgb(1.00, 1.00, 1.00), // pure white (Void)
        _  => Color::WHITE,
    }
}

/// Aura generated per second by a shape of this level.
/// Levels 1–5 are tuned by hand; 6–25 follow a ×4 exponential curve.
/// Levels above MAX_LEVEL are unreachable in normal gameplay but return 0.0.
pub fn aura_rate(level: u32) -> f64 {
    match level {
        1 => 0.1,
        2 => 0.4,
        3 => 1.5,
        4 => 6.0,
        5 => 25.0,
        6..=MAX_LEVEL => 0.1 * 4.0f64.powi(level as i32 - 1),
        _ => 0.0,
    }
}

// ── Setup: camera & grid background ──────────────────────────────────────────

pub fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

/// Spawn background tiles for every grid cell.
pub fn setup_grid(mut commands: Commands) {
    let bg_color = Color::srgb(0.06, 0.03, 0.14);
    for col in 0..GRID_COLS {
        for row in 0..GRID_ROWS {
            let pos = grid_to_world(col, row);
            commands.spawn((
                Sprite {
                    color: bg_color,
                    custom_size: Some(Vec2::splat(CELL_SIZE)),
                    ..default()
                },
                Transform::from_xyz(pos.x, pos.y, 0.0),
            ));
        }
    }
}

// ── Spawn a shape entity ──────────────────────────────────────────────────────

/// Spawns a shape at `(col, row)` and registers it in `grid`.
/// The level-number label is a separate entity whose ID is stored on the
/// shape via [`LabelEntity`] so it can be despawned in O(1).
pub fn spawn_shape(
    commands: &mut Commands,
    grid: &mut Grid,
    col: i32,
    row: i32,
    level: u32,
) -> Entity {
    let world = grid_to_world(col, row);

    // Spawn the floating label first so we know its entity ID.
    let label_entity = commands
        .spawn((
            Text2d::new(level.to_string()),
            TextFont {
                font_size: 26.0,
                ..default()
            },
            TextColor(Color::WHITE),
            Transform::from_xyz(world.x, world.y, 2.0),
            ShapeLabel,
        ))
        .id();

    // Spawn the shape sprite, storing a reference to its label.
    let shape_entity = commands
        .spawn((
            Sprite {
                color: shape_color(level),
                custom_size: Some(Vec2::splat(CELL_SIZE - 6.0)),
                ..default()
            },
            Transform::from_xyz(world.x, world.y, 1.0),
            GridPos { col, row },
            ShapeLevel(level),
            LabelEntity(label_entity),
        ))
        .id();

    grid.insert(col, row, shape_entity);
    shape_entity
}

// ── Input: click to spawn ─────────────────────────────────────────────────────

pub fn handle_input(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    mut grid: ResMut<Grid>,
    mut tokens: ResMut<SpawnTokens>,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let Ok(window) = windows.get_single() else {
        return;
    };
    let Ok((camera, cam_xf)) = camera_q.get_single() else {
        return;
    };
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    let Ok(world_pos) = camera.viewport_to_world_2d(cam_xf, cursor) else {
        return;
    };
    let Some((col, row)) = world_to_grid(world_pos) else {
        return;
    };

    if !grid.is_empty(col, row) {
        return; // cell already occupied
    }
    if tokens.current < 1.0 {
        return; // no tokens available
    }

    tokens.current -= 1.0;
    spawn_shape(&mut commands, &mut grid, col, row, 1);
}

// ── Token regeneration ────────────────────────────────────────────────────────

pub fn regen_tokens(
    time: Res<Time>,
    mut tokens: ResMut<SpawnTokens>,
    rebirth: Res<RebirthState>,
) {
    let rate = tokens.regen_rate + rebirth.paragon_regen_bonus();
    tokens.current = (tokens.current + rate * time.delta_secs()).min(tokens.max);
}

// ── Auto-merge adjacent same-level pairs ──────────────────────────────────────

pub fn auto_merge(
    mut commands: Commands,
    mut grid: ResMut<Grid>,
    time: Res<Time>,
    mut merge_timer: ResMut<MergeTimer>,
    shapes_q: Query<(&ShapeLevel, &LabelEntity)>,
) {
    merge_timer.0.tick(time.delta());
    if !merge_timer.0.just_finished() {
        return;
    }

    // Collect occupied cells with level and label ID.
    let occupied: Vec<((i32, i32), Entity, u32, Entity)> = grid
        .cells
        .iter()
        .filter_map(|(&pos, &entity)| {
            shapes_q
                .get(entity)
                .ok()
                .map(|(sl, le)| (pos, entity, sl.0, le.0))
        })
        .collect();

    // Find the first mergeable pair: adjacent cells with the same level < MAX.
    for &((col_a, row_a), entity_a, level_a, label_a) in &occupied {
        if level_a >= MAX_LEVEL {
            continue;
        }
        for (col_b, row_b) in Grid::neighbours(col_a, row_a) {
            if let Some(&entity_b) = grid.cells.get(&(col_b, row_b)) {
                let Ok((sl_b, label_b_ref)) = shapes_q.get(entity_b) else {
                    continue;
                };
                if sl_b.0 != level_a {
                    continue;
                }

                let new_level = level_a + 1;

                // Despawn entity_b and its label (O(1) — ID stored directly).
                commands.entity(label_b_ref.0).despawn();
                commands.entity(entity_b).despawn();
                grid.remove(col_b, row_b);

                // Despawn the old label for entity_a.
                commands.entity(label_a).despawn();

                // Spawn a new label for the upgraded level.
                let world = grid_to_world(col_a, row_a);
                let new_label = commands
                    .spawn((
                        Text2d::new(new_level.to_string()),
                        TextFont {
                            font_size: 26.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        Transform::from_xyz(world.x, world.y, 2.0),
                        ShapeLabel,
                    ))
                    .id();

                // Upgrade entity_a in place.
                commands.entity(entity_a).insert((
                    ShapeLevel(new_level),
                    Sprite {
                        color: shape_color(new_level),
                        custom_size: Some(Vec2::splat(CELL_SIZE - 6.0)),
                        ..default()
                    },
                    LabelEntity(new_label),
                ));

                return; // one merge per tick
            }
        }
    }
}

// ── Aura generation ───────────────────────────────────────────────────────────

pub fn generate_aura(
    time: Res<Time>,
    mut aura: ResMut<AuraPool>,
    upgrades: Res<UpgradeState>,
    rebirth: Res<RebirthState>,
    shapes_q: Query<&ShapeLevel>,
) {
    let dt = time.delta_secs_f64();
    let base_rate: f64 = shapes_q.iter().map(|sl| aura_rate(sl.0)).sum();
    let total_rate = base_rate
        * upgrades.aura_multiplier()
        * rebirth.rebirth_aura_multiplier()
        * rebirth.paragon_aura_multiplier();
    aura.rate = total_rate;
    aura.total += total_rate * dt;
}

// ── Upgrade purchases ─────────────────────────────────────────────────────────

pub fn handle_upgrades(
    interaction_q: Query<(&Interaction, &UpgradeButton), Changed<Interaction>>,
    mut aura: ResMut<AuraPool>,
    mut upgrades: ResMut<UpgradeState>,
    mut tokens: ResMut<SpawnTokens>,
    mut merge_timer: ResMut<MergeTimer>,
) {
    for (interaction, btn) in interaction_q.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match btn.0 {
            UpgradeKind::AuraMultiplier => {
                let cost = upgrades.aura_multi_cost();
                if aura.total >= cost {
                    aura.total -= cost;
                    upgrades.aura_multi_level += 1;
                }
            }
            UpgradeKind::TokenCapacity => {
                let cost = upgrades.token_cap_cost();
                if aura.total >= cost {
                    aura.total -= cost;
                    upgrades.token_cap_level += 1;
                    tokens.max = upgrades.token_capacity();
                }
            }
            UpgradeKind::MergeSpeed => {
                let cost = upgrades.merge_speed_cost();
                if aura.total >= cost {
                    aura.total -= cost;
                    upgrades.merge_speed_level += 1;
                    merge_timer.0 = Timer::from_seconds(
                        upgrades.merge_interval(),
                        TimerMode::Repeating,
                    );
                }
            }
        }
    }
}

// ── Rebirth ───────────────────────────────────────────────────────────────────

/// Triggered when the player presses the Rebirth button.
///
/// Requirements: at least one shape on the grid must be at level ≥
/// `REBIRTH_THRESHOLD`.  On success the grid is cleared, session resources are
/// reset, and the rebirth count / Paragon Points are updated.
pub fn handle_rebirth(
    mut commands: Commands,
    interaction_q: Query<&Interaction, (Changed<Interaction>, With<RebirthButton>)>,
    mut grid: ResMut<Grid>,
    shapes_q: Query<(&ShapeLevel, &LabelEntity)>,
    mut aura: ResMut<AuraPool>,
    mut tokens: ResMut<SpawnTokens>,
    mut upgrades: ResMut<UpgradeState>,
    mut rebirth: ResMut<RebirthState>,
    mut merge_timer: ResMut<MergeTimer>,
) {
    if interaction_q.iter().all(|i| *i != Interaction::Pressed) {
        return;
    }

    // Determine the highest level shape currently on the grid.
    let max_level = shapes_q.iter().map(|(sl, _)| sl.0).max().unwrap_or(0);
    if max_level < REBIRTH_THRESHOLD {
        return; // threshold not met — button press is ignored
    }

    // Award Paragon Points and advance the rebirth counter.
    rebirth.paragon_points += RebirthState::pp_earned(max_level);
    rebirth.rebirth_count += 1;

    // Despawn every shape entity (and its floating label) then clear the grid.
    for &entity in grid.cells.values() {
        if let Ok((_, label)) = shapes_q.get(entity) {
            commands.entity(label.0).despawn();
        }
        commands.entity(entity).despawn();
    }
    grid.cells.clear();

    // Reset all session-scoped resources to their defaults.
    *aura = AuraPool::default();
    let reset_upgrades = UpgradeState::default();
    tokens.max = reset_upgrades.token_capacity();
    tokens.current = tokens.current.min(tokens.max);
    tokens.regen_rate = SpawnTokens::default().regen_rate;
    *upgrades = reset_upgrades;
    *merge_timer = MergeTimer::default();
}

// ── Paragon upgrades ──────────────────────────────────────────────────────────

pub fn handle_paragon_upgrades(
    interaction_q: Query<(&Interaction, &ParagonButton), Changed<Interaction>>,
    mut rebirth: ResMut<RebirthState>,
) {
    for (interaction, btn) in interaction_q.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match btn.0 {
            ParagonKind::AuraBoost => {
                let cost = rebirth.paragon_aura_cost();
                if rebirth.paragon_points >= cost {
                    rebirth.paragon_points -= cost;
                    rebirth.paragon_aura_level += 1;
                }
            }
            ParagonKind::TokenRegen => {
                let cost = rebirth.paragon_regen_cost();
                if rebirth.paragon_points >= cost {
                    rebirth.paragon_points -= cost;
                    rebirth.paragon_regen_level += 1;
                }
            }
        }
    }
}
