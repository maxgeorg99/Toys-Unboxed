use bevy::prelude::*;
use simulation_core::types::{PlayerId, SimUnitId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
pub enum AnimationState {
    Idle,
    Run,
    Attack,
    Death,
}

#[derive(Component)]
pub struct Owner(pub PlayerId);

#[derive(Component)]
pub struct BattleUnit;

#[derive(Component)]
pub struct ProjectileVisual {
    pub sim_id: u64,
    pub frame_count: usize,
}

#[derive(Debug, Clone)]
pub struct ClipData {
    pub texture: Handle<Image>,
    pub atlas_layout: Handle<TextureAtlasLayout>,
    pub frame_count: usize,
    pub fps: f32,
    pub looping: bool,
}

#[derive(Component, Clone)]
pub struct UnitAnimations {
    pub idle: ClipData,
    pub run: ClipData,
    pub attack: ClipData,
    pub death: ClipData,
}

impl UnitAnimations {
    pub fn get(&self, state: AnimationState) -> &ClipData {
        match state {
            AnimationState::Idle => &self.idle,
            AnimationState::Run => &self.run,
            AnimationState::Attack => &self.attack,
            AnimationState::Death => &self.death,
        }
    }
}

#[derive(Component)]
pub struct AnimationTimer(pub Timer);

#[derive(Component)]
pub struct SimUnitLink(pub SimUnitId);

#[derive(Component)]
pub struct FormationMember;

/// Marker: this entity can be picked up by the player.
#[derive(Component)]
pub struct Draggable;

/// Attached while the entity is being dragged; stores grab offset so the
/// sprite doesn't snap its centre to the cursor.
#[derive(Component)]
pub struct Dragging {
    pub offset: Vec2,
}

#[derive(Component)]
pub struct TroopUnitId(pub String);

#[derive(Component)]
pub struct Selected;
