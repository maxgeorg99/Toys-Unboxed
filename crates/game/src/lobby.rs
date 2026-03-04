use bevy::input::keyboard::Key;
use bevy::prelude::*;
use spacetimedb_sdk::Table;

use crate::module_bindings::{
    SessionStatus, SessionTableAccess, SessionPlayerTableAccess, UserTableAccess,
    create_session_reducer::create_session as CreateSessionReducer,
    join_session_reducer::join_session as JoinSessionReducer,
    leave_session_reducer::leave_session as LeaveSessionReducer,
    start_session_reducer::start_session as StartSessionReducer,
};
use crate::networking::SpacetimeDB;

#[derive(Resource, Default)]
pub struct LobbyState {
    pub join_code_input: String,
    pub session_code: Option<String>,
    pub in_session: bool,
    pub is_host: bool,
    pub status_message: String,
}

#[derive(Component)]
pub struct LobbyUI;

#[derive(Component)]
pub struct BrowserPanel;

#[derive(Component)]
pub struct InLobbyPanel;

#[derive(Component)]
pub struct CreateSessionButton;

#[derive(Component)]
pub struct JoinByCodeButton;

#[derive(Component)]
pub struct LobbyBackButton;

#[derive(Component)]
pub struct StartGameButton;

#[derive(Component)]
pub struct LeaveLobbyButton;

#[derive(Component)]
pub struct JoinCodeText;

#[derive(Component)]
pub struct SessionCodeDisplay;

#[derive(Component)]
pub struct PlayerListText;

#[derive(Component)]
pub struct StatusText;

pub fn setup_lobby(mut commands: Commands) {
    commands.init_resource::<LobbyState>();

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(20.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.08, 0.08, 0.12)),
            LobbyUI,
        ))
        .with_children(|root| {
            root.spawn((
                Text::new("LOBBY"),
                TextFont { font_size: 48.0, ..default() },
                TextColor(Color::srgb(1.0, 0.9, 0.4)),
            ));

            root.spawn((
                Text::new(""),
                TextFont { font_size: 16.0, ..default() },
                TextColor(Color::srgb(1.0, 0.5, 0.5)),
                StatusText,
            ));

            // Browser panel
            root.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(12.0),
                    ..default()
                },
                BrowserPanel,
            ))
            .with_children(|panel| {
                spawn_button(panel, "CREATE SESSION", CreateSessionButton);

                panel.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(8.0),
                    ..default()
                })
                .with_children(|row| {
                    row.spawn((
                        Node {
                            padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
                            border_radius: BorderRadius::all(Val::Px(6.0)),
                            min_width: Val::Px(120.0),
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.15, 0.15, 0.2)),
                    ))
                    .with_child((
                        Text::new("______"),
                        TextFont { font_size: 20.0, ..default() },
                        TextColor(Color::srgb(0.8, 0.8, 0.8)),
                        JoinCodeText,
                    ));

                    spawn_button(row, "JOIN", JoinByCodeButton);
                });

                spawn_button(panel, "BACK", LobbyBackButton);
            });

            // In-lobby panel
            root.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(12.0),
                    display: Display::None,
                    ..default()
                },
                InLobbyPanel,
            ))
            .with_children(|panel| {
                panel.spawn((
                    Text::new("Code: ------"),
                    TextFont { font_size: 24.0, ..default() },
                    TextColor(Color::srgb(0.6, 1.0, 0.6)),
                    SessionCodeDisplay,
                ));

                panel.spawn((
                    Text::new("Players:\n  (loading...)"),
                    TextFont { font_size: 18.0, ..default() },
                    TextColor(Color::srgb(0.8, 0.8, 0.8)),
                    PlayerListText,
                ));

                spawn_button(panel, "START GAME", StartGameButton);
                spawn_button(panel, "LEAVE", LeaveLobbyButton);
            });
        });
}

fn spawn_button(parent: &mut ChildSpawnerCommands, label: &str, marker: impl Component) {
    parent
        .spawn((
            Button,
            Node {
                padding: UiRect::axes(Val::Px(24.0), Val::Px(10.0)),
                border_radius: BorderRadius::all(Val::Px(8.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.25, 0.25, 0.35)),
            marker,
        ))
        .with_child((
            Text::new(label),
            TextFont { font_size: 18.0, ..default() },
            TextColor(Color::WHITE),
        ));
}

pub fn cleanup_lobby(
    mut commands: Commands,
    query: Query<Entity, With<LobbyUI>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<LobbyState>();
}

pub fn handle_create_session(
    interaction_q: Query<&Interaction, (Changed<Interaction>, With<CreateSessionButton>)>,
    stdb: Option<SpacetimeDB>,
    mut lobby: ResMut<LobbyState>,
) {
    let Some(stdb) = stdb else { return };
    for interaction in &interaction_q {
        if *interaction == Interaction::Pressed {
            if let Err(e) = stdb.reducers().create_session() {
                lobby.status_message = format!("Failed: {e}");
            }
        }
    }
}

pub fn handle_join_by_code(
    interaction_q: Query<&Interaction, (Changed<Interaction>, With<JoinByCodeButton>)>,
    stdb: Option<SpacetimeDB>,
    mut lobby: ResMut<LobbyState>,
) {
    let Some(stdb) = stdb else { return };
    for interaction in &interaction_q {
        if *interaction == Interaction::Pressed {
            let code = lobby.join_code_input.clone();
            if code.len() != 6 {
                lobby.status_message = "Code must be 6 characters".into();
                return;
            }
            if let Err(e) = stdb.reducers().join_session(code) {
                lobby.status_message = format!("Failed: {e}");
            }
        }
    }
}

pub fn handle_join_code_input(
    mut events: MessageReader<bevy::input::keyboard::KeyboardInput>,
    mut lobby: ResMut<LobbyState>,
    mut text_q: Query<&mut Text, With<JoinCodeText>>,
) {
    if lobby.in_session {
        return;
    }
    for event in events.read() {
        if !event.state.is_pressed() {
            continue;
        }
        match event.key_code {
            KeyCode::Backspace => {
                lobby.join_code_input.pop();
            }
            _ => {
                if let Key::Character(ref ch) = event.logical_key {
                    if lobby.join_code_input.len() < 6 {
                        lobby.join_code_input.push_str(&ch.to_uppercase());
                    }
                }
            }
        }
    }
    if let Ok(mut t) = text_q.single_mut() {
        let display: String = format!(
            "{}{}",
            lobby.join_code_input,
            "_".repeat(6 - lobby.join_code_input.len().min(6))
        );
        **t = display;
    }
}

pub fn poll_session_status(
    stdb: Option<SpacetimeDB>,
    mut lobby: ResMut<LobbyState>,
    mut next_state: ResMut<NextState<super::AppState>>,
) {
    let Some(stdb) = stdb else { return };
    let Some(my_identity) = stdb.try_identity() else { return };

    let my_user = stdb.db().user().identity().find(&my_identity);
    let session_id = my_user.as_ref().and_then(|u| u.session_id);

    if let Some(sid) = session_id {
        lobby.in_session = true;

        let session = stdb.db().session().iter().find(|s| s.session_id == sid);

        if let Some(ref sess) = session {
            lobby.session_code = Some(sess.session_code.clone());

            if sess.status == SessionStatus::InProgress {
                next_state.set(super::AppState::InGame);
                return;
            }
        }

        let is_host = stdb
            .db()
            .session_player()
            .iter()
            .any(|sp| sp.session_id == sid && sp.player_identity == my_identity && sp.is_host);
        lobby.is_host = is_host;
    } else {
        lobby.in_session = false;
        lobby.session_code = None;
        lobby.is_host = false;
    }
}

pub fn handle_start_game(
    interaction_q: Query<&Interaction, (Changed<Interaction>, With<StartGameButton>)>,
    stdb: Option<SpacetimeDB>,
    lobby: Res<LobbyState>,
    mut status: ResMut<LobbyState>,
) {
    let Some(stdb) = stdb else { return };
    if !lobby.is_host {
        return;
    }
    for interaction in &interaction_q {
        if *interaction == Interaction::Pressed {
            if let Err(e) = stdb.reducers().start_session() {
                status.status_message = format!("Failed: {e}");
            }
        }
    }
}

pub fn handle_leave_session(
    interaction_q: Query<&Interaction, (Changed<Interaction>, With<LeaveLobbyButton>)>,
    stdb: Option<SpacetimeDB>,
    mut lobby: ResMut<LobbyState>,
) {
    let Some(stdb) = stdb else { return };
    for interaction in &interaction_q {
        if *interaction == Interaction::Pressed {
            if let Err(e) = stdb.reducers().leave_session() {
                lobby.status_message = format!("Failed: {e}");
            }
        }
    }
}

pub fn handle_lobby_back(
    interaction_q: Query<&Interaction, (Changed<Interaction>, With<LobbyBackButton>)>,
    mut next_state: ResMut<NextState<super::AppState>>,
    lobby: Res<LobbyState>,
) {
    if lobby.in_session {
        return;
    }
    for interaction in &interaction_q {
        if *interaction == Interaction::Pressed {
            next_state.set(super::AppState::MainMenu);
        }
    }
}

pub fn update_panel_visibility(
    lobby: Res<LobbyState>,
    mut browser_q: Query<&mut Node, (With<BrowserPanel>, Without<InLobbyPanel>)>,
    mut in_lobby_q: Query<&mut Node, (With<InLobbyPanel>, Without<BrowserPanel>)>,
) {
    if let Ok(mut node) = browser_q.single_mut() {
        node.display = if lobby.in_session { Display::None } else { Display::Flex };
    }
    if let Ok(mut node) = in_lobby_q.single_mut() {
        node.display = if lobby.in_session { Display::Flex } else { Display::None };
    }
}

pub fn update_in_lobby_view(
    stdb: Option<SpacetimeDB>,
    lobby: Res<LobbyState>,
    mut code_q: Query<&mut Text, (With<SessionCodeDisplay>, Without<PlayerListText>, Without<StatusText>)>,
    mut player_q: Query<&mut Text, (With<PlayerListText>, Without<SessionCodeDisplay>, Without<StatusText>)>,
    mut status_q: Query<&mut Text, (With<StatusText>, Without<SessionCodeDisplay>, Without<PlayerListText>)>,
    mut start_vis_q: Query<&mut Visibility, With<StartGameButton>>,
) {
    if let Ok(mut t) = status_q.single_mut() {
        **t = lobby.status_message.clone();
    }

    if !lobby.in_session {
        return;
    }

    if let (Some(code), Ok(mut t)) = (&lobby.session_code, code_q.single_mut()) {
        **t = format!("Code: {code}");
    }

    let Some(stdb) = stdb else { return };
    let Some(my_identity) = stdb.try_identity() else { return };
    let my_user = stdb.db().user().identity().find(&my_identity);
    let Some(sid) = my_user.as_ref().and_then(|u| u.session_id) else { return };

    let players: Vec<String> = stdb
        .db()
        .session_player()
        .iter()
        .filter(|sp| sp.session_id == sid)
        .map(|sp| {
            let name = stdb
                .db()
                .user()
                .identity()
                .find(&sp.player_identity)
                .and_then(|u| u.name.clone())
                .unwrap_or_else(|| format!("Player {}", sp.slot));
            let host_tag = if sp.is_host { " (Host)" } else { "" };
            format!("  Slot {}: {}{}", sp.slot, name, host_tag)
        })
        .collect();

    if let Ok(mut t) = player_q.single_mut() {
        **t = format!("Players:\n{}", players.join("\n"));
    }

    for mut vis in start_vis_q.iter_mut() {
        *vis = if lobby.is_host { Visibility::Visible } else { Visibility::Hidden };
    }
}
