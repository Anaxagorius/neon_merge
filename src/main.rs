use bevy::prelude::*;

mod animations;
mod components;
mod resources;
mod save;
mod systems;
mod ui;

use animations::{animate_glow, animate_merge_flash, animate_pulse, animate_spawn_pop};
use resources::{BuyQuantity, GoldPool, Grid, DragState, ShopState, RebirthState, UpgradeState};
use save::{auto_save, load_game, AutoSaveTimer};
use systems::{
    generate_gold, handle_buy_quantity_selection, handle_drag_start, handle_drag_update,
    handle_drag_end, handle_shop_purchase, handle_paragon_upgrades, handle_rebirth,
    handle_upgrades, setup_camera, setup_grid,
};
use ui::{
    button_interaction, setup_hud, update_buy_quantity_ui, update_hud, update_paragon_ui,
    update_rebirth_ui, update_shop_ui, update_upgrade_ui,
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
        .init_resource::<GoldPool>()
        .init_resource::<DragState>()
        .init_resource::<ShopState>()
        .init_resource::<UpgradeState>()
        .init_resource::<RebirthState>()
        .init_resource::<AutoSaveTimer>()
        .init_resource::<BuyQuantity>()
        // Startup
        .add_systems(Startup, (setup_camera, setup_grid, setup_hud, load_game))
        // Every frame
        .add_systems(
            Update,
            (
                (
                    handle_drag_start,
                    handle_drag_update,
                    handle_drag_end,
                    handle_buy_quantity_selection,
                    handle_shop_purchase,
                    generate_gold,
                    handle_upgrades,
                    handle_rebirth,
                    handle_paragon_upgrades,
                    animate_spawn_pop,
                ),
                (
                    animate_merge_flash,
                    animate_pulse,
                    animate_glow,
                    auto_save,
                    update_hud,
                    update_buy_quantity_ui,
                    update_shop_ui,
                    update_upgrade_ui,
                    update_rebirth_ui,
                    update_paragon_ui,
                    button_interaction,
                ),
            ),
        )
        .run();
}
