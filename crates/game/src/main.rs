mod animation;
mod battle;
mod camera;
mod components;
mod details_ui;
mod interaction;
mod map;
mod placement_visuals;
mod recruitment_ui;
mod troop_spawner;

use bevy::{asset::AssetPlugin, prelude::*};
use bevy_ecs_tiled::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Toys Unboxed".into(),
                    ..default()
                }),
                ..default()
            })
            .set(AssetPlugin {
                file_path: "../../assets".into(),
                ..default()
            }),
        )
        .add_plugins(TiledPlugin::default())
        .add_message::<troop_spawner::SpawnTroopEvent>()
        .add_systems(Startup, (
            camera::setup_camera,
            map::spawn_map,
            troop_spawner::init_units_config,
        ))
        .add_systems(Startup, (
            troop_spawner::spawn_enemy_troops,
            recruitment_ui::setup_recruitment_ui,
            details_ui::setup_details_ui,
            battle::setup_battle_ui,
        ).after(troop_spawner::init_units_config))
        .add_systems(Update, (
            camera::camera_pan,
            camera::camera_zoom,
            animation::animate_sprites,
            animation::on_animation_state_changed,
            troop_spawner::handle_spawn_troop_events,
            recruitment_ui::build_unit_grid,
            recruitment_ui::handle_recruit_buttons,
            recruitment_ui::handle_filter_buttons,
            recruitment_ui::highlight_active_filters,
        ))
        .add_systems(Update, details_ui::update_details_ui)
        .add_systems(Update, interaction::deselect_on_empty_click)
        .add_systems(Update, (
            interaction::drag_start,
            interaction::drag_move,
            interaction::drag_end,
        ).chain())
        .add_systems(Update, (interaction::troop_rotate, interaction::counter_rotate_sprites).chain())
        .add_systems(Update, (
            placement_visuals::draw_grid_overlay,
            placement_visuals::draw_drag_indicators,
        ))
        .add_systems(Update, battle::handle_start_battle_button)
        .add_systems(Update, (
            battle::init_battle,
            battle::tick_battle,
            battle::sync_sim_to_bevy,
            battle::sync_projectiles_to_bevy,
            battle::move_projectile_visuals,
            battle::animate_projectiles,
            battle::check_resolution,
        ).chain())
        .run();
}
