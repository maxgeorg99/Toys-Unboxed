use bevy::prelude::*;

use simulation_core::types::{AttackType, DefenseType};

use crate::troop_spawner::{SpawnTroopEvent, UnitConfigRes};

// ─── Layout constants ───────────────────────────────────────────────────────

// Card grid: 8 cols × 72px + 7 gaps × 5px = 611px
const CARD_PX: f32 = 72.0;
const CARD_GAP: f32 = 5.0;
const GRID_COLS: usize = 8;
const GRID_ROWS: usize = 4;
const GRID_WIDTH: f32 = GRID_COLS as f32 * CARD_PX + (GRID_COLS - 1) as f32 * CARD_GAP;
const GRID_HEIGHT: f32 = GRID_ROWS as f32 * CARD_PX + (GRID_ROWS - 1) as f32 * CARD_GAP + 18.0;

// Filter button sizes
const FILTER_BTN: f32 = 40.0;

// ─── Components ─────────────────────────────────────────────────────────────

#[derive(Component)]
pub struct RecruitmentPanel;

#[derive(Component)]
pub struct UnitGridArea;

#[derive(Component)]
pub struct RecruitButton {
    pub unit_id: String,
}

#[derive(Component)]
pub struct FilterAttackButton {
    pub attack_type: AttackType,
}

#[derive(Component)]
pub struct FilterDefenseButton {
    pub defense_type: DefenseType,
}

// ─── Resources ──────────────────────────────────────────────────────────────

#[derive(Resource, Default)]
pub struct RecruitUnitFilter {
    pub attack: Option<AttackType>,
    pub defense: Option<DefenseType>,
}

#[derive(Resource)]
pub struct RecruitUiState {
    pub grid_built: bool,
}

impl Default for RecruitUiState {
    fn default() -> Self {
        Self { grid_built: false }
    }
}

// ─── Setup ──────────────────────────────────────────────────────────────────

pub fn setup_recruitment_ui(mut commands: Commands) {
    commands.init_resource::<RecruitUnitFilter>();
    commands.init_resource::<RecruitUiState>();

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(0.0),
                right: Val::Px(0.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Stretch,
                padding: UiRect {
                    top: Val::Px(8.0),
                    bottom: Val::Px(6.0),
                    left: Val::Px(12.0),
                    right: Val::Px(12.0),
                },
                row_gap: Val::Px(6.0),
                border_radius: BorderRadius::top(Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.06, 0.09, 0.16, 0.95)),
            RecruitmentPanel,
        ))
        .with_children(|panel| {
            // ── Header ──────────────────────────────────────────
            panel.spawn((
                Text::new("RECRUIT TROOPS"),
                TextFont {
                    font_size: 15.0,
                    ..default()
                },
                TextColor(Color::srgb(0.95, 0.82, 0.4)),
            ));

            // ── Separator ───────────────────────────────────────
            panel.spawn((
                Node {
                    height: Val::Px(1.0),
                    width: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.12)),
            ));

            // ── Unit grid + filter column ───────────────────────
            panel
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::FlexStart,
                    column_gap: Val::Px(8.0),
                    ..default()
                })
                .with_children(|row| {
                    // Unit card grid area (filled by build_unit_grid)
                    row.spawn((
                        Node {
                            flex_direction: FlexDirection::Row,
                            flex_wrap: FlexWrap::Wrap,
                            column_gap: Val::Px(CARD_GAP),
                            row_gap: Val::Px(CARD_GAP),
                            justify_content: JustifyContent::FlexStart,
                            align_content: AlignContent::FlexStart,
                            width: Val::Px(GRID_WIDTH),
                            height: Val::Px(GRID_HEIGHT),
                            ..default()
                        },
                        UnitGridArea,
                    ));

                    // ── Filter panel: ATK | DEF columns ─────────
                    row.spawn((
                        Node {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::FlexStart,
                            column_gap: Val::Px(6.0),
                            padding: UiRect::all(Val::Px(6.0)),
                            border_radius: BorderRadius::all(Val::Px(6.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.04)),
                    ))
                    .with_children(|filter_panel| {
                        // ATK column
                        filter_panel
                            .spawn(Node {
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::Center,
                                row_gap: Val::Px(4.0),
                                ..default()
                            })
                            .with_children(|col| {
                                col.spawn((
                                    Text::new("ATK"),
                                    TextFont {
                                        font_size: 9.0,
                                        ..default()
                                    },
                                    TextColor(Color::srgba(0.7, 0.7, 0.7, 0.6)),
                                ));
                                for atk in [
                                    AttackType::Blunt,
                                    AttackType::Pierce,
                                    AttackType::Magic,
                                    AttackType::Divine,
                                ] {
                                    col.spawn((
                                        Button,
                                        Node {
                                            width: Val::Px(FILTER_BTN),
                                            height: Val::Px(FILTER_BTN),
                                            border_radius: BorderRadius::all(Val::Px(6.0)),
                                            justify_content: JustifyContent::Center,
                                            align_items: AlignItems::Center,
                                            ..default()
                                        },
                                        BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.08)),
                                        FilterAttackButton { attack_type: atk },
                                    ))
                                    .with_child((
                                        Text::new(attack_label(atk)),
                                        TextFont {
                                            font_size: 10.0,
                                            ..default()
                                        },
                                        TextColor(Color::WHITE),
                                    ));
                                }
                            });

                        // Vertical separator
                        filter_panel.spawn((
                            Node {
                                width: Val::Px(1.0),
                                height: Val::Percent(100.0),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.12)),
                        ));

                        // DEF column
                        filter_panel
                            .spawn(Node {
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::Center,
                                row_gap: Val::Px(4.0),
                                ..default()
                            })
                            .with_children(|col| {
                                col.spawn((
                                    Text::new("DEF"),
                                    TextFont {
                                        font_size: 9.0,
                                        ..default()
                                    },
                                    TextColor(Color::srgba(0.7, 0.7, 0.7, 0.6)),
                                ));
                                for def in [
                                    DefenseType::Armor,
                                    DefenseType::Agility,
                                    DefenseType::Mystical,
                                ] {
                                    col.spawn((
                                        Button,
                                        Node {
                                            width: Val::Px(FILTER_BTN),
                                            height: Val::Px(FILTER_BTN),
                                            border_radius: BorderRadius::all(Val::Px(6.0)),
                                            justify_content: JustifyContent::Center,
                                            align_items: AlignItems::Center,
                                            ..default()
                                        },
                                        BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.08)),
                                        FilterDefenseButton { defense_type: def },
                                    ))
                                    .with_child((
                                        Text::new(defense_label(def)),
                                        TextFont {
                                            font_size: 10.0,
                                            ..default()
                                        },
                                        TextColor(Color::WHITE),
                                    ));
                                }
                            });
                    });
                });
        });
}

// ─── Build / rebuild unit card grid ─────────────────────────────────────────

pub fn build_unit_grid(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    config_res: Res<UnitConfigRes>,
    filter: Res<RecruitUnitFilter>,
    grid_area: Query<Entity, With<UnitGridArea>>,
    mut ui_state: ResMut<RecruitUiState>,
) {
    if ui_state.grid_built {
        return;
    }
    let Ok(grid_entity) = grid_area.single() else {
        return;
    };

    const EXCLUDED: &[&str] = &["dragon", "knight", "mage", "dwarf"];

    let mut cards: Vec<_> = config_res
        .0
        .units
        .iter()
        .filter(|u| !u.avatar_path.is_empty() && !EXCLUDED.contains(&u.id.as_str()))
        .collect();

    // Apply active filters
    if let Some(atk) = filter.attack {
        cards.retain(|u| u.attack_type == atk);
    }
    if let Some(def) = filter.defense {
        cards.retain(|u| u.defense_type == def);
    }

    cards.sort_by(|a, b| a.meat_cost.cmp(&b.meat_cost).then(a.name.cmp(&b.name)));

    // Pre-load avatar handles outside the closure
    let render_data: Vec<_> = cards
        .iter()
        .map(|unit| {
            let avatar_img = asset_server.load(&unit.avatar_path);
            (
                unit.name.clone(),
                unit.id.clone(),
                unit.meat_cost,
                unit.attack_type,
                unit.defense_type,
                avatar_img,
            )
        })
        .collect();

    commands.entity(grid_entity).despawn_children();
    commands.entity(grid_entity).with_children(|grid| {
        for (name, unit_id, meat_cost, _atk, _def, avatar_img) in &render_data {
            spawn_unit_card(grid, name, unit_id, avatar_img.clone());
        }
    });

    ui_state.grid_built = true;
}

fn spawn_unit_card(
    parent: &mut ChildSpawnerCommands,
    unit_name_full: &str,
    unit_id: &str,
    avatar_img: Handle<Image>,
) {
    parent
        .spawn((
            Button,
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(4.0)),
                row_gap: Val::Px(3.0),
                width: Val::Px(CARD_PX),
                border_radius: BorderRadius::all(Val::Px(6.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.12, 0.28, 0.18, 0.92)),
            RecruitButton {
                unit_id: unit_id.to_string(),
            },
        ))
        .with_children(|card| {
            // Avatar container (relative, for overlaid badges)
            card.spawn(Node {
                width: Val::Px(52.0),
                height: Val::Px(52.0),
                position_type: PositionType::Relative,
                ..default()
            })
            .with_children(|av| {
                av.spawn((
                    ImageNode::new(avatar_img),
                    Node {
                        width: Val::Px(52.0),
                        height: Val::Px(52.0),
                        ..default()
                    },
                ));
            });

            // Unit name
            card.spawn((
                Text::new(short_name(unit_name_full)),
                TextFont {
                    font_size: 9.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
            ));
        });
}

fn short_name(name: &str) -> String {
    if name.len() <= 10 {
        name.to_string()
    } else if let Some(pos) = name[..10].rfind(' ') {
        name[..pos].to_string()
    } else {
        format!("{}.", &name[..9])
    }
}

fn attack_label(atk: AttackType) -> &'static str {
    match atk {
        AttackType::Blunt => "BLT",
        AttackType::Pierce => "PRC",
        AttackType::Magic => "MAG",
        AttackType::Divine => "DIV",
    }
}

fn defense_label(def: DefenseType) -> &'static str {
    match def {
        DefenseType::Armor => "ARM",
        DefenseType::Agility => "AGI",
        DefenseType::Mystical => "MYS",
    }
}

// ─── Filter toggle ──────────────────────────────────────────────────────────

pub fn handle_filter_buttons(
    attack_btns: Query<(&Interaction, &FilterAttackButton), Changed<Interaction>>,
    defense_btns: Query<(&Interaction, &FilterDefenseButton), Changed<Interaction>>,
    mut filter: ResMut<RecruitUnitFilter>,
    mut ui_state: ResMut<RecruitUiState>,
) {
    for (interaction, btn) in attack_btns.iter() {
        if *interaction == Interaction::Pressed {
            filter.attack = if filter.attack == Some(btn.attack_type) {
                None
            } else {
                Some(btn.attack_type)
            };
            ui_state.grid_built = false;
        }
    }
    for (interaction, btn) in defense_btns.iter() {
        if *interaction == Interaction::Pressed {
            filter.defense = if filter.defense == Some(btn.defense_type) {
                None
            } else {
                Some(btn.defense_type)
            };
            ui_state.grid_built = false;
        }
    }
}

// ─── Filter button highlight ────────────────────────────────────────────────

pub fn highlight_active_filters(
    filter: Res<RecruitUnitFilter>,
    mut atk_btns: Query<(&FilterAttackButton, &mut BackgroundColor)>,
    mut def_btns: Query<(&FilterDefenseButton, &mut BackgroundColor), Without<FilterAttackButton>>,
) {
    let active_color = Color::srgba(0.4, 0.6, 1.0, 0.35);
    let inactive_color = Color::srgba(1.0, 1.0, 1.0, 0.08);

    for (btn, mut bg) in atk_btns.iter_mut() {
        *bg = if filter.attack == Some(btn.attack_type) {
            BackgroundColor(active_color)
        } else {
            BackgroundColor(inactive_color)
        };
    }
    for (btn, mut bg) in def_btns.iter_mut() {
        *bg = if filter.defense == Some(btn.defense_type) {
            BackgroundColor(active_color)
        } else {
            BackgroundColor(inactive_color)
        };
    }
}

// ─── Recruit button interaction ─────────────────────────────────────────────

pub fn handle_recruit_buttons(
    interaction_q: Query<(&Interaction, &RecruitButton), Changed<Interaction>>,
    camera_q: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    windows: Query<&Window>,
    mut spawn_events: MessageWriter<SpawnTroopEvent>,
) {
    for (interaction, button) in &interaction_q {
        if *interaction != Interaction::Pressed {
            continue;
        }

        // Spawn at the centre of the current camera view
        let world_pos = camera_centre(&camera_q, &windows).unwrap_or(Vec2::ZERO);

        spawn_events.write(SpawnTroopEvent {
            unit_id: button.unit_id.clone(),
            world_pos,
        });
    }
}

fn camera_centre(
    camera_q: &Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    windows: &Query<&Window>,
) -> Option<Vec2> {
    let (camera, cam_gt) = camera_q.single().ok()?;
    let window = windows.single().ok()?;
    let center = Vec2::new(window.width() / 2.0, window.height() / 2.0);
    camera.viewport_to_world_2d(cam_gt, center).ok()
}
