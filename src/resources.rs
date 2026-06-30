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

// ── Gold (primary currency) ───────────────────────────────────────────────────

#[derive(Resource)]
pub struct GoldPool {
    pub total: f64,
    pub rate: f64, // gold per second (computed each frame)
    pub total_gold_earned: f64, // lifetime total for rebirth gating
}

impl Default for GoldPool {
    fn default() -> Self {
        Self {
            total: 10.0,
            rate: 0.0,
            total_gold_earned: 0.0,
        }
    }
}

// ── Drag state ────────────────────────────────────────────────────────────────

/// Resource to track drag-and-drop state.
#[derive(Resource, Default)]
pub struct DragState {
    pub dragging: Option<DragInfo>,
}

pub struct DragInfo {
    pub entity: Entity,
    pub original_col: i32,
    pub original_row: i32,
    pub offset: Vec2, // cursor offset from shape center at pick-up
}

// ── Shop system ───────────────────────────────────────────────────────────────

/// Shop state for purchasing shapes.
#[derive(Resource)]
pub struct ShopState {
    pub available_levels: Vec<u32>, // shape levels available for purchase
    pub refresh_cost: f64,          // cost to refresh the shop (future use, start at 0)
}

impl Default for ShopState {
    fn default() -> Self {
        Self {
            available_levels: vec![1, 1, 2], // start: two circles and one triangle
            refresh_cost: 0.0,
        }
    }
}

// ── Upgrade state ─────────────────────────────────────────────────────────────

/// Tracks the level of each purchased upgrade.
#[derive(Resource, Default)]
pub struct UpgradeState {
    pub aura_multi_level: u32,
    pub merge_speed_level: u32,
}

impl UpgradeState {
    pub fn aura_multi_cost(&self) -> f64 {
        50.0 * 5.0f64.powi(self.aura_multi_level as i32)
    }
    pub fn merge_speed_cost(&self) -> f64 {
        30.0 * 6.0f64.powi(self.merge_speed_level as i32)
    }
    pub fn aura_multiplier(&self) -> f64 {
        1.0 + 0.5 * self.aura_multi_level as f64
    }
    pub fn next_aura_multiplier(&self) -> f64 {
        1.0 + 0.5 * (self.aura_multi_level + 1) as f64
    }
}

// ── Rebirth & Paragon ─────────────────────────────────────────────────────────

/// Persistent state that survives Rebirths (Paragon levels) and accumulates
/// across them (rebirth count, Paragon Points).
#[derive(Resource, Default)]
pub struct RebirthState {
    /// Number of times the player has rebirthed.
    pub rebirth_count: u32,
    /// Unspent Paragon Points.
    pub paragon_points: u32,
    /// Paragon upgrade level: permanent Gold multiplier.
    pub paragon_aura_level: u32,
}

impl RebirthState {
    /// Permanently compounds Gold rate by ×1.20 per rebirth.
    /// 1 rebirth → ×1.20, 2 → ×1.44, 5 → ×2.49, etc.
    pub fn rebirth_gold_multiplier(&self) -> f64 {
        1.20_f64.powi(self.rebirth_count as i32)
    }

    /// +30 % permanent Gold multiplier per Paragon Aura level.
    pub fn paragon_aura_multiplier(&self) -> f64 {
        1.0 + 0.30 * self.paragon_aura_level as f64
    }

    pub fn next_paragon_aura_multiplier(&self) -> f64 {
        1.0 + 0.30 * (self.paragon_aura_level + 1) as f64
    }

    /// PP cost doubles each level: 1, 2, 4, 8, …
    pub fn paragon_aura_cost(&self) -> u32 {
        1u32.checked_shl(self.paragon_aura_level).unwrap_or(u32::MAX)
    }

    /// Paragon Points earned when rebirthing with the given highest shape level.
    /// Formula: floor(max_level / 5), minimum 1 when threshold is met.
    pub fn pp_earned(max_level: u32) -> u32 {
        (max_level / 5).max(1)
    }
    
    /// Total gold required to unlock rebirth.
    pub fn rebirth_gold_requirement(&self) -> f64 {
        10_000.0 * (self.rebirth_count + 1) as f64
    }
}

// ── Buy quantity ──────────────────────────────────────────────────────────────

/// How many items to purchase per shop button click.
#[derive(Resource, Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum BuyQuantity {
    #[default]
    One,
    Ten,
    TwentyFive,
    Fifty,
    Hundred,
    Max,
}

impl BuyQuantity {
    /// Display label for the selector button.
    pub fn label(self) -> &'static str {
        match self {
            BuyQuantity::One      => "x1",
            BuyQuantity::Ten      => "x10",
            BuyQuantity::TwentyFive => "x25",
            BuyQuantity::Fifty    => "x50",
            BuyQuantity::Hundred  => "x100",
            BuyQuantity::Max      => "MAX",
        }
    }

    /// Concrete count, or `None` for MAX (caller must compute from context).
    pub fn count(self) -> Option<u32> {
        match self {
            BuyQuantity::One      => Some(1),
            BuyQuantity::Ten      => Some(10),
            BuyQuantity::TwentyFive => Some(25),
            BuyQuantity::Fifty    => Some(50),
            BuyQuantity::Hundred  => Some(100),
            BuyQuantity::Max      => None,
        }
    }

    /// All variants in display order.
    pub const ALL: [BuyQuantity; 6] = [
        BuyQuantity::One,
        BuyQuantity::Ten,
        BuyQuantity::TwentyFive,
        BuyQuantity::Fifty,
        BuyQuantity::Hundred,
        BuyQuantity::Max,
    ];
}

// ── Large-number formatter ────────────────────────────────────────────────────

/// Formats a gold value with appropriate suffix (K, M, B, T, Qa).
pub fn fmt_gold(value: f64) -> String {
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
