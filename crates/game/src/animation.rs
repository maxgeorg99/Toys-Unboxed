use bevy::prelude::*;

use crate::components::{AnimationState, AnimationTimer, UnitAnimations};

/// Advance sprite animation frames based on timer.
pub fn animate_sprites(
    time: Res<Time>,
    mut query: Query<(
        &mut AnimationTimer,
        &mut Sprite,
        &AnimationState,
        &UnitAnimations,
    )>,
) {
    for (mut timer, mut sprite, anim_state, animations) in &mut query {
        timer.0.tick(time.delta());
        if timer.0.just_finished() {
            let clip = animations.get(*anim_state);
            if let Some(atlas) = &mut sprite.texture_atlas {
                let next = atlas.index + 1;
                if next >= clip.frame_count {
                    if clip.looping {
                        atlas.index = 0;
                    }
                    // Non-looping: stay on last frame
                } else {
                    atlas.index = next;
                }
            }
        }
    }
}

/// When AnimationState changes, swap texture + atlas layout to match the new clip.
pub fn on_animation_state_changed(
    mut query: Query<
        (&AnimationState, &UnitAnimations, &mut Sprite, &mut AnimationTimer),
        Changed<AnimationState>,
    >,
) {
    for (anim_state, animations, mut sprite, mut timer) in &mut query {
        let clip = animations.get(*anim_state);

        sprite.image = clip.texture.clone();
        if let Some(atlas) = &mut sprite.texture_atlas {
            atlas.layout = clip.atlas_layout.clone();
            atlas.index = 0;
        }

        timer.0 = Timer::from_seconds(1.0 / clip.fps, TimerMode::Repeating);
    }
}
