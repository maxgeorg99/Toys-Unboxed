use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnitAnimationState {
    Idle,
    Run,
    Attack,
    Death,
}

#[derive(Debug, Clone)]
pub struct AnimationClip {
    pub texture: Handle<Image>,
    pub texture_atlas: Handle<TextureAtlasLayout>,
    pub frame_count: usize,
    pub fps: f32,
    pub looping: bool,       // idle/run loop, attack/death don't
}

#[derive(Debug, Clone, Asset, TypePath)]
pub struct UnitAnimationSet {
    pub sprite_size: Vec2,   // actual pixel size of one frame
    pub idle:   AnimationClip,
    pub run:    AnimationClip,
    pub attack: AnimationClip,
    pub death:  AnimationClip,
}

impl UnitAnimationSet {
    pub fn get(&self, state: UnitAnimationState) -> &AnimationClip {
        match state {
            UnitAnimationState::Idle   => &self.idle,
            UnitAnimationState::Run    => &self.run,
            UnitAnimationState::Attack => &self.attack,
            UnitAnimationState::Death  => &self.death,
        }
    }
}

fn main() {}