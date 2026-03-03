use bevy::prelude::*;
use bevy::sprite::Anchor;

use simulation_core::formation::Formation;
use simulation_core::types::SimUnitId;
use simulation_core::unit_data::{UnitDef, UnitsConfig};

use crate::components::*;

const UNIT_TOML: &str = include_str!("../../../assets/units.toml");
const SPRITE_SCALE: f32 = 0.15;
const FORMATION_SPACING: f32 = 16.0;
// Shift the anchor below-center so the character art (which sits in the lower
// portion of the frame) aligns with the tile center.  Tweak Y to taste.
const SPRITE_ANCHOR: Vec2 = Vec2::new(0.0, -0.25);
const IDLE_FPS: f32 = 6.0;
const RUN_FPS: f32 = 8.0;
const ATTACK_FPS: f32 = 10.0;
const DEATH_FPS: f32 = 8.0;

/// Message fired to request spawning a troop formation at a world position.
#[derive(Message)]
pub struct SpawnTroopEvent {
    pub unit_id: String,
    pub world_pos: Vec2,
}

/// Insert UnitsConfig as a Bevy resource on startup.
pub fn init_units_config(mut commands: Commands) {
    let config = UnitsConfig::load_from_str(UNIT_TOML).expect("failed to parse units.toml");
    commands.insert_resource(UnitConfigRes(config));
}

/// Wrapper so we can use UnitsConfig as a Bevy Resource.
#[derive(Resource)]
pub struct UnitConfigRes(pub UnitsConfig);

fn make_atlas_layout(
    frame_size: UVec2,
    frame_count: usize,
    layouts: &mut Assets<TextureAtlasLayout>,
) -> Handle<TextureAtlasLayout> {
    let layout = TextureAtlasLayout::from_grid(frame_size, frame_count as u32, 1, None, None);
    layouts.add(layout)
}

fn build_unit_animations(
    unit: &UnitDef,
    asset_server: &AssetServer,
    layouts: &mut Assets<TextureAtlasLayout>,
) -> (UnitAnimations, Handle<Image>, Handle<TextureAtlasLayout>) {
    let frame_size = UVec2::new(unit.frame_size[0], unit.frame_size[1]);
    let death_size = unit
        .death_frame_size
        .map(|s| UVec2::new(s[0], s[1]))
        .unwrap_or(frame_size);

    let idle_tex: Handle<Image> = asset_server.load(&unit.idle_sprite_path);
    let run_tex: Handle<Image> = asset_server.load(&unit.sprite_path);
    let attack_tex: Handle<Image> = asset_server.load(&unit.attack_sprite_path);
    let death_tex: Handle<Image> = asset_server.load(&unit.death_sprite_path);

    let idle_layout = make_atlas_layout(frame_size, unit.idle_frame_count, layouts);
    let run_layout = make_atlas_layout(frame_size, unit.frame_count, layouts);
    let attack_layout = make_atlas_layout(frame_size, unit.attack_frame_count, layouts);
    let death_layout = make_atlas_layout(death_size, unit.death_frame_count, layouts);

    let anims = UnitAnimations {
        idle: ClipData {
            texture: idle_tex.clone(),
            atlas_layout: idle_layout.clone(),
            frame_count: unit.idle_frame_count,
            fps: IDLE_FPS,
            looping: true,
        },
        run: ClipData {
            texture: run_tex,
            atlas_layout: run_layout,
            frame_count: unit.frame_count,
            fps: RUN_FPS,
            looping: true,
        },
        attack: ClipData {
            texture: attack_tex,
            atlas_layout: attack_layout,
            frame_count: unit.attack_frame_count,
            fps: ATTACK_FPS,
            looping: false,
        },
        death: ClipData {
            texture: death_tex,
            atlas_layout: death_layout,
            frame_count: unit.death_frame_count,
            fps: DEATH_FPS,
            looping: false,
        },
    };

    (anims, idle_tex, idle_layout)
}

/// System that handles SpawnTroopEvent to spawn troop formations.
pub fn handle_spawn_troop_events(
    mut events: MessageReader<SpawnTroopEvent>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    config_res: Res<UnitConfigRes>,
) {
    for event in events.read() {
        let Some(unit) = config_res.0.find_by_id(&event.unit_id) else {
            warn!("Unit '{}' not found in units.toml", event.unit_id);
            continue;
        };

        let (unit_animations, idle_tex, idle_layout) =
            build_unit_animations(unit, &asset_server, &mut texture_atlas_layouts);

        let formation = Formation::new(unit.troops_width, unit.troops_height, FORMATION_SPACING);

        commands
            .spawn((
                Draggable,
                Transform::from_xyz(event.world_pos.x, event.world_pos.y, 0.0),
                Visibility::default(),
            ))
            .with_children(|parent| {
                for (i, (ox, oy)) in formation.positions().iter().enumerate() {
                    parent.spawn((
                        Sprite {
                            image: idle_tex.clone(),
                            texture_atlas: Some(TextureAtlas {
                                layout: idle_layout.clone(),
                                index: 0,
                            }),
                            ..default()
                        },
                        Anchor(SPRITE_ANCHOR),
                        Transform::from_xyz(*ox, *oy, 1.0)
                            .with_scale(Vec3::splat(SPRITE_SCALE)),
                        AnimationState::Idle,
                        AnimationTimer(Timer::from_seconds(
                            1.0 / IDLE_FPS,
                            TimerMode::Repeating,
                        )),
                        unit_animations.clone(),
                        SimUnitLink(SimUnitId(i as u64)),
                        FormationMember,
                    ));
                }
            });
    }
}

/// Startup system: spawns the initial skull troop.
pub fn spawn_skull_troop(mut events: MessageWriter<SpawnTroopEvent>) {
    events.write(SpawnTroopEvent {
        unit_id: "skull".to_string(),
        world_pos: Vec2::ZERO,
    });
}
