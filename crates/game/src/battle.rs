use std::collections::HashMap;

use bevy::prelude::*;

use simulation_core::game_state::{AnimState, GameState};
use simulation_core::types::{GamePhase, PlayerId, SimUnitId};

use crate::components::*;
use crate::economy::ResolutionTimer;
use crate::recruitment_ui::RecruitmentPanel;
use crate::troop_spawner::UnitConfigRes;

const PROJECTILE_SCALE: f32 = 0.15;
const PROJECTILE_FPS: f32 = 10.0;
const FALLBACK_PROJECTILE: &str = "Units/Red Units/Archer/Arrow.png";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Resource, Default)]
pub enum BattlePhase {
    #[default]
    Placement,
    Battle,
    Resolution,
}

#[derive(Resource)]
pub struct BattleState {
    pub game_state: GameState,
    pub sim_to_entity: HashMap<SimUnitId, Entity>,
}

#[derive(Component)]
pub struct StartBattleButton;

#[derive(Component)]
pub struct BattleOverlayRoot;

pub fn setup_battle_ui(mut commands: Commands) {
    commands.init_resource::<BattlePhase>();

    commands
        .spawn((
            Button,
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(12.0),
                left: Val::Px(12.0),
                padding: UiRect::axes(Val::Px(24.0), Val::Px(12.0)),
                border_radius: BorderRadius::all(Val::Px(8.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.7, 0.15, 0.15)),
            StartBattleButton,
        ))
        .with_child((
            Text::new("START BATTLE"),
            TextFont {
                font_size: 18.0,
                ..default()
            },
            TextColor(Color::WHITE),
        ));
}

pub fn handle_start_battle_button(
    interaction_q: Query<&Interaction, (Changed<Interaction>, With<StartBattleButton>)>,
    mut phase: ResMut<BattlePhase>,
) {
    for interaction in &interaction_q {
        if *interaction == Interaction::Pressed && *phase == BattlePhase::Placement {
            *phase = BattlePhase::Battle;
        }
    }
}

pub fn init_battle(
    mut commands: Commands,
    phase: Res<BattlePhase>,
    config_res: Res<UnitConfigRes>,
    troops: Query<(Entity, &Owner, &TroopUnitId, &Children)>,
    members: Query<&GlobalTransform, With<FormationMember>>,
    mut button_q: Query<&mut Visibility, With<StartBattleButton>>,
    mut panel_q: Query<&mut Visibility, (With<RecruitmentPanel>, Without<StartBattleButton>)>,
) {
    if !phase.is_changed() || *phase != BattlePhase::Battle {
        return;
    }

    let mut game_state = GameState::new();
    let mut sim_to_entity: HashMap<SimUnitId, Entity> = HashMap::new();

    for (troop_entity, owner, troop_unit_id, children) in &troops {
        for child in children.iter() {
            let Ok(global_tf) = members.get(child) else {
                continue;
            };
            let pos = global_tf.translation();
            let sim_id = game_state.add_unit(
                &troop_unit_id.0,
                owner.0,
                pos.x,
                pos.y,
                &config_res.0,
            );
            sim_to_entity.insert(sim_id, child);

            commands.entity(child).insert((
                SimUnitLink(sim_id),
                BattleUnit,
                TroopUnitId(troop_unit_id.0.clone()),
            ));
            commands.entity(child).remove_parent_in_place();
            commands.entity(child).insert(
                Transform::from_xyz(pos.x, pos.y, 1.0)
                    .with_scale(global_tf.compute_transform().scale),
            );
        }

        commands.entity(troop_entity).remove::<Draggable>();
    }

    game_state.execute(
        simulation_core::game_state::Command::StartBattle,
        &config_res.0,
    );

    commands.insert_resource(BattleState {
        game_state,
        sim_to_entity,
    });

    for mut vis in button_q.iter_mut() {
        *vis = Visibility::Hidden;
    }
    for mut vis in panel_q.iter_mut() {
        *vis = Visibility::Hidden;
    }
}

pub fn tick_battle(
    phase: Res<BattlePhase>,
    time: Res<Time>,
    config_res: Res<UnitConfigRes>,
    mut battle_state: Option<ResMut<BattleState>>,
) {
    if *phase != BattlePhase::Battle {
        return;
    }
    let Some(ref mut state) = battle_state else {
        return;
    };
    state.game_state.tick(time.delta_secs(), &config_res.0);
}

pub fn sync_sim_to_bevy(
    phase: Res<BattlePhase>,
    battle_state: Option<Res<BattleState>>,
    mut units: Query<(&SimUnitLink, &mut Transform, &mut AnimationState)>,
) {
    if *phase != BattlePhase::Battle {
        return;
    }
    let Some(ref state) = battle_state else {
        return;
    };

    for (link, mut tf, mut anim) in &mut units {
        let Some(sim_unit) = state.game_state.units.iter().find(|u| u.id == link.0) else {
            continue;
        };
        tf.translation.x = sim_unit.x;
        tf.translation.y = sim_unit.y;

        let new_anim = match sim_unit.animation_state {
            AnimState::Idle => AnimationState::Idle,
            AnimState::Run => AnimationState::Run,
            AnimState::Attack => AnimationState::Attack,
            AnimState::Death => AnimationState::Death,
        };
        if *anim != new_anim {
            *anim = new_anim;
        }
    }
}

pub fn sync_projectiles_to_bevy(
    mut commands: Commands,
    phase: Res<BattlePhase>,
    battle_state: Option<Res<BattleState>>,
    config_res: Res<UnitConfigRes>,
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    existing: Query<(Entity, &ProjectileVisual)>,
) {
    if *phase != BattlePhase::Battle {
        return;
    }
    let Some(ref state) = battle_state else {
        return;
    };

    let active_ids: Vec<u64> = state.game_state.projectiles.iter().map(|p| p.id).collect();

    for (entity, pv) in &existing {
        if !active_ids.contains(&pv.sim_id) {
            commands.entity(entity).despawn();
        }
    }

    let existing_ids: Vec<u64> = existing.iter().map(|(_, pv)| pv.sim_id).collect();

    for proj in &state.game_state.projectiles {
        if existing_ids.contains(&proj.id) {
            continue;
        }

        let unit_def = config_res.0.find_by_id(&proj.source_def_id);
        let sprite_path: String = unit_def
            .and_then(|d| {
                if d.projectile_sprite_path.is_empty() {
                    None
                } else {
                    Some(d.projectile_sprite_path.clone())
                }
            })
            .unwrap_or_else(|| FALLBACK_PROJECTILE.to_string());

        let tex: Handle<Image> = asset_server.load(&sprite_path);
        let frame_count = unit_def.map(|d| d.projectile_frame_count).unwrap_or(1);
        let frame_size = unit_def
            .and_then(|d| d.projectile_frame_size)
            .unwrap_or([64, 64]);

        let pv = ProjectileVisual {
            sim_id: proj.id,
            frame_count,
        };

        if frame_count > 1 {
            let layout = TextureAtlasLayout::from_grid(
                UVec2::new(frame_size[0], frame_size[1]),
                frame_count as u32,
                1,
                None,
                None,
            );
            let layout_handle = layouts.add(layout);

            commands.spawn((
                Sprite {
                    image: tex,
                    texture_atlas: Some(TextureAtlas {
                        layout: layout_handle,
                        index: 0,
                    }),
                    ..default()
                },
                Transform::from_xyz(proj.x, proj.y, 5.0)
                    .with_scale(Vec3::splat(PROJECTILE_SCALE)),
                AnimationTimer(Timer::from_seconds(
                    1.0 / PROJECTILE_FPS,
                    TimerMode::Repeating,
                )),
                pv,
            ));
        } else {
            commands.spawn((
                Sprite {
                    image: tex,
                    ..default()
                },
                Transform::from_xyz(proj.x, proj.y, 5.0)
                    .with_scale(Vec3::splat(PROJECTILE_SCALE)),
                pv,
            ));
        }
    }
}

pub fn move_projectile_visuals(
    phase: Res<BattlePhase>,
    battle_state: Option<Res<BattleState>>,
    mut projectiles: Query<(&ProjectileVisual, &mut Transform)>,
) {
    if *phase != BattlePhase::Battle {
        return;
    }
    let Some(ref state) = battle_state else {
        return;
    };

    for (pv, mut tf) in &mut projectiles {
        if let Some(proj) = state.game_state.projectiles.iter().find(|p| p.id == pv.sim_id) {
            tf.translation.x = proj.x;
            tf.translation.y = proj.y;
        }
    }
}

pub fn animate_projectiles(
    time: Res<Time>,
    mut projectiles: Query<(&ProjectileVisual, &mut AnimationTimer, &mut Sprite)>,
) {
    for (pv, mut timer, mut sprite) in &mut projectiles {
        if pv.frame_count <= 1 {
            continue;
        }
        timer.0.tick(time.delta());
        if timer.0.just_finished() {
            if let Some(ref mut atlas) = sprite.texture_atlas {
                atlas.index = (atlas.index + 1) % pv.frame_count;
            }
        }
    }
}

pub fn check_resolution(
    mut commands: Commands,
    mut phase: ResMut<BattlePhase>,
    battle_state: Option<Res<BattleState>>,
) {
    if *phase != BattlePhase::Battle {
        return;
    }
    let Some(ref state) = battle_state else {
        return;
    };

    if state.game_state.phase != GamePhase::Resolution {
        return;
    }

    *phase = BattlePhase::Resolution;

    commands.insert_resource(ResolutionTimer(Timer::from_seconds(
        crate::economy::RESOLUTION_DELAY_SECS,
        TimerMode::Once,
    )));

    let player_alive = state
        .game_state
        .units
        .iter()
        .any(|u| u.is_alive && u.owner == PlayerId(0));
    let enemy_alive = state
        .game_state
        .units
        .iter()
        .any(|u| u.is_alive && u.owner == PlayerId(1));

    let (text, color) = match (player_alive, enemy_alive) {
        (true, false) => ("VICTORY!", Color::srgb(0.2, 1.0, 0.3)),
        (false, true) => ("DEFEAT!", Color::srgb(1.0, 0.2, 0.2)),
        _ => ("DRAW!", Color::srgb(1.0, 1.0, 0.3)),
    };

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
            BattleOverlayRoot,
        ))
        .with_child((
            Text::new(text),
            TextFont {
                font_size: 72.0,
                ..default()
            },
            TextColor(color),
        ));
}
