use bevy::core_pipeline::bloom::Bloom;
use bevy::prelude::*;
use bevy::render::mesh::Mesh2d;
use bevy::sprite::{ColorMaterial, MeshMaterial2d};
use bevy::math::primitives::{Circle as CirclePrimitive, RegularPolygon, Rectangle as RectanglePrimitive, Ellipse};

use crate::animations::{MergeFlash, SpawnPop, PulseEffect, GlowEffect};
use crate::components::{BuyQuantityButton, GridPos, LabelEntity, ParagonButton, ParagonKind, RebirthButton, ShapeLabel, ShapeLevel, UpgradeButton, UpgradeKind, ShopButton};
use crate::resources::{
    grid_to_world, world_to_grid, BuyQuantity, GoldPool, Grid, RebirthState, DragState, DragInfo, ShopState,
    UpgradeState, CELL_SIZE, GRID_COLS, GRID_ROWS,
};
use crate::save::{write_save, SaveData};

// ── Shape catalogue ───────────────────────────────────────────────────────────

pub const MAX_LEVEL: u32 = 50;

/// Human-readable name for each shape level (1–50).
pub fn shape_name(level: u32) -> &'static str {
    match level {
        // 2D shapes (Basic tier)
        1  => "Circle",
        2  => "Triangle",
        3  => "Square",
        4  => "Rectangle",
        5  => "Parallelogram",
        6  => "Trapezoid",
        7  => "Rhombus",
        8  => "Kite",
        9  => "Pentagon",
        10 => "Hexagon",
        11 => "Heptagon",
        12 => "Octagon",
        13 => "Nonagon",
        14 => "Decagon",
        15 => "Hendecagon",
        16 => "Dodecagon",
        17 => "Star",
        18 => "Ellipse",
        19 => "Sector",
        20 => "Semicircle",
        21 => "Crescent",
        22 => "IsoscelesTriangle",
        23 => "EquilateralTriangle",
        24 => "RightTriangle",
        25 => "ScaleneTriangle",
        // 3D shapes (Specialized tier)
        26 => "Sphere",
        27 => "Cube",
        28 => "Cuboid",
        29 => "Cylinder",
        30 => "Cone",
        31 => "PyramidSquare",
        32 => "Tetrahedron",
        33 => "Octahedron",
        34 => "Dodecahedron",
        35 => "Icosahedron",
        36 => "TriangularPrism",
        37 => "PentagonalPrism",
        38 => "HexagonalPrism",
        39 => "Frustum",
        40 => "Torus",
        41 => "Ellipsoid",
        42 => "Hemisphere",
        43 => "ObliqueCylinder",
        44 => "ObliqueCone",
        45 => "Bipyramid",
        // Prestige shapes
        46 => "ReuleauxTriangle",
        47 => "Superellipse",
        48 => "Lune",
        49 => "Annulus",
        50 => "CrossSection",
        _  => "Unknown",
    }
}

/// HDR brightness multiplier applied to all shape colours.
/// Values above 1.0 in linear space trigger Bloom glow on the HDR camera.
const HDR_BOOST: f32 = 2.5;
const HDR_BOOST_PRESTIGE: f32 = 4.0; // Extra boost for prestige shapes (46-50)

/// Neon colour for each shape level (1–50), expressed in linear HDR space
/// so they bloom under the HDR camera with Bloom post-processing.
pub fn shape_color(level: u32) -> Color {
    let (r, g, b, boost): (f32, f32, f32, f32) = match level {
        // 2D shapes (1-25)
        1  => (0.10, 0.55, 1.00, HDR_BOOST), // electric blue
        2  => (0.00, 0.95, 0.90, HDR_BOOST), // cyan
        3  => (0.65, 0.10, 1.00, HDR_BOOST), // deep purple
        4  => (1.00, 0.10, 0.65, HDR_BOOST), // hot pink
        5  => (1.00, 0.82, 0.00, HDR_BOOST), // gold
        6  => (0.20, 1.00, 0.40, HDR_BOOST), // neon green
        7  => (1.00, 0.45, 0.00, HDR_BOOST), // neon orange
        8  => (0.50, 0.00, 1.00, HDR_BOOST), // violet
        9  => (0.00, 0.70, 1.00, HDR_BOOST), // sky blue
        10 => (1.00, 1.00, 0.30, HDR_BOOST), // bright yellow
        11 => (1.00, 0.20, 0.20, HDR_BOOST), // red-orange
        12 => (0.30, 1.00, 0.80, HDR_BOOST), // aquamarine
        13 => (0.80, 0.00, 0.80, HDR_BOOST), // magenta
        14 => (0.60, 1.00, 0.00, HDR_BOOST), // chartreuse
        15 => (0.00, 0.40, 1.00, HDR_BOOST), // royal blue
        16 => (1.00, 0.60, 0.80, HDR_BOOST), // light pink
        17 => (0.40, 0.80, 0.00, HDR_BOOST), // olive green
        18 => (1.00, 0.30, 0.30, HDR_BOOST), // coral
        19 => (0.20, 0.80, 1.00, HDR_BOOST), // ice blue
        20 => (0.90, 0.50, 1.00, HDR_BOOST), // lavender
        21 => (1.00, 0.70, 0.10, HDR_BOOST), // amber
        22 => (0.00, 1.00, 0.60, HDR_BOOST), // mint
        23 => (0.70, 0.20, 1.00, HDR_BOOST), // indigo
        24 => (1.00, 0.95, 0.70, HDR_BOOST), // warm white
        25 => (1.00, 1.00, 1.00, HDR_BOOST), // pure white
        // 3D shapes (26-45) - intense saturated neons
        26 => (0.50, 0.10, 0.95, HDR_BOOST), // deep purple
        27 => (0.10, 0.60, 1.00, HDR_BOOST), // electric blue
        28 => (1.00, 0.15, 0.80, HDR_BOOST), // hot magenta
        29 => (1.00, 0.50, 0.05, HDR_BOOST), // hot orange
        30 => (0.95, 0.05, 0.40, HDR_BOOST), // deep red
        31 => (0.15, 0.95, 0.95, HDR_BOOST), // bright cyan
        32 => (0.85, 0.85, 0.10, HDR_BOOST), // bright yellow
        33 => (0.25, 1.00, 0.25, HDR_BOOST), // lime green
        34 => (0.95, 0.20, 1.00, HDR_BOOST), // bright purple
        35 => (0.10, 0.90, 0.60, HDR_BOOST), // teal
        36 => (1.00, 0.75, 0.20, HDR_BOOST), // gold
        37 => (0.60, 0.15, 1.00, HDR_BOOST), // violet
        38 => (1.00, 0.40, 0.60, HDR_BOOST), // rose
        39 => (0.40, 0.90, 1.00, HDR_BOOST), // light blue
        40 => (1.00, 1.00, 0.50, HDR_BOOST), // pale yellow
        41 => (0.80, 0.25, 0.90, HDR_BOOST), // purple
        42 => (0.20, 0.85, 0.85, HDR_BOOST), // cyan
        43 => (1.00, 0.60, 0.10, HDR_BOOST), // orange
        44 => (0.70, 0.90, 0.20, HDR_BOOST), // yellow-green
        45 => (0.90, 0.30, 0.70, HDR_BOOST), // pink
        // Prestige shapes (46-50) - bright green spectrum with extra HDR
        46 => (0.20, 1.00, 0.30, HDR_BOOST_PRESTIGE), // bright green
        47 => (0.50, 1.00, 0.20, HDR_BOOST_PRESTIGE), // acid green
        48 => (0.70, 1.00, 0.40, HDR_BOOST_PRESTIGE), // lime
        49 => (0.40, 1.00, 0.60, HDR_BOOST_PRESTIGE), // mint green
        50 => (0.80, 1.00, 0.80, HDR_BOOST_PRESTIGE), // white-green
        _  => (1.00, 1.00, 1.00, HDR_BOOST),
    };
    Color::linear_rgb(r * boost, g * boost, b * boost)
}

/// Gold generated per second by a shape of this level.
/// Levels 1–5 are tuned by hand; 6+ follow an exponential curve.
pub fn gold_rate(level: u32) -> f64 {
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

/// Cost in gold to purchase a shape of this level from the shop.
pub fn shape_shop_cost(level: u32) -> f64 {
    let base_cost = 10.0 * 3.0f64.powi(level as i32 - 1);
    // 3D shapes (26-45) cost 5x more
    if level >= 26 && level <= 45 {
        base_cost * 5.0
    } else {
        base_cost
    }
}

// ── Setup: camera & grid background ──────────────────────────────────────────

pub fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Camera {
            hdr: true,
            ..default()
        },
        Bloom {
            intensity: 0.25,
            ..default()
        },
    ));
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

/// Helper function to create a mesh for a given shape level.
fn shape_mesh(level: u32) -> Mesh {
    match level {
        1 => Mesh::from(CirclePrimitive { radius: 30.0 }),
        2 => Mesh::from(RegularPolygon::new(32.0, 3)),
        3 => Mesh::from(RegularPolygon::new(32.0, 4)),
        4 => Mesh::from(RectanglePrimitive { half_size: Vec2::new(36.0, 24.0) }),
        5 => Mesh::from(RegularPolygon::new(32.0, 4)), // Parallelogram approximation
        6 => Mesh::from(RegularPolygon::new(32.0, 4)), // Trapezoid approximation
        7 => Mesh::from(RegularPolygon::new(32.0, 4)), // Rhombus
        8 => Mesh::from(RegularPolygon::new(32.0, 4)), // Kite approximation
        9 => Mesh::from(RegularPolygon::new(30.0, 5)),
        10 => Mesh::from(RegularPolygon::new(30.0, 6)),
        11 => Mesh::from(RegularPolygon::new(30.0, 7)),
        12 => Mesh::from(RegularPolygon::new(30.0, 8)),
        13 => Mesh::from(RegularPolygon::new(30.0, 9)),
        14 => Mesh::from(RegularPolygon::new(30.0, 10)),
        15 => Mesh::from(RegularPolygon::new(30.0, 11)),
        16 => Mesh::from(RegularPolygon::new(30.0, 12)),
        17 => Mesh::from(RegularPolygon::new(34.0, 5)), // Star
        18 => Mesh::from(Ellipse { half_size: Vec2::new(34.0, 22.0) }),
        19 => Mesh::from(CirclePrimitive { radius: 32.0 }), // Sector fallback
        20 => Mesh::from(CirclePrimitive { radius: 32.0 }), // Semicircle fallback
        21 => Mesh::from(CirclePrimitive { radius: 30.0 }), // Crescent fallback
        22 => Mesh::from(RegularPolygon::new(32.0, 3)), // Isosceles
        23 => Mesh::from(RegularPolygon::new(32.0, 3)), // Equilateral
        24 => Mesh::from(RegularPolygon::new(32.0, 3)), // Right
        25 => Mesh::from(RegularPolygon::new(32.0, 3)), // Scalene
        // 3D shapes: cycle through 3-8 sides
        26..=45 => {
            let sides = 3 + ((level - 26) % 6);
            Mesh::from(RegularPolygon::new(32.0, sides))
        }
        // Prestige shapes
        46..=50 => Mesh::from(RegularPolygon::new(34.0, 3)),
        _ => Mesh::from(CirclePrimitive { radius: 30.0 }),
    }
}

/// Spawns a shape at `(col, row)` and registers it in `grid`.
/// The level-number label is a separate entity whose ID is stored on the
/// shape via [`LabelEntity`] so it can be despawned in O(1).
pub fn spawn_shape(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
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

    // Create mesh and material
    let mesh_handle = meshes.add(shape_mesh(level));
    let material_handle = materials.add(ColorMaterial {
        color: shape_color(level),
        ..default()
    });

    // Build shape entity components
    let mut shape_bundle = commands.spawn((
        Mesh2d(mesh_handle),
        MeshMaterial2d(material_handle),
        Transform::from_xyz(world.x, world.y, 1.0).with_scale(Vec3::ZERO),
        GridPos { col, row },
        ShapeLevel(level),
        LabelEntity(label_entity),
        SpawnPop(Timer::from_seconds(0.25, TimerMode::Once)),
    ));

    // Add visual effects for special tiers
    if level >= 26 && level <= 45 {
        // Pulse effect for 3D shapes
        shape_bundle.insert(PulseEffect {
            timer: Timer::from_seconds(1.5, TimerMode::Repeating),
            base_scale: 1.0,
        });
    } else if level >= 46 {
        // Glow effect for prestige shapes
        shape_bundle.insert(GlowEffect {
            timer: Timer::from_seconds(2.0, TimerMode::Repeating),
        });
    }

    let shape_entity = shape_bundle.id();

    grid.insert(col, row, shape_entity);
    shape_entity
}

// ── Drag-to-merge ─────────────────────────────────────────────────────────────

pub fn handle_drag_start(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    mut drag_state: ResMut<DragState>,
    mut grid: ResMut<Grid>,
    shapes_q: Query<(&GridPos, &Transform)>,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }
    if drag_state.dragging.is_some() {
        return; // Already dragging
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

    // Check if there's a shape at this cell
    if let Some(entity) = grid.get(col, row) {
        if let Ok((grid_pos, transform)) = shapes_q.get(entity) {
            let shape_world = Vec2::new(transform.translation.x, transform.translation.y);
            let offset = world_pos - shape_world;
            
            // Remove from grid temporarily (cell appears free while dragging)
            grid.remove(col, row);
            
            drag_state.dragging = Some(DragInfo {
                entity,
                original_col: grid_pos.col,
                original_row: grid_pos.row,
                offset,
            });
        }
    }
}

pub fn handle_drag_update(
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    drag_state: Res<DragState>,
    mut shapes_q: Query<&mut Transform>,
) {
    let Some(ref drag_info) = drag_state.dragging else {
        return;
    };

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

    // Update dragged shape position (follow cursor with offset)
    if let Ok(mut transform) = shapes_q.get_mut(drag_info.entity) {
        transform.translation.x = world_pos.x - drag_info.offset.x;
        transform.translation.y = world_pos.y - drag_info.offset.y;
        transform.translation.z = 5.0; // Lift above other shapes
    }
}

pub fn handle_drag_end(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    mut drag_state: ResMut<DragState>,
    mut grid: ResMut<Grid>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    upgrades: Res<UpgradeState>,
    mut gold: ResMut<GoldPool>,
    mut shape_queries: ParamSet<(
        Query<(&ShapeLevel, &LabelEntity)>,
        Query<(&ShapeLevel, &LabelEntity, &mut GridPos, &mut Transform)>,
        Query<&mut Transform, With<ShapeLabel>>,
    )>,
) {
    if !mouse.just_released(MouseButton::Left) {
        return;
    }
    
    let Some(drag_info) = drag_state.dragging.take() else {
        return;
    };

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
    
    let target_cell = world_to_grid(world_pos);

    // Get dragged shape info
    let (dragged_level, dragged_label_entity) = {
        let shape_info_q = shape_queries.p0();
        let Ok((shape_level, label_entity)) = shape_info_q.get(drag_info.entity) else {
            return;
        };
        (shape_level.0, label_entity.0)
    };

    if let Some((target_col, target_row)) = target_cell {
        // Check if target cell has a shape
        if let Some(target_entity) = grid.get(target_col, target_row) {
            // Try to merge
            let target_label_entity = {
                let shape_info_q = shape_queries.p0();
                match shape_info_q.get(target_entity) {
                    Ok((target_level, target_label))
                        if target_level.0 == dragged_level && dragged_level < MAX_LEVEL =>
                    {
                        Some(target_label.0)
                    }
                    _ => None,
                }
            };
            if let Some(target_label_entity) = target_label_entity {
                // Merge! Despawn dragged shape and its label
                commands.entity(dragged_label_entity).despawn();
                commands.entity(drag_info.entity).despawn();

                // Upgrade target in place
                let new_level = dragged_level + 1;
                let new_color = shape_color(new_level);
                let new_color_linear: LinearRgba = new_color.to_linear();

                // Despawn old target label
                commands.entity(target_label_entity).despawn();

                // Spawn new label for upgraded level
                let world = grid_to_world(target_col, target_row);
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

                // Update target shape
                let new_mesh = meshes.add(shape_mesh(new_level));
                let new_material = materials.add(ColorMaterial {
                    color: Color::linear_rgb(3.5, 3.5, 3.5), // HDR white flash
                    ..default()
                });

                commands.entity(target_entity).insert((
                    Mesh2d(new_mesh),
                    MeshMaterial2d(new_material),
                    ShapeLevel(new_level),
                    LabelEntity(new_label),
                    MergeFlash {
                        timer: Timer::from_seconds(0.35, TimerMode::Once),
                        base_color: new_color_linear,
                    },
                ));

                // Add visual effects for special tiers
                if new_level >= 26 && new_level <= 45 {
                    commands.entity(target_entity).insert(PulseEffect {
                        timer: Timer::from_seconds(1.5, TimerMode::Repeating),
                        base_scale: 1.0,
                    });
                } else if new_level >= 46 {
                    commands.entity(target_entity).insert(GlowEffect {
                        timer: Timer::from_seconds(2.0, TimerMode::Repeating),
                    });
                }

                return; // Merge complete
            }
        } else if Grid::in_bounds(target_col, target_row) {
            // Target cell is empty and in bounds - place shape there
            let world = grid_to_world(target_col, target_row);
            {
                let mut dragged_shape_q = shape_queries.p1();
                let Ok((_, _, mut grid_pos, mut transform)) = dragged_shape_q.get_mut(drag_info.entity) else {
                    return;
                };
                transform.translation.x = world.x;
                transform.translation.y = world.y;
                transform.translation.z = 1.0;
                grid_pos.col = target_col;
                grid_pos.row = target_row;
            }
            grid.insert(target_col, target_row, drag_info.entity);

            // Update label position
            if let Ok(mut label_transform) = shape_queries.p2().get_mut(dragged_label_entity) {
                label_transform.translation.x = world.x;
                label_transform.translation.y = world.y;
            }

            // Award per-click gold when the shape is returned to its original cell (a click, not a drag).
            if target_col == drag_info.original_col && target_row == drag_info.original_row {
                let bonus = upgrades.click_gold_per_click();
                gold.total += bonus;
                gold.total_gold_earned += bonus;
            }

            return;
        }
    }

    // If we get here, snap back to original position
    let mut dragged_shape_q = shape_queries.p1();
    let Ok((_, _, _, mut transform)) = dragged_shape_q.get_mut(drag_info.entity) else {
        return;
    };
    let world = grid_to_world(drag_info.original_col, drag_info.original_row);
    transform.translation.x = world.x;
    transform.translation.y = world.y;
    transform.translation.z = 1.0;
    grid.insert(drag_info.original_col, drag_info.original_row, drag_info.entity);
}

// ── Shop system ───────────────────────────────────────────────────────────────

/// Handle clicks on the buy-quantity selector buttons in the top HUD.
pub fn handle_buy_quantity_selection(
    interaction_q: Query<(&Interaction, &BuyQuantityButton), Changed<Interaction>>,
    mut buy_qty: ResMut<BuyQuantity>,
) {
    for (interaction, btn) in interaction_q.iter() {
        if *interaction == Interaction::Pressed {
            *buy_qty = btn.0;
        }
    }
}

pub fn handle_shop_purchase(
    interaction_q: Query<(&Interaction, &ShopButton), Changed<Interaction>>,
    mut gold: ResMut<GoldPool>,
    mut shop: ResMut<ShopState>,
    mut grid: ResMut<Grid>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    buy_qty: Res<BuyQuantity>,
) {
    for (interaction, btn) in interaction_q.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let slot_index = btn.0;
        if slot_index >= shop.available_levels.len() {
            continue;
        }

        // Determine how many purchases to attempt
        let mut remaining_cells: u32 = (0..GRID_ROWS)
            .flat_map(|r| (0..GRID_COLS).map(move |c| (c, r)))
            .filter(|&(c, r)| grid.is_empty(c, r))
            .count() as u32;

        let count = buy_qty.count().unwrap_or(remaining_cells).min(remaining_cells);

        for _ in 0..count {
            if remaining_cells == 0 {
                break;
            }

            let level = shop.available_levels[slot_index];
            let cost = shape_shop_cost(level);

            if gold.total < cost {
                break;
            }
            gold.total -= cost;

            // Find first empty cell (scan bottom-left to top-right)
            let mut spawn_cell = None;
            'outer: for row in 0..GRID_ROWS {
                for col in 0..GRID_COLS {
                    if grid.is_empty(col, row) {
                        spawn_cell = Some((col, row));
                        break 'outer;
                    }
                }
            }

            let Some((col, row)) = spawn_cell else {
                // Grid is full; refund the cost we just deducted and stop
                gold.total += cost;
                break;
            };

            spawn_shape(&mut commands, &mut meshes, &mut materials, &mut grid, col, row, level);
            remaining_cells -= 1;

            // Refresh this shop slot with a new random level
            // Weighted: 70% level 1, 20% level 2, 10% level 3+ (up to unlocked max)
            let max_unlocked = 3; // TODO: Could make this dynamic based on player progress
            let roll = rand::random::<f32>();
            let new_level = if roll < 0.7 {
                1
            } else if roll < 0.9 {
                2
            } else {
                (rand::random::<u32>() % max_unlocked.min(MAX_LEVEL)).max(1)
            };
            shop.available_levels[slot_index] = new_level;
        }
    }
}

// ── Gold generation ───────────────────────────────────────────────────────────

pub fn generate_gold(
    time: Res<Time>,
    mut gold: ResMut<GoldPool>,
    upgrades: Res<UpgradeState>,
    rebirth: Res<RebirthState>,
    shapes_q: Query<&ShapeLevel>,
) {
    let dt = time.delta_secs_f64();
    let base_rate: f64 = shapes_q.iter().map(|sl| gold_rate(sl.0)).sum();
    let total_rate = base_rate
        * upgrades.aura_multiplier()
        * rebirth.rebirth_gold_multiplier()
        * rebirth.paragon_aura_multiplier();
    gold.rate = total_rate;
    gold.total += total_rate * dt;
    gold.total_gold_earned += total_rate * dt;
}

// ── Upgrade purchases ─────────────────────────────────────────────────────────

pub fn handle_upgrades(
    interaction_q: Query<(&Interaction, &UpgradeButton), Changed<Interaction>>,
    mut gold: ResMut<GoldPool>,
    mut upgrades: ResMut<UpgradeState>,
) {
    for (interaction, btn) in interaction_q.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match btn.0 {
            UpgradeKind::AuraMultiplier => {
                let cost = upgrades.aura_multi_cost();
                if gold.total >= cost {
                    gold.total -= cost;
                    upgrades.aura_multi_level += 1;
                }
            }
            UpgradeKind::TokenCapacity => {
                // Legacy upgrade - no longer used, but kept for UI compatibility
            }
            UpgradeKind::MergeSpeed => {
                // Legacy upgrade - no longer used (no auto-merge timer)
            }
            UpgradeKind::ClickGold => {
                let cost = upgrades.click_gold_cost();
                if gold.total >= cost {
                    gold.total -= cost;
                    upgrades.click_gold_level += 1;
                }
            }
        }
    }
}

// ── Rebirth ───────────────────────────────────────────────────────────────────

/// Triggered when the player presses the Rebirth button.
///
/// Requirements: total_gold_earned >= rebirth threshold.
/// On success the grid is cleared, session resources are reset, and the rebirth
/// count / Paragon Points are updated.
pub fn handle_rebirth(
    mut commands: Commands,
    interaction_q: Query<&Interaction, (Changed<Interaction>, With<RebirthButton>)>,
    mut grid: ResMut<Grid>,
    shapes_q: Query<(&ShapeLevel, &LabelEntity)>,
    mut gold: ResMut<GoldPool>,
    mut upgrades: ResMut<UpgradeState>,
    mut rebirth: ResMut<RebirthState>,
) {
    if interaction_q.iter().all(|i| *i != Interaction::Pressed) {
        return;
    }

    // Check if we've earned enough gold
    let threshold = rebirth.rebirth_gold_requirement();
    if gold.total_gold_earned < threshold {
        return; // Threshold not met
    }

    // Determine the highest level shape currently on the grid (for PP calculation)
    let max_level = shapes_q.iter().map(|(sl, _)| sl.0).max().unwrap_or(0);

    // Award Paragon Points and advance the rebirth counter
    rebirth.paragon_points += RebirthState::pp_earned(max_level);
    rebirth.rebirth_count += 1;

    // Despawn every shape entity (and its floating label) then clear the grid
    for &entity in grid.cells.values() {
        if let Ok((_, label)) = shapes_q.get(entity) {
            commands.entity(label.0).despawn();
        }
        commands.entity(entity).despawn();
    }
    grid.cells.clear();

    // Reset all session-scoped resources to their defaults
    let reset_gold = GoldPool {
        total: 10.0,
        rate: 0.0,
        total_gold_earned: gold.total_gold_earned, // Preserve lifetime total!
    };
    *gold = reset_gold;
    *upgrades = UpgradeState::default();

    // Persist progress immediately after the rebirth
    write_save(&SaveData {
        gold_total: gold.total,
        total_gold_earned: gold.total_gold_earned,
        aura_multi_level: upgrades.aura_multi_level,
        merge_speed_level: upgrades.merge_speed_level,
        rebirth_count: rebirth.rebirth_count,
        paragon_points: rebirth.paragon_points,
        paragon_aura_level: rebirth.paragon_aura_level,
        shapes: vec![],
    });
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
        }
    }
}
