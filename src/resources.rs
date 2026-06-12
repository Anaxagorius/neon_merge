use bevy::prelude::*;
use std::collections::HashMap;

// ── Grid ─────────────────────────────────────────────────────────────────────

/// Pixel step between adjacent cell centres (cell size + gap).
pub const CELL_STEP: f32 = 76.0;
/// Visual size of each cell sprite.
pub const CELL_SIZE: f32 = 72.0;
/// Number of columns.
pub const GRID_COLS: i32 = 5;
/// Number of rows.
pub const GRID_ROWS: i32 = 7;

/// World-space centre of the grid (shifted down slightly for HUD room).
pub const GRID_ORIGIN: Vec2 = Vec2::new(0.0, -60.0);

/// Map from grid position → entity occupying that cell.
#[derive(Resource, Default)]
pub struct Grid {
    pub cells: HashMap<(i32, i32), Entity>,
}

impl Grid {
    pub fn is_empty(&self, col: i32, row: i32) -> bool {
        !self.cells.contains_key(&(col, row))
    }

    pub fn get(&self, col: i32, row: i32) -> Option<Entity> {
        self.cells.get(&(col, row)).copied()
    }

    pub fn insert(&mut self, col: i32, row: i32, entity: Entity) {
        self.cells.insert((col, row), entity);
    }

    pub fn remove(&mut self, col: i32, row: i32) {
        self.cells.remove(&(col, row));
    }

    pub fn in_bounds(col: i32, row: i32) -> bool {
        col >= 0 && col < GRID_COLS && row >= 0 && row < GRID_ROWS
    }

    /// All four cardinal neighbours that are in-bounds.
    pub fn neighbours(col: i32, row: i32) -> impl Iterator<Item = (i32, i32)> {
        let candidates = [
            (col - 1, row),
            (col + 1, row),
            (col, row - 1),
            (col, row + 1),
        ];
        candidates
            .into_iter()
            .filter(|&(c, r)| Grid::in_bounds(c, r))
    }
}

/// Convert a `(col, row)` grid position to a world-space `Vec3`.
pub fn grid_to_world(col: i32, row: i32) -> Vec3 {
    let x = GRID_ORIGIN.x + col as f32 * CELL_STEP
        - (GRID_COLS - 1) as f32 * CELL_STEP * 0.5;
    let y = GRID_ORIGIN.y + row as f32 * CELL_STEP
        - (GRID_ROWS - 1) as f32 * CELL_STEP * 0.5;
    Vec3::new(x, y, 1.0)
}

/// Convert a world-space point to the nearest `(col, row)`, or `None` if
/// it doesn't land inside the grid bounds.
pub fn world_to_grid(world: Vec2) -> Option<(i32, i32)> {
    let local = world - GRID_ORIGIN
        + Vec2::new(
            (GRID_COLS - 1) as f32 * CELL_STEP * 0.5,
            (GRID_ROWS - 1) as f32 * CELL_STEP * 0.5,
        );
    let col = (local.x / CELL_STEP).round() as i32;
    let row = (local.y / CELL_STEP).round() as i32;
    if Grid::in_bounds(col, row) {
        Some((col, row))
    } else {
        None
    }
}

// ── Aura (primary currency) ───────────────────────────────────────────────────

#[derive(Resource)]
pub struct AuraPool {
    pub total: f64,
    pub rate: f64, // aura per second (computed each frame)
}

impl Default for AuraPool {
    fn default() -> Self {
        Self {
            total: 10.0,
            rate: 0.0,
        }
    }
}

// ── Spawn Tokens ──────────────────────────────────────────────────────────────

/// Governs how often the player can place new shapes.
#[derive(Resource)]
pub struct SpawnTokens {
    pub current: f32,
    pub max: f32,
    pub regen_rate: f32, // tokens per second
}

impl Default for SpawnTokens {
    fn default() -> Self {
        Self {
            current: 5.0,
            max: 10.0,
            regen_rate: 0.5, // 0.5 tokens per second (one token every 2 seconds)
        }
    }
}

// ── Merge timer ───────────────────────────────────────────────────────────────

/// Tick that drives automatic merge detection.
#[derive(Resource)]
pub struct MergeTimer(pub Timer);

impl Default for MergeTimer {
    fn default() -> Self {
        MergeTimer(Timer::from_seconds(0.35, TimerMode::Repeating))
    }
}

// ── Upgrade state ─────────────────────────────────────────────────────────────

/// Tracks the level of each purchased upgrade.
#[derive(Resource, Default)]
pub struct UpgradeState {
    pub aura_multi_level: u32,
    pub token_cap_level: u32,
    pub merge_speed_level: u32,
}

impl UpgradeState {
    pub fn aura_multi_cost(&self) -> f64 {
        50.0 * 5.0f64.powi(self.aura_multi_level as i32)
    }
    pub fn token_cap_cost(&self) -> f64 {
        25.0 * 4.0f64.powi(self.token_cap_level as i32)
    }
    pub fn merge_speed_cost(&self) -> f64 {
        30.0 * 6.0f64.powi(self.merge_speed_level as i32)
    }
    pub fn aura_multiplier(&self) -> f64 {
        1.0 + 0.5 * self.aura_multi_level as f64
    }
    pub fn token_capacity(&self) -> f32 {
        10.0 + 5.0 * self.token_cap_level as f32
    }
    pub fn merge_interval(&self) -> f32 {
        Self::merge_interval_at(self.merge_speed_level)
    }
    pub fn merge_interval_at(level: u32) -> f32 {
        (0.35 / (1.0 + 0.3 * level as f32)).max(0.05)
    }
}

// ── Large-number formatter ────────────────────────────────────────────────────

/// Formats an aura value with appropriate suffix (K, M, B, T, Qa).
pub fn fmt_aura(value: f64) -> String {
    if value >= 1e15 {
        format!("{:.2}Qa", value / 1e15)
    } else if value >= 1e12 {
        format!("{:.2}T", value / 1e12)
    } else if value >= 1e9 {
        format!("{:.2}B", value / 1e9)
    } else if value >= 1e6 {
        format!("{:.2}M", value / 1e6)
    } else if value >= 1e3 {
        format!("{:.1}K", value / 1e3)
    } else {
        format!("{:.1}", value)
    }
}
