use bevy::prelude::*;

mod animations;
mod components;
mod resources;
mod save;
mod systems;
mod ui;

use animations::{animate_merge_flash, animate_spawn_pop};
use resources::{AuraPool, Grid, MergeTimer, RebirthState, SpawnTokens, UpgradeState};
use save::{auto_save, load_game, AutoSaveTimer};
use systems::{
    auto_merge, generate_aura, handle_input, handle_paragon_upgrades, handle_rebirth,
    handle_upgrades, regen_tokens, setup_camera, setup_grid,
};
use ui::{
    setup_hud, update_hud, update_paragon_panel, update_rebirth_panel, update_upgrade_panel,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Neon Merge".into(),
                resolution: (480., 720.).into(),
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.04, 0.02, 0.10)))
        .init_resource::<Grid>()
        .init_resource::<AuraPool>()
        .init_resource::<SpawnTokens>()
        .init_resource::<MergeTimer>()
        .init_resource::<UpgradeState>()
        .init_resource::<RebirthState>()
        .init_resource::<AutoSaveTimer>()
        // Startup
        .add_systems(Startup, (setup_camera, setup_grid, setup_hud, load_game))
        // Every frame
        .add_systems(
            Update,
            (
                handle_input,
                regen_tokens,
                auto_merge,
                generate_aura,
                handle_upgrades,
                handle_rebirth,
                handle_paragon_upgrades,
                animate_spawn_pop,
                animate_merge_flash,
                auto_save,
                update_hud,
                update_upgrade_panel,
                update_rebirth_panel,
                update_paragon_panel,
            ),
        )
        .run();
}
