mod animation;
mod camera;
mod components;
mod interaction;
mod map;
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
        .add_systems(Startup, (
            camera::setup_camera,
            map::spawn_map,
            troop_spawner::spawn_skull_troop,
        ))
        .add_systems(Update, (
            camera::camera_pan,
            camera::camera_zoom,
            animation::animate_sprites,
            animation::on_animation_state_changed,
        ))
        .add_systems(Update, (
            interaction::drag_start,
            interaction::drag_move,
            interaction::drag_end,
        ).chain())
        .add_systems(Update, interaction::troop_rotate)
        .run();
}
