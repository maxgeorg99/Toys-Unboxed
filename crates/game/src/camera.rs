use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;

const PAN_SPEED: f32 = 200.0;
const MIN_ZOOM: f32 = 0.25;
const MAX_ZOOM: f32 = 3.0;

pub fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

/// WASD / arrow-key panning.
pub fn camera_pan(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut camera_q: Query<&mut Transform, With<Camera2d>>,
) {
    let mut dir = Vec2::ZERO;

    if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft) {
        dir.x -= 1.0;
    }
    if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) {
        dir.x += 1.0;
    }
    if keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp) {
        dir.y += 1.0;
    }
    if keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown) {
        dir.y -= 1.0;
    }

    if dir == Vec2::ZERO {
        return;
    }

    dir = dir.normalize();
    let delta = dir * PAN_SPEED * time.delta_secs();

    for mut transform in &mut camera_q {
        transform.translation.x += delta.x;
        transform.translation.y += delta.y;
    }
}

/// Scroll-wheel zoom via `Projection::Orthographic` scale.
pub fn camera_zoom(
    mut scroll_events: MessageReader<MouseWheel>,
    mut camera_q: Query<&mut Projection, With<Camera2d>>,
) {
    let scroll: f32 = scroll_events.read().map(|e| e.y).sum();
    if scroll == 0.0 {
        return;
    }

    let factor = 1.0 - scroll * 0.1;

    for mut proj in &mut camera_q {
        if let Projection::Orthographic(ref mut ortho) = *proj {
            ortho.scale = (ortho.scale * factor).clamp(MIN_ZOOM, MAX_ZOOM);
        }
    }
}
