mod animation;
mod battle;
mod camera;
mod components;
mod details_ui;
mod economy;
mod interaction;
mod lobby;
mod map;
mod menu;
#[allow(dead_code, unused, clippy::all)]
mod module_bindings;
mod networking;
mod placement_visuals;
mod recruitment_ui;
mod troop_spawner;

use bevy::{asset::AssetPlugin, prelude::*};
use bevy_ecs_tiled::prelude::*;
use bevy_spacetimedb::StdbPlugin;
use module_bindings::{
    DbConnection, RemoteModule, RemoteTables,
    UserTableAccess, SessionTableAccess, SessionPlayerTableAccess, PlacedTroopTableAccess,
};

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AppState {
    #[default]
    MainMenu,
    Lobby,
    InGame,
}

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
        .add_plugins(
            StdbPlugin::<DbConnection, RemoteModule>::default()
                .with_uri("http://127.0.0.1:3000")
                .with_module_name("toys-unboxed")
                .with_run_fn(DbConnection::run_threaded)
                .with_delayed_connect(true)
                .add_table(|t: &RemoteTables| t.user())
                .add_table(|t: &RemoteTables| t.session())
                .add_table(|t: &RemoteTables| t.session_player())
                .add_table(|t: &RemoteTables| t.placed_troop()),
        )
        .init_state::<AppState>()
        .add_message::<troop_spawner::SpawnTroopEvent>()
        // Global startup (always needed)
        .add_systems(Startup, (
            camera::setup_camera,
            troop_spawner::init_units_config,
        ))
        // Networking (runs in all states)
        .add_systems(Update, (
            networking::on_connected,
            networking::on_disconnected,
            networking::on_connection_error,
        ))
        // Menu
        .add_systems(OnEnter(AppState::MainMenu), menu::setup_menu)
        .add_systems(Update, menu::handle_play_button.run_if(in_state(AppState::MainMenu)))
        .add_systems(OnExit(AppState::MainMenu), menu::cleanup_menu)
        // Lobby
        .add_systems(OnEnter(AppState::Lobby), (
            networking::connect_to_stdb,
            lobby::setup_lobby,
        ).chain())
        .add_systems(Update, (
            lobby::handle_create_session,
            lobby::handle_join_by_code,
            lobby::handle_join_code_input,
            lobby::poll_session_status,
            lobby::handle_start_game,
            lobby::handle_leave_session,
            lobby::handle_lobby_back,
            lobby::update_panel_visibility,
            lobby::update_in_lobby_view,
        ).run_if(in_state(AppState::Lobby)))
        .add_systems(OnExit(AppState::Lobby), lobby::cleanup_lobby)
        // InGame setup
        .add_systems(OnEnter(AppState::InGame), (
            map::spawn_map,
            economy::setup_economy,
            networking::init_multiplayer_resources,
            troop_spawner::spawn_enemy_troops,
            recruitment_ui::setup_recruitment_ui,
            details_ui::setup_details_ui,
            battle::setup_battle_ui,
            economy::setup_economy_hud,
        ).after(troop_spawner::init_units_config))
        // InGame update
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
        ).run_if(in_state(AppState::InGame)))
        .add_systems(Update, details_ui::update_details_ui.run_if(in_state(AppState::InGame)))
        .add_systems(Update, interaction::deselect_on_empty_click.run_if(in_state(AppState::InGame)))
        .add_systems(Update, (
            interaction::drag_start,
            interaction::drag_move,
            interaction::drag_end,
        ).chain().run_if(in_state(AppState::InGame)))
        .add_systems(Update, (interaction::troop_rotate, interaction::counter_rotate_sprites).chain().run_if(in_state(AppState::InGame)))
        .add_systems(Update, (
            placement_visuals::draw_grid_overlay,
            placement_visuals::draw_drag_indicators,
        ).run_if(in_state(AppState::InGame)))
        .add_systems(Update, battle::handle_start_battle_button.run_if(in_state(AppState::InGame)))
        .add_systems(Update, economy::update_economy_hud.run_if(in_state(AppState::InGame)))
        .add_systems(Update, (
            battle::init_battle,
            battle::tick_battle,
            battle::sync_sim_to_bevy,
            battle::sync_projectiles_to_bevy,
            battle::move_projectile_visuals,
            battle::animate_projectiles,
            battle::check_resolution,
        ).chain().run_if(in_state(AppState::InGame)))
        .add_systems(Update, economy::advance_round.after(battle::check_resolution).run_if(in_state(AppState::InGame)))
        // Troop placement sync (InGame)
        .add_systems(Update, (
            networking::detect_multiplayer_session,
            networking::sync_local_placement_to_server,
            networking::handle_own_troop_inserted,
            networking::spawn_opponent_troops_on_battle_start,
        ).run_if(in_state(AppState::InGame)))
        .run();
}
