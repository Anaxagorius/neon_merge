use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

use crate::components::{GridPos, ShapeLevel};
use crate::resources::{GoldPool, Grid, RebirthState, UpgradeState};
use crate::systems::spawn_shape;

// ── Save file location ────────────────────────────────────────────────────────

const SAVE_FILE: &str = "neon_merge_save.json";

fn save_path() -> PathBuf {
    PathBuf::from(SAVE_FILE)
}

// ── Data structure ────────────────────────────────────────────────────────────

/// All persistent game state that is written to / read from disk.
#[derive(Serialize, Deserialize, Default)]
pub struct SaveData {
    pub gold_total: f64,
    pub total_gold_earned: f64,
    // Session upgrades
    pub aura_multi_level: u32,
    pub merge_speed_level: u32,
    // Rebirth / Paragon
    pub rebirth_count: u32,
    pub paragon_points: u32,
    pub paragon_aura_level: u32,
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
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut gold: ResMut<GoldPool>,
    mut upgrades: ResMut<UpgradeState>,
    mut rebirth: ResMut<RebirthState>,
) {
    let Some(data) = read_save() else {
        return; // no save file — use defaults
    };

    gold.total = data.gold_total;
    gold.total_gold_earned = data.total_gold_earned;
    upgrades.aura_multi_level = data.aura_multi_level;
    upgrades.merge_speed_level = data.merge_speed_level;

    rebirth.rebirth_count = data.rebirth_count;
    rebirth.paragon_points = data.paragon_points;
    rebirth.paragon_aura_level = data.paragon_aura_level;
    
    for saved in data.shapes {
        if Grid::in_bounds(saved.col, saved.row) && grid.is_empty(saved.col, saved.row) {
            spawn_shape(
                &mut commands,
                &mut meshes,
                &mut materials,
                &mut grid,
                saved.col,
                saved.row,
                saved.level,
            );
        }
    }

    info!(
        "Save loaded: {} rebirths, {:.1} gold",
        rebirth.rebirth_count, gold.total
    );
}

/// Periodic auto-save (every 30 s).
pub fn auto_save(
    time: Res<Time>,
    mut timer: ResMut<AutoSaveTimer>,
    gold: Res<GoldPool>,
    upgrades: Res<UpgradeState>,
    rebirth: Res<RebirthState>,
    shapes_q: Query<(&ShapeLevel, &GridPos)>,
) {
    timer.0.tick(time.delta());
    if !timer.0.just_finished() {
        return;
    }
    write_save(&SaveData {
        gold_total: gold.total,
        total_gold_earned: gold.total_gold_earned,
        aura_multi_level: upgrades.aura_multi_level,
        merge_speed_level: upgrades.merge_speed_level,
        rebirth_count: rebirth.rebirth_count,
        paragon_points: rebirth.paragon_points,
        paragon_aura_level: rebirth.paragon_aura_level,
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
