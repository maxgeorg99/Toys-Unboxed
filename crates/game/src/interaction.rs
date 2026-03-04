use bevy::prelude::*;

use crate::battle::BattlePhase;
use crate::components::{Draggable, Dragging, FormationMember, Selected};
use crate::placement_visuals;

const HALF_SIZE: f32 = 16.0;
const ROTATE_STEP: f32 = std::f32::consts::FRAC_PI_2; // 90 degrees

/// Convert the cursor's screen position to a world position, accounting for
/// camera pan / zoom.
fn cursor_world_pos(window: &Window, camera: &Camera, camera_gt: &GlobalTransform) -> Option<Vec2> {
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
    previously_selected: Query<Entity, With<Selected>>,
    mut commands: Commands,
    phase: Res<BattlePhase>,
) {
    if *phase != BattlePhase::Placement {
        return;
    }
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let Ok(window) = windows.single() else { return };
    let Ok((camera, cam_gt)) = camera_q.single() else {
        return;
    };
    let Some(world_pos) = cursor_world_pos(window, camera, cam_gt) else {
        return;
    };

    for (member_gt, child_of) in &members {
        let pos = member_gt.translation().truncate();
        let half = HALF_SIZE;

        if (world_pos.x - pos.x).abs() < half && (world_pos.y - pos.y).abs() < half {
            let parent = child_of.parent();
            if let Ok(parent_gt) = parents.get(parent) {
                let offset = world_pos - parent_gt.translation().truncate();
                for e in &previously_selected {
                    commands.entity(e).remove::<Selected>();
                }
                commands.entity(parent).insert((Dragging { offset }, Selected));
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
    let Ok((camera, cam_gt)) = camera_q.single() else {
        return;
    };
    let Some(world_pos) = cursor_world_pos(window, camera, cam_gt) else {
        return;
    };

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
    phase: Res<BattlePhase>,
) {
    if *phase != BattlePhase::Placement {
        return;
    }
    if !mouse.just_pressed(MouseButton::Middle) {
        return;
    }

    let Ok(window) = windows.single() else { return };
    let Ok((camera, cam_gt)) = camera_q.single() else {
        return;
    };
    let Some(world_pos) = cursor_world_pos(window, camera, cam_gt) else {
        return;
    };

    for (member_gt, child_of) in &members {
        let pos = member_gt.translation().truncate();
        let half = HALF_SIZE;

        if (world_pos.x - pos.x).abs() < half && (world_pos.y - pos.y).abs() < half {
            let parent = child_of.parent();
            if let Ok(mut transform) = troops.get_mut(parent) {
                transform.rotate_z(ROTATE_STEP);
                return;
            }
        }
    }
}

pub fn counter_rotate_sprites(
    troops: Query<(&Transform, &Children), With<Draggable>>,
    mut members: Query<&mut Transform, (With<FormationMember>, Without<Draggable>)>,
) {
    for (troop_transform, children) in &troops {
        let (_, _, angle) = troop_transform.rotation.to_euler(EulerRot::XYZ);
        let counter = Quat::from_rotation_z(-angle);

        for child in children.iter() {
            if let Ok(mut member_transform) = members.get_mut(child) {
                member_transform.rotation = counter;
            }
        }
    }
}

/// On left-button release, snap to the tile grid and stop dragging.
pub fn drag_end(
    mouse: Res<ButtonInput<MouseButton>>,
    mut dragged: Query<(Entity, &mut Transform), With<Dragging>>,
    mut commands: Commands,
) {
    if !mouse.just_released(MouseButton::Left) {
        return;
    }

    for (entity, mut transform) in &mut dragged {
        let snapped = placement_visuals::snap_to_grid(transform.translation.truncate());
        transform.translation.x = snapped.x;
        transform.translation.y = snapped.y;
        commands.entity(entity).remove::<Dragging>();
    }
}

pub fn deselect_on_empty_click(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    members: Query<&GlobalTransform, With<FormationMember>>,
    selected: Query<Entity, With<Selected>>,
    ui_interactions: Query<&Interaction, With<Node>>,
    mut commands: Commands,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    if ui_interactions.iter().any(|i| *i != Interaction::None) {
        return;
    }

    let Ok(window) = windows.single() else { return };
    let Ok((camera, cam_gt)) = camera_q.single() else {
        return;
    };
    let Some(world_pos) = cursor_world_pos(window, camera, cam_gt) else {
        return;
    };

    let hit = members.iter().any(|gt| {
        let pos = gt.translation().truncate();
        (world_pos.x - pos.x).abs() < HALF_SIZE && (world_pos.y - pos.y).abs() < HALF_SIZE
    });

    if !hit {
        for e in &selected {
            commands.entity(e).remove::<Selected>();
        }
    }
}
