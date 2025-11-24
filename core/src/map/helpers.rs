use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

pub fn cursor_to_map_pos(cursor_pos: Vec2, map_transform: &Transform) -> Vec2 {
    let cursor_pos = Vec4::from((cursor_pos, 0.0, 1.0));
    let cursor_in_map_pos = map_transform.to_matrix().inverse() * cursor_pos;
    cursor_in_map_pos.xy()
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

pub fn is_within_range_of_placed_tile(
    tile_pos: &TilePos,
    tile_storage: &TileStorage,
    tile_texture_q: &Query<&mut TileTextureIndex>,
    map_size: &TilemapSize,
) -> bool {
    for dx in -2..=2 {
        for dy in -2..=2 {
            if dx == 0 && dy == 0 {
                continue;
            }

            let nx = tile_pos.x as i32 + dx;
            let ny = tile_pos.y as i32 + dy;

            if nx >= 0 && nx < map_size.x as i32 && ny >= 0 && ny < map_size.y as i32 {
                let neighbor_pos = TilePos {
                    x: nx as u32,
                    y: ny as u32,
                };

                if let Some(neighbor_entity) = tile_storage.get(&neighbor_pos)
                    && let Ok(texture) = tile_texture_q.get(neighbor_entity)
                    && texture.0 >= 2
                {
                    return true;
                }
            }
        }
    }

    false
}