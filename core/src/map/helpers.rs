use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::spatial::SpatialGrid;

const PLACEABLE_EXPANSION_RADIUS: i32 = 2;

pub fn cursor_to_map_pos(cursor_pos: Vec2, map_transform: &Transform) -> Vec2 {
    let cursor_pos = Vec4::from((cursor_pos, 0.0, 1.0));
    let cursor_in_map_pos = map_transform.to_matrix().inverse() * cursor_pos;
    cursor_in_map_pos.xy()
}

/// Convert a `TilePos` into the world-space center of that tile for a centered tilemap
pub fn tile_center_to_world(
    tile_pos: &TilePos,
    map_size: &TilemapSize,
    grid_size: &TilemapGridSize,
    map_transform: &Transform,
) -> Vec3 {
    let half_width = map_size.x as f32 * grid_size.x / 2.0;
    let half_height = map_size.y as f32 * grid_size.y / 2.0;

    let x = -half_width + (tile_pos.x as f32 + 0.5) * grid_size.x;
    let y = -half_height + (tile_pos.y as f32 + 0.5) * grid_size.y;

    map_transform.transform_point(Vec3::new(x, y, 0.0))
}

pub fn count_placed_tiles(
    tile_storage: &TileStorage,
    tile_texture_q: &Query<&mut TileTextureIndex>,
    map_size: &TilemapSize,
) -> u32 {
    let mut count = 0;

    for x in 0..map_size.x {
        for y in 0..map_size.y {
            let tile_pos = TilePos { x, y };

            if let Some(tile_entity) = tile_storage.get(&tile_pos)
                && let Ok(texture) = tile_texture_q.get(tile_entity)
                && texture.0 >= 2
            {
                count += 1;
            }
        }
    }

    count
}

pub fn is_within_range_of_placed_tile(tile_pos: &TilePos, spatial_grid: &SpatialGrid) -> bool {
    spatial_grid.has_building_in_radius(tile_pos, PLACEABLE_EXPANSION_RADIUS)
}
