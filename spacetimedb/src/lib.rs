use spacetimedb::{Identity, ReducerContext, SpacetimeType, Table};

// ─── Enums ──────────────────────────────────────────────────────────────────

#[derive(SpacetimeType, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SessionStatus {
    #[default]
    WaitingForPlayers,
    Ready,
    InProgress,
    Finished,
}

// ─── Tables ─────────────────────────────────────────────────────────────────

#[spacetimedb::table(name = user, public)]
pub struct User {
    #[primary_key]
    pub identity: Identity,
    pub name: Option<String>,
    #[index(btree)]
    pub online: bool,
    #[index(btree)]
    pub session_id: Option<u64>,
}

#[spacetimedb::table(name = session, public)]
#[derive(Clone)]
pub struct Session {
    #[primary_key]
    #[auto_inc]
    pub session_id: u64,
    #[unique]
    #[index(btree)]
    pub session_code: String,
    pub status: SessionStatus,
    pub max_players: u8,
}

#[spacetimedb::table(name = session_player, public)]
#[derive(Clone)]
pub struct SessionPlayer {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    #[index(btree)]
    pub session_id: u64,
    #[index(btree)]
    pub player_identity: Identity,
    pub slot: u8,
    pub is_host: bool,
}

#[spacetimedb::table(name = placed_troop, public)]
#[derive(Clone)]
pub struct PlacedTroop {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    #[index(btree)]
    pub session_id: u64,
    pub owner_identity: Identity,
    pub unit_id: String,
    pub world_x: f32,
    pub world_y: f32,
    pub rotation: f32,
}

// ─── Lifecycle ──────────────────────────────────────────────────────────────

#[spacetimedb::reducer(init)]
pub fn init(_ctx: &ReducerContext) {}

#[spacetimedb::reducer(client_connected)]
pub fn identity_connected(ctx: &ReducerContext) {
    let identity = ctx.sender;
    if let Some(mut user) = ctx.db.user().identity().find(identity) {
        user.online = true;
        if let Some(sid) = user.session_id {
            if ctx.db.session().session_id().find(sid).is_none() {
                user.session_id = None;
            }
        }
        ctx.db.user().identity().update(user);
    } else {
        ctx.db.user().insert(User {
            identity,
            name: None,
            online: true,
            session_id: None,
        });
    }
}

#[spacetimedb::reducer(client_disconnected)]
pub fn identity_disconnected(ctx: &ReducerContext) {
    let identity = ctx.sender;
    if let Some(mut user) = ctx.db.user().identity().find(identity) {
        user.online = false;
        let session_id = user.session_id;
        ctx.db.user().identity().update(user);

        if let Some(sid) = session_id {
            let all_offline = ctx
                .db
                .session_player()
                .iter()
                .filter(|sp| sp.session_id == sid)
                .all(|sp| {
                    ctx.db
                        .user()
                        .identity()
                        .find(sp.player_identity)
                        .map(|u| !u.online)
                        .unwrap_or(true)
                });

            if all_offline {
                cleanup_session(ctx, sid);
            }
        }
    }
}

fn cleanup_session(ctx: &ReducerContext, sid: u64) {
    let troops: Vec<u64> = ctx
        .db
        .placed_troop()
        .iter()
        .filter(|t| t.session_id == sid)
        .map(|t| t.id)
        .collect();
    for id in troops {
        ctx.db.placed_troop().id().delete(id);
    }

    let players: Vec<SessionPlayer> = ctx
        .db
        .session_player()
        .iter()
        .filter(|sp| sp.session_id == sid)
        .collect();
    for sp in &players {
        if let Some(mut u) = ctx.db.user().identity().find(sp.player_identity) {
            u.session_id = None;
            ctx.db.user().identity().update(u);
        }
        ctx.db.session_player().id().delete(sp.id);
    }

    ctx.db.session().session_id().delete(sid);
}

// ─── Session Reducers ───────────────────────────────────────────────────────

#[spacetimedb::reducer]
pub fn create_session(ctx: &ReducerContext) -> Result<(), String> {
    let identity = ctx.sender;
    let mut player = ctx
        .db
        .user()
        .identity()
        .find(identity)
        .ok_or("Player not found")?;

    if player.session_id.is_some() {
        return Err("Already in a session".to_string());
    }

    let code = generate_session_code(ctx);
    let session = ctx.db.session().insert(Session {
        session_id: 0,
        session_code: code,
        status: SessionStatus::WaitingForPlayers,
        max_players: 2,
    });

    ctx.db.session_player().insert(SessionPlayer {
        id: 0,
        session_id: session.session_id,
        player_identity: identity,
        slot: 1,
        is_host: true,
    });

    player.session_id = Some(session.session_id);
    ctx.db.user().identity().update(player);
    Ok(())
}

#[spacetimedb::reducer]
pub fn join_session(ctx: &ReducerContext, code: String) -> Result<(), String> {
    let identity = ctx.sender;
    let mut player = ctx
        .db
        .user()
        .identity()
        .find(identity)
        .ok_or("Player not found")?;

    if player.session_id.is_some() {
        return Err("Already in a session".to_string());
    }

    let mut session = ctx
        .db
        .session()
        .session_code()
        .find(&code)
        .ok_or("Session not found")?;

    if session.status != SessionStatus::WaitingForPlayers {
        return Err("Session is not accepting players".to_string());
    }

    let current: Vec<SessionPlayer> = ctx
        .db
        .session_player()
        .iter()
        .filter(|sp| sp.session_id == session.session_id)
        .collect();

    if current.len() >= session.max_players as usize {
        return Err("Session is full".to_string());
    }

    if current.iter().any(|sp| sp.player_identity == identity) {
        return Err("Already in this session".to_string());
    }

    let next_slot = (current.len() + 1) as u8;
    ctx.db.session_player().insert(SessionPlayer {
        id: 0,
        session_id: session.session_id,
        player_identity: identity,
        slot: next_slot,
        is_host: false,
    });

    if current.len() + 1 >= session.max_players as usize {
        session.status = SessionStatus::Ready;
    }
    ctx.db.session().session_id().update(session.clone());

    player.session_id = Some(session.session_id);
    ctx.db.user().identity().update(player);
    Ok(())
}

#[spacetimedb::reducer]
pub fn leave_session(ctx: &ReducerContext) -> Result<(), String> {
    let identity = ctx.sender;
    let mut player = ctx
        .db
        .user()
        .identity()
        .find(identity)
        .ok_or("Player not found")?;

    let session_id = player.session_id.ok_or("Not in a session")?;

    player.session_id = None;
    ctx.db.user().identity().update(player);

    let is_host = ctx
        .db
        .session_player()
        .iter()
        .find(|sp| sp.session_id == session_id && sp.player_identity == identity)
        .map(|sp| sp.is_host)
        .unwrap_or(false);

    let sp_ids: Vec<u64> = ctx
        .db
        .session_player()
        .iter()
        .filter(|sp| sp.session_id == session_id && sp.player_identity == identity)
        .map(|sp| sp.id)
        .collect();
    for id in sp_ids {
        ctx.db.session_player().id().delete(id);
    }

    if is_host {
        cleanup_session(ctx, session_id);
    } else {
        if let Some(mut session) = ctx.db.session().session_id().find(session_id) {
            session.status = SessionStatus::WaitingForPlayers;
            ctx.db.session().session_id().update(session);
        }
    }

    Ok(())
}

#[spacetimedb::reducer]
pub fn start_session(ctx: &ReducerContext) -> Result<(), String> {
    let identity = ctx.sender;
    let player = ctx
        .db
        .user()
        .identity()
        .find(identity)
        .ok_or("Player not found")?;

    let session_id = player.session_id.ok_or("Not in a session")?;

    let is_host = ctx
        .db
        .session_player()
        .iter()
        .any(|sp| sp.session_id == session_id && sp.player_identity == identity && sp.is_host);

    if !is_host {
        return Err("Only the host can start the session".to_string());
    }

    let mut session = ctx
        .db
        .session()
        .session_id()
        .find(session_id)
        .ok_or("Session not found")?;

    if session.status != SessionStatus::Ready {
        return Err("Session is not ready".to_string());
    }

    session.status = SessionStatus::InProgress;
    ctx.db.session().session_id().update(session);
    Ok(())
}

// ─── Troop Placement Reducers ───────────────────────────────────────────────

#[spacetimedb::reducer]
pub fn place_troop(
    ctx: &ReducerContext,
    unit_id: String,
    x: f32,
    y: f32,
    rotation: f32,
) -> Result<(), String> {
    let identity = ctx.sender;
    let player = ctx
        .db
        .user()
        .identity()
        .find(identity)
        .ok_or("Player not found")?;
    let session_id = player.session_id.ok_or("Not in a session")?;

    ctx.db.placed_troop().insert(PlacedTroop {
        id: 0,
        session_id,
        owner_identity: identity,
        unit_id,
        world_x: x,
        world_y: y,
        rotation,
    });
    Ok(())
}

#[spacetimedb::reducer]
pub fn update_troop(ctx: &ReducerContext, troop_id: u64, x: f32, y: f32, rotation: f32) -> Result<(), String> {
    let identity = ctx.sender;
    let mut troop = ctx
        .db
        .placed_troop()
        .id()
        .find(troop_id)
        .ok_or("Troop not found")?;

    if troop.owner_identity != identity {
        return Err("Not your troop".to_string());
    }

    troop.world_x = x;
    troop.world_y = y;
    troop.rotation = rotation;
    ctx.db.placed_troop().id().update(troop);
    Ok(())
}

#[spacetimedb::reducer]
pub fn remove_troop(ctx: &ReducerContext, troop_id: u64) -> Result<(), String> {
    let identity = ctx.sender;
    let troop = ctx
        .db
        .placed_troop()
        .id()
        .find(troop_id)
        .ok_or("Troop not found")?;

    if troop.owner_identity != identity {
        return Err("Not your troop".to_string());
    }

    ctx.db.placed_troop().id().delete(troop_id);
    Ok(())
}

// ─── Helpers ────────────────────────────────────────────────────────────────

fn generate_session_code(ctx: &ReducerContext) -> String {
    let ts = ctx.timestamp.to_micros_since_unix_epoch();
    let chars = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
    let mut code = String::with_capacity(6);
    let mut seed = ts as u64;
    for _ in 0..6 {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let idx = (seed >> 33) as usize % chars.len();
        code.push(chars[idx] as char);
    }
    code
}
