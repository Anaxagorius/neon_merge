use bevy::prelude::*;

use crate::components::{GridPos, LabelEntity, ShapeLabel, ShapeLevel};
use crate::resources::{
    grid_to_world, world_to_grid, AuraPool, Grid, MergeTimer, SpawnTokens, CELL_SIZE, GRID_COLS,
    GRID_ROWS,
};

// ── Shape catalogue ───────────────────────────────────────────────────────────

pub const MAX_LEVEL: u32 = 5;

/// Human-readable name for each shape level.
pub fn shape_name(level: u32) -> &'static str {
    match level {
        1 => "Spark",
        2 => "Crystal",
        3 => "Prism",
        4 => "Nova",
        5 => "Singularity",
        _ => "Unknown",
    }
}

/// Neon colour for each shape level.
pub fn shape_color(level: u32) -> Color {
    match level {
        1 => Color::srgb(0.10, 0.55, 1.00), // electric blue
        2 => Color::srgb(0.00, 0.95, 0.90), // cyan
        3 => Color::srgb(0.65, 0.10, 1.00), // deep purple
        4 => Color::srgb(1.00, 0.10, 0.65), // hot pink
        5 => Color::srgb(1.00, 0.82, 0.00), // gold
        _ => Color::WHITE,
    }
}

/// Aura generated per second by a shape of this level.
pub fn aura_rate(level: u32) -> f64 {
    match level {
        1 => 0.1,
        2 => 0.4,
        3 => 1.5,
        4 => 6.0,
        5 => 25.0,
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

pub fn regen_tokens(time: Res<Time>, mut tokens: ResMut<SpawnTokens>) {
    tokens.current = (tokens.current + tokens.regen_rate * time.delta_secs()).min(tokens.max);
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
    shapes_q: Query<&ShapeLevel>,
) {
    let dt = time.delta_secs_f64();
    let total_rate: f64 = shapes_q.iter().map(|sl| aura_rate(sl.0)).sum();
    aura.rate = total_rate;
    aura.total += total_rate * dt;
}
