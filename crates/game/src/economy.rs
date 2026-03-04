use std::collections::HashSet;

use bevy::prelude::*;

use simulation_core::types::PlayerId;

use crate::battle::{BattleOverlayRoot, BattlePhase, BattleState, StartBattleButton};
use crate::components::*;
use crate::recruitment_ui::{RecruitUiState, RecruitmentPanel};
use crate::troop_spawner::{choose_and_spawn_enemies, SpawnTroopEvent, UnitConfigRes};

pub const STARTING_GOLD: u32 = 10;
pub const INCOME_PER_ROUND: u32 = 8;
pub const SLOTS_PER_ROUND: u32 = 2;
pub const RESOLUTION_DELAY_SECS: f32 = 2.5;
pub const BASE_ENEMY_COUNT: u32 = 2;

#[derive(Resource)]
pub struct Economy {
    pub gold: u32,
}

#[derive(Resource)]
pub struct RoundState {
    pub round: u32,
}

#[derive(Resource)]
pub struct RecruitSlots {
    pub remaining: u32,
}

#[derive(Resource)]
pub struct ResolutionTimer(pub Timer);

#[derive(Component)]
pub struct GoldText;

#[derive(Component)]
pub struct RoundText;

#[derive(Component)]
pub struct SlotsText;

pub fn setup_economy(mut commands: Commands) {
    commands.insert_resource(Economy { gold: STARTING_GOLD });
    commands.insert_resource(RoundState { round: 1 });
    commands.insert_resource(RecruitSlots { remaining: SLOTS_PER_ROUND });
}

const HUD_ICON_SZ: f32 = 20.0;

pub fn setup_economy_hud(mut commands: Commands, asset_server: Res<AssetServer>) {
    let gold_icon = asset_server.load("Icons/Gold_Icon.png");

    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            top: Val::Px(8.0),
            left: Val::Percent(50.0),
            margin: UiRect::left(Val::Px(-200.0)),
            width: Val::Px(400.0),
            height: Val::Px(36.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            column_gap: Val::Px(24.0),
            border_radius: BorderRadius::all(Val::Px(8.0)),
            ..default()
        })
        .insert(BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)))
        .with_children(|bar| {
            bar.spawn((
                Text::new("Round 1"),
                TextFont { font_size: 16.0, ..default() },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                RoundText,
            ));

            bar.spawn(Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(4.0),
                ..default()
            })
            .with_children(|gold_row| {
                gold_row.spawn((
                    ImageNode::new(gold_icon),
                    Node {
                        width: Val::Px(HUD_ICON_SZ),
                        height: Val::Px(HUD_ICON_SZ),
                        ..default()
                    },
                ));
                gold_row.spawn((
                    Text::new(format!("{STARTING_GOLD}")),
                    TextFont { font_size: 16.0, ..default() },
                    TextColor(Color::srgb(1.0, 0.85, 0.3)),
                    GoldText,
                ));
            });

            bar.spawn((
                Text::new(format!("Slots: {SLOTS_PER_ROUND}/{SLOTS_PER_ROUND}")),
                TextFont { font_size: 16.0, ..default() },
                TextColor(Color::srgb(0.6, 0.85, 1.0)),
                SlotsText,
            ));
        });
}

pub fn update_economy_hud(
    economy: Res<Economy>,
    round: Res<RoundState>,
    slots: Res<RecruitSlots>,
    mut gold_q: Query<&mut Text, (With<GoldText>, Without<RoundText>, Without<SlotsText>)>,
    mut round_q: Query<&mut Text, (With<RoundText>, Without<GoldText>, Without<SlotsText>)>,
    mut slots_q: Query<&mut Text, (With<SlotsText>, Without<GoldText>, Without<RoundText>)>,
) {
    if let Ok(mut t) = gold_q.single_mut() {
        **t = format!("{}", economy.gold);
    }
    if let Ok(mut t) = round_q.single_mut() {
        **t = format!("Round {}", round.round);
    }
    if let Ok(mut t) = slots_q.single_mut() {
        **t = format!("Slots: {}/{SLOTS_PER_ROUND}", slots.remaining);
    }
}

pub fn advance_round(
    mut commands: Commands,
    time: Res<Time>,
    phase: Res<BattlePhase>,
    mut timer: Option<ResMut<ResolutionTimer>>,
    battle_state: Option<Res<BattleState>>,
    config_res: Res<UnitConfigRes>,
    mut economy: ResMut<Economy>,
    mut round: ResMut<RoundState>,
    mut slots: ResMut<RecruitSlots>,
    mut spawn_events: MessageWriter<SpawnTroopEvent>,
    to_despawn: Query<Entity, Or<(With<BattleUnit>, With<ProjectileVisual>, With<BattleOverlayRoot>)>>,
    troop_parents: Query<Entity, (With<TroopUnitId>, With<Owner>, Without<BattleUnit>)>,
    mut vis_q: ParamSet<(
        Query<&mut Visibility, With<StartBattleButton>>,
        Query<&mut Visibility, With<RecruitmentPanel>>,
    )>,
    mut ui_state: ResMut<RecruitUiState>,
) {
    if *phase != BattlePhase::Resolution {
        return;
    }
    let Some(ref mut timer) = timer else { return };
    timer.0.tick(time.delta());
    if !timer.0.just_finished() {
        return;
    }

    let surviving_def_ids: Vec<String> = battle_state
        .as_ref()
        .map(|bs| {
            let mut seen = HashSet::new();
            bs.game_state
                .units
                .iter()
                .filter(|u| u.is_alive && u.owner == PlayerId(0))
                .filter(|u| seen.insert(u.def_id.clone()))
                .map(|u| u.def_id.clone())
                .collect()
        })
        .unwrap_or_default();

    for entity in &to_despawn {
        commands.entity(entity).despawn();
    }
    for entity in &troop_parents {
        commands.entity(entity).despawn();
    }

    commands.remove_resource::<BattleState>();
    commands.remove_resource::<ResolutionTimer>();

    for (i, def_id) in surviving_def_ids.iter().enumerate() {
        let x = -60.0 + (i as f32) * 40.0;
        spawn_events.write(SpawnTroopEvent {
            unit_id: def_id.clone(),
            world_pos: Vec2::new(x, -100.0),
            owner: PlayerId(0),
            remote_troop_id: None,
        });
    }

    let enemy_count = (BASE_ENEMY_COUNT + round.round / 3) as usize;
    choose_and_spawn_enemies(&config_res.0, enemy_count, &mut spawn_events);

    round.round += 1;
    economy.gold += INCOME_PER_ROUND;
    slots.remaining = SLOTS_PER_ROUND;

    commands.insert_resource(BattlePhase::Placement);

    for mut vis in vis_q.p0().iter_mut() {
        *vis = Visibility::Visible;
    }
    for mut vis in vis_q.p1().iter_mut() {
        *vis = Visibility::Visible;
    }
    ui_state.grid_built = false;
}
