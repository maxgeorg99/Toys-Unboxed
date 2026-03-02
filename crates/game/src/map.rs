use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

pub fn spawn_map(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(TiledMap(asset_server.load("box-map.tmx")));
}
