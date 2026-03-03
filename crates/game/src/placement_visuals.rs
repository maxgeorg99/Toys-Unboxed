use bevy::prelude::*;

use crate::components::{Dragging, FormationMember};

// Map constants matching box-map.tmx
pub const TILE_SIZE: f32 = 16.0;
const MAP_COLS: u32 = 30;
const MAP_ROWS: u32 = 40;

// bevy_ecs_tiled (TilemapAnchor::None) places tile centers at (col*16, row*16).
// Grid boundaries (tile edges) are therefore offset by half a tile.
const GRID_ORIGIN: Vec2 = Vec2::new(-8.0, -8.0);

/// Snap a world position to the nearest tile center.
pub fn snap_to_grid(pos: Vec2) -> Vec2 {
    Vec2::new(
        (pos.x / TILE_SIZE).round() * TILE_SIZE,
        (pos.y / TILE_SIZE).round() * TILE_SIZE,
    )
}

/// Draws tile-aligned grid lines over the map to help with troop placement.
pub fn draw_grid_overlay(mut gizmos: Gizmos) {
    let color = Color::srgba(1.0, 1.0, 1.0, 0.12);
    let x_min = GRID_ORIGIN.x;
    let x_max = GRID_ORIGIN.x + MAP_COLS as f32 * TILE_SIZE;
    let y_min = GRID_ORIGIN.y;
    let y_max = GRID_ORIGIN.y + MAP_ROWS as f32 * TILE_SIZE;

    for col in 0..=MAP_COLS {
        let x = GRID_ORIGIN.x + col as f32 * TILE_SIZE;
        gizmos.line_2d(Vec2::new(x, y_min), Vec2::new(x, y_max), color);
    }

    for row in 0..=MAP_ROWS {
        let y = GRID_ORIGIN.y + row as f32 * TILE_SIZE;
        gizmos.line_2d(Vec2::new(x_min, y), Vec2::new(x_max, y), color);
    }
}

/// Draws a green rectangle on each tile a dragged troop's members will occupy.
pub fn draw_drag_indicators(
    mut gizmos: Gizmos,
    members: Query<(&GlobalTransform, &ChildOf), With<FormationMember>>,
    dragged: Query<(), With<Dragging>>,
) {
    let color = Color::srgba(0.2, 0.85, 0.3, 0.35);
    let half_size = Vec2::splat(TILE_SIZE / 2.0);

    for (gt, child_of) in &members {
        if dragged.get(child_of.parent()).is_ok() {
            let pos = gt.translation().truncate();
            let snapped = snap_to_grid(pos);
            gizmos.rect_2d(snapped, half_size, color);
        }
    }
}
