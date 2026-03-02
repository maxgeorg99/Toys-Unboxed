use bevy::prelude::*;

use crate::components::{Draggable, Dragging, FormationMember};

const HALF_SIZE: f32 = 16.0;
const ROTATE_STEP: f32 = std::f32::consts::FRAC_PI_2; // 90 degrees

/// Convert the cursor's screen position to a world position, accounting for
/// camera pan / zoom.
fn cursor_world_pos(
    window: &Window,
    camera: &Camera,
    camera_gt: &GlobalTransform,
) -> Option<Vec2> {
    let cursor = window.cursor_position()?;
    camera.viewport_to_world_2d(camera_gt, cursor).ok()
}

/// On left-click, hit-test formation member sprites and start dragging their
/// parent troop entity.
pub fn drag_start(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    members: Query<(&GlobalTransform, &ChildOf), With<FormationMember>>,
    parents: Query<&GlobalTransform, (With<Draggable>, Without<Dragging>)>,
    mut commands: Commands,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let Ok(window) = windows.single() else { return };
    let Ok((camera, cam_gt)) = camera_q.single() else { return };
    let Some(world_pos) = cursor_world_pos(window, camera, cam_gt) else { return };

    for (member_gt, child_of) in &members {
        let pos = member_gt.translation().truncate();
        let scale = member_gt.compute_transform().scale.truncate();
        let half = HALF_SIZE * scale;

        if (world_pos.x - pos.x).abs() < half.x && (world_pos.y - pos.y).abs() < half.y {
            let parent = child_of.parent();
            // Only drag if parent is draggable and not already being dragged
            if let Ok(parent_gt) = parents.get(parent) {
                let offset = world_pos - parent_gt.translation().truncate();
                commands.entity(parent).insert(Dragging { offset });
                return;
            }
        }
    }
}

/// Each frame, move dragged troop entities to follow the cursor.
pub fn drag_move(
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    mut dragged: Query<(&Dragging, &mut Transform)>,
) {
    let Ok(window) = windows.single() else { return };
    let Ok((camera, cam_gt)) = camera_q.single() else { return };
    let Some(world_pos) = cursor_world_pos(window, camera, cam_gt) else { return };

    for (dragging, mut transform) in &mut dragged {
        let target = world_pos - dragging.offset;
        transform.translation.x = target.x;
        transform.translation.y = target.y;
    }
}

/// On middle-click, hit-test formation members and rotate their parent troop
/// by 90 degrees.
pub fn troop_rotate(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    members: Query<(&GlobalTransform, &ChildOf), With<FormationMember>>,
    mut troops: Query<&mut Transform, With<Draggable>>,
) {
    if !mouse.just_pressed(MouseButton::Middle) {
        return;
    }

    let Ok(window) = windows.single() else { return };
    let Ok((camera, cam_gt)) = camera_q.single() else { return };
    let Some(world_pos) = cursor_world_pos(window, camera, cam_gt) else { return };

    for (member_gt, child_of) in &members {
        let pos = member_gt.translation().truncate();
        let scale = member_gt.compute_transform().scale.truncate();
        let half = HALF_SIZE * scale;

        if (world_pos.x - pos.x).abs() < half.x && (world_pos.y - pos.y).abs() < half.y {
            let parent = child_of.parent();
            if let Ok(mut transform) = troops.get_mut(parent) {
                transform.rotate_z(ROTATE_STEP);
                return;
            }
        }
    }
}

/// On left-button release, stop dragging all entities.
pub fn drag_end(
    mouse: Res<ButtonInput<MouseButton>>,
    dragged: Query<Entity, With<Dragging>>,
    mut commands: Commands,
) {
    if !mouse.just_released(MouseButton::Left) {
        return;
    }

    for entity in &dragged {
        commands.entity(entity).remove::<Dragging>();
    }
}
