use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

use crate::components::{GridPos, ShapeLevel};
use crate::resources::{AuraPool, MergeTimer, RebirthState, SpawnTokens, UpgradeState};
use crate::systems::spawn_shape;
use crate::resources::Grid;

// ── Save file location ────────────────────────────────────────────────────────

const SAVE_FILE: &str = "neon_merge_save.json";

fn save_path() -> PathBuf {
    PathBuf::from(SAVE_FILE)
}

// ── Data structure ────────────────────────────────────────────────────────────

/// All persistent game state that is written to / read from disk.
#[derive(Serialize, Deserialize, Default)]
pub struct SaveData {
    pub aura_total: f64,
    // Session upgrades
    pub aura_multi_level: u32,
    pub token_cap_level: u32,
    pub merge_speed_level: u32,
    // Rebirth / Paragon
    pub rebirth_count: u32,
    pub paragon_points: u32,
    pub paragon_aura_level: u32,
    pub paragon_regen_level: u32,
    // Grid state
    pub shapes: Vec<SavedShape>,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct SavedShape {
    pub col: i32,
    pub row: i32,
    pub level: u32,
}

// ── Helpers (called directly from systems) ────────────────────────────────────

/// Serialise `data` to `neon_merge_save.json` next to the executable.
/// Save errors are logged as warnings so players can diagnose failures
/// without crashing the game.
pub fn write_save(data: &SaveData) {
    match serde_json::to_string_pretty(data) {
        Ok(json) => {
            if let Err(e) = fs::write(save_path(), json) {
                warn!("Failed to write save file: {e}");
            }
        }
        Err(e) => warn!("Failed to serialise save data: {e}"),
    }
}

fn read_save() -> Option<SaveData> {
    let bytes = fs::read(save_path()).ok()?;
    serde_json::from_slice(&bytes).ok()
}

// ── Auto-save resource ────────────────────────────────────────────────────────

/// Ticks every frame; fires a save every 30 seconds.
#[derive(Resource)]
pub struct AutoSaveTimer(pub Timer);

impl Default for AutoSaveTimer {
    fn default() -> Self {
        AutoSaveTimer(Timer::from_seconds(30.0, TimerMode::Repeating))
    }
}

// ── Bevy systems ──────────────────────────────────────────────────────────────

/// Startup system — loads the save file and applies it to all persistent
/// resources.  Runs after every resource has been default-initialised.
pub fn load_game(
    mut commands: Commands,
    mut grid: ResMut<Grid>,
    mut aura: ResMut<AuraPool>,
    mut upgrades: ResMut<UpgradeState>,
    mut tokens: ResMut<SpawnTokens>,
    mut rebirth: ResMut<RebirthState>,
    mut merge_timer: ResMut<MergeTimer>,
) {
    let Some(data) = read_save() else {
        return; // no save file — use defaults
    };

    aura.total = data.aura_total;
    upgrades.aura_multi_level = data.aura_multi_level;
    upgrades.token_cap_level = data.token_cap_level;
    upgrades.merge_speed_level = data.merge_speed_level;

    // Sync derived values that depend on upgrade levels.
    tokens.max = upgrades.token_capacity();
    tokens.current = tokens.current.min(tokens.max);
    merge_timer.0 = Timer::from_seconds(upgrades.merge_interval(), TimerMode::Repeating);

    rebirth.rebirth_count = data.rebirth_count;
    rebirth.paragon_points = data.paragon_points;
    rebirth.paragon_aura_level = data.paragon_aura_level;
    rebirth.paragon_regen_level = data.paragon_regen_level;
    for saved in data.shapes {
        if Grid::in_bounds(saved.col, saved.row) && grid.is_empty(saved.col, saved.row) {
            spawn_shape(
                &mut commands,
                &mut grid,
                saved.col,
                saved.row,
                saved.level,
            );
        }
    }

    info!(
        "Save loaded: {} rebirths, {:.1} aura",
        rebirth.rebirth_count, aura.total
    );
}

/// Periodic auto-save (every 30 s).
pub fn auto_save(
    time: Res<Time>,
    mut timer: ResMut<AutoSaveTimer>,
    aura: Res<AuraPool>,
    upgrades: Res<UpgradeState>,
    rebirth: Res<RebirthState>,
    shapes_q: Query<(&ShapeLevel, &GridPos)>,
) {
    timer.0.tick(time.delta());
    if !timer.0.just_finished() {
        return;
    }
    write_save(&SaveData {
        aura_total: aura.total,
        aura_multi_level: upgrades.aura_multi_level,
        token_cap_level: upgrades.token_cap_level,
        merge_speed_level: upgrades.merge_speed_level,
        rebirth_count: rebirth.rebirth_count,
        paragon_points: rebirth.paragon_points,
        paragon_aura_level: rebirth.paragon_aura_level,
        paragon_regen_level: rebirth.paragon_regen_level,
        shapes: shapes_q
            .iter()
            .map(|(shape, pos)| SavedShape {
                col: pos.col,
                row: pos.row,
                level: shape.0,
            })
            .collect(),
    });
    info!("Game auto-saved.");
}
