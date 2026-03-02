use bevy::prelude::*;

use simulation_core::formation::Formation;
use simulation_core::types::SimUnitId;
use simulation_core::unit_data::UnitsConfig;

use crate::components::*;

const UNIT_TOML: &str = include_str!("../../../assets/units.toml");
const SKULL_ID: &str = "skull";
const SPRITE_SCALE: f32 = 0.15;
const FORMATION_SPACING: f32 = 32.0;
const IDLE_FPS: f32 = 6.0;
const RUN_FPS: f32 = 8.0;
const ATTACK_FPS: f32 = 10.0;
const DEATH_FPS: f32 = 8.0;

fn make_atlas_layout(
    frame_size: UVec2,
    frame_count: usize,
    layouts: &mut Assets<TextureAtlasLayout>,
) -> Handle<TextureAtlasLayout> {
    let layout = TextureAtlasLayout::from_grid(frame_size, frame_count as u32, 1, None, None);
    layouts.add(layout)
}

pub fn spawn_skull_troop(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let config = UnitsConfig::load_from_str(UNIT_TOML).expect("failed to parse units.toml");
    let skull = config.find_by_id(SKULL_ID).expect("skull not found in units.toml");

    let frame_size = UVec2::new(skull.frame_size[0], skull.frame_size[1]);

    // Death frame size may differ from normal frame size
    let death_size = skull
        .death_frame_size
        .map(|s| UVec2::new(s[0], s[1]))
        .unwrap_or(frame_size);

    // Load textures
    let idle_tex: Handle<Image> = asset_server.load(&skull.idle_sprite_path);
    let run_tex: Handle<Image> = asset_server.load(&skull.sprite_path);
    let attack_tex: Handle<Image> = asset_server.load(&skull.attack_sprite_path);
    let death_tex: Handle<Image> = asset_server.load(&skull.death_sprite_path);

    // Create atlas layouts
    let idle_layout = make_atlas_layout(frame_size, skull.idle_frame_count, &mut texture_atlas_layouts);
    let run_layout = make_atlas_layout(frame_size, skull.frame_count, &mut texture_atlas_layouts);
    let attack_layout = make_atlas_layout(frame_size, skull.attack_frame_count, &mut texture_atlas_layouts);
    let death_layout = make_atlas_layout(death_size, skull.death_frame_count, &mut texture_atlas_layouts);

    let unit_animations = UnitAnimations {
        idle: ClipData {
            texture: idle_tex.clone(),
            atlas_layout: idle_layout.clone(),
            frame_count: skull.idle_frame_count,
            fps: IDLE_FPS,
            looping: true,
        },
        run: ClipData {
            texture: run_tex,
            atlas_layout: run_layout,
            frame_count: skull.frame_count,
            fps: RUN_FPS,
            looping: true,
        },
        attack: ClipData {
            texture: attack_tex,
            atlas_layout: attack_layout,
            frame_count: skull.attack_frame_count,
            fps: ATTACK_FPS,
            looping: false,
        },
        death: ClipData {
            texture: death_tex,
            atlas_layout: death_layout,
            frame_count: skull.death_frame_count,
            fps: DEATH_FPS,
            looping: false,
        },
    };

    let formation = Formation::new(skull.troops_width, skull.troops_height, FORMATION_SPACING);

    commands
        .spawn((
            Draggable,
            Transform::default(),
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
