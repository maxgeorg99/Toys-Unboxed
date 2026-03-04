use std::collections::HashMap;

use bevy::prelude::*;
use bevy_spacetimedb::*;
use spacetimedb_sdk::Table;

use simulation_core::types::PlayerId;

use crate::battle::BattlePhase;
use crate::components::*;
use crate::module_bindings::{
    DbConnection, PlacedTroop, PlacedTroopTableAccess, RemoteModule, SessionPlayerTableAccess,
    SessionTableAccess, UserTableAccess,
    place_troop_reducer::place_troop as PlaceTroopReducer,
    update_troop_reducer::update_troop as UpdateTroopReducer,
};
use crate::troop_spawner::SpawnTroopEvent;

pub type SpacetimeDB<'a> = Res<'a, StdbConnection<DbConnection>>;

#[derive(Resource, Default)]
pub struct MultiplayerSession(pub Option<u64>);

#[derive(Resource, Default)]
pub struct LocalTroopMapping {
    pub entity_to_stdb_id: HashMap<Entity, u64>,
}

pub fn connect_to_stdb(world: &mut World) {
    connect_with_token::<DbConnection, RemoteModule>(world, None);
}

pub fn on_connected(messages: Option<ReadStdbConnectedMessage>, stdb: Option<SpacetimeDB>) {
    let (Some(mut messages), Some(stdb)) = (messages, stdb) else {
        return;
    };
    for _ in messages.read() {
        info!("Connected to SpacetimeDB");

        stdb.subscription_builder()
            .on_applied(|_| {})
            .on_error(|_, err| error!("user subscription failed: {}", err))
            .subscribe("SELECT * FROM user");

        stdb.subscription_builder()
            .on_applied(|_| {})
            .on_error(|_, err| error!("session subscription failed: {}", err))
            .subscribe("SELECT * FROM session");

        stdb.subscription_builder()
            .on_applied(|_| {})
            .on_error(|_, err| error!("session_player subscription failed: {}", err))
            .subscribe("SELECT * FROM session_player");

        stdb.subscription_builder()
            .on_applied(|_| {})
            .on_error(|_, err| error!("placed_troop subscription failed: {}", err))
            .subscribe("SELECT * FROM placed_troop");
    }
}

pub fn on_disconnected(messages: Option<ReadStdbDisconnectedMessage>) {
    let Some(mut messages) = messages else { return };
    for _ in messages.read() {
        warn!("Disconnected from SpacetimeDB");
    }
}

pub fn on_connection_error(messages: Option<ReadStdbConnectionErrorMessage>) {
    let Some(mut messages) = messages else { return };
    for msg in messages.read() {
        error!("SpacetimeDB connection error: {:?}", msg.err);
    }
}

pub fn init_multiplayer_resources(mut commands: Commands) {
    commands.init_resource::<MultiplayerSession>();
    commands.init_resource::<LocalTroopMapping>();
}

pub fn detect_multiplayer_session(
    stdb: Option<SpacetimeDB>,
    mut mp_session: ResMut<MultiplayerSession>,
) {
    let Some(stdb) = stdb else { return };
    let Some(my_identity) = stdb.try_identity() else { return };

    let session_id = stdb
        .db()
        .user()
        .identity()
        .find(&my_identity)
        .and_then(|u| u.session_id);

    if let Some(sid) = session_id {
        let player_count = stdb
            .db()
            .session_player()
            .iter()
            .filter(|sp| sp.session_id == sid)
            .count();
        if player_count > 1 {
            mp_session.0 = Some(sid);
        }
    }
}

pub fn sync_local_placement_to_server(
    stdb: Option<SpacetimeDB>,
    mut removed_dragging: RemovedComponents<Dragging>,
    troops: Query<(&TroopUnitId, &Transform, &Owner)>,
    mut mapping: ResMut<LocalTroopMapping>,
    phase: Res<BattlePhase>,
) {
    if *phase != BattlePhase::Placement {
        return;
    }
    let Some(stdb) = stdb else { return };

    for entity in removed_dragging.read() {
        let Ok((unit_id, transform, owner)) = troops.get(entity) else {
            continue;
        };
        if owner.0 != PlayerId(0) {
            continue;
        }

        let pos = transform.translation;
        let rotation = transform.rotation.to_euler(EulerRot::ZYX).0;

        if let Some(&stdb_id) = mapping.entity_to_stdb_id.get(&entity) {
            let _ = stdb
                .reducers()
                .update_troop(stdb_id, pos.x, pos.y, rotation);
        } else {
            let _ = stdb
                .reducers()
                .place_troop(unit_id.0.clone(), pos.x, pos.y, rotation);
        }
    }
}

pub fn handle_own_troop_inserted(
    mut messages: Option<ReadInsertMessage<PlacedTroop>>,
    stdb: Option<SpacetimeDB>,
    mut mapping: ResMut<LocalTroopMapping>,
    troops: Query<(Entity, &TroopUnitId, &Transform, &Owner)>,
) {
    let Some(ref mut messages) = messages else { return };
    let Some(stdb) = stdb else { return };
    let Some(my_identity) = stdb.try_identity() else { return };

    for msg in messages.read() {
        if msg.row.owner_identity != my_identity {
            continue;
        }
        for (entity, unit_id, transform, owner) in &troops {
            if owner.0 != PlayerId(0) || unit_id.0 != msg.row.unit_id {
                continue;
            }
            if mapping.entity_to_stdb_id.contains_key(&entity) {
                continue;
            }
            let pos = transform.translation;
            if (pos.x - msg.row.world_x).abs() < 1.0 && (pos.y - msg.row.world_y).abs() < 1.0 {
                mapping.entity_to_stdb_id.insert(entity, msg.row.id);
                break;
            }
        }
    }
}

pub fn spawn_opponent_troops_on_battle_start(
    stdb: Option<SpacetimeDB>,
    phase: Res<BattlePhase>,
    mp_session: Res<MultiplayerSession>,
    mut spawn_events: MessageWriter<SpawnTroopEvent>,
) {
    if !phase.is_changed() || *phase != BattlePhase::Battle {
        return;
    }
    let Some(sid) = mp_session.0 else { return };
    let Some(stdb) = stdb else { return };
    let Some(my_identity) = stdb.try_identity() else { return };

    for troop in stdb.db().placed_troop().iter() {
        if troop.session_id != sid || troop.owner_identity == my_identity {
            continue;
        }
        spawn_events.write(SpawnTroopEvent {
            unit_id: troop.unit_id.clone(),
            world_pos: Vec2::new(troop.world_x, troop.world_y),
            owner: PlayerId(1),
            remote_troop_id: Some(troop.id),
        });
    }
}
