use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use super::resources::*;
use super::helpers::*;

pub fn expand_placeable_area(
    mut placeable_map: ResMut<PlaceableMap>,
    tile_storage_q: Query<&TileStorage, With<TilemapSize>>,
    tile_texture_q: Query<&TileTextureIndex>,
    tilemap_size_q: Query<&TilemapSize>,
) {
    let Some(tile_storage) = tile_storage_q.iter().next() else { return };
    let Some(map_size) = tilemap_size_q.iter().next() else { return };

    let mut newly_placeable = Vec::new();

    for x in 0..map_size.x {
        for y in 0..map_size.y {
            let tile_pos = TilePos { x, y };

            if let Some(tile_entity) = tile_storage.get(&tile_pos)
                && let Ok(texture) = tile_texture_q.get(tile_entity)
                && texture.0 >= 2
            {
                for dx in -2..=2 {
                    for dy in -2..=2 {
                        if dx == 0 && dy == 0 {
                            continue;
                        }

                        let nx = x as i32 + dx;
                        let ny = y as i32 + dy;

                        if nx >= 0 && nx < map_size.x as i32
                            && ny >= 0 && ny < map_size.y as i32
                        {
                            let neighbor_pos = TilePos {
                                x: nx as u32,
                                y: ny as u32
                            };

                            if !placeable_map.is_placeable(&neighbor_pos) {
                                newly_placeable.push(neighbor_pos);
                            }
                        }
                    }
                }
            }
        }
    }

    for pos in newly_placeable {
        placeable_map.mark_placeable(pos);
    }
}

pub fn update_placeable_indicators(
    placeable_map: Res<PlaceableMap>,
    tile_storage_q: Query<&TileStorage>,
    mut tile_texture_q: Query<&mut TileTextureIndex>,
    tilemap_size_q: Query<&TilemapSize>,
) {
    if !placeable_map.is_changed() {
        return;
    }

    let Some(tile_storage) = tile_storage_q.iter().next() else { return };
    let Some(map_size) = tilemap_size_q.iter().next() else { return };

    for x in 0..map_size.x {
        for y in 0..map_size.y {
            let tile_pos = TilePos { x, y };

            if let Some(tile_entity) = tile_storage.get(&tile_pos)
                && let Ok(mut texture) = tile_texture_q.get_mut(tile_entity)
            {
                if texture.0 >= 2 {
                    continue;
                }

                let is_placeable = placeable_map.is_placeable(&tile_pos);

                if is_placeable && texture.0 == 0 {
                    texture.0 = 1;
                } else if !is_placeable && texture.0 == 1 {
                    texture.0 = 0;
                }
            }
        }
    }
}

pub fn incremental_update_placeable_area(
    demolished_pos: TilePos,
    placeable_map: &mut PlaceableMap,
    tile_storage: &TileStorage,
    tile_texture_q: &Query<&mut TileTextureIndex>,
    map_size: &TilemapSize,
) {
    for dx in -4..=4 {
        for dy in -4..=4 {
            let nx = demolished_pos.x as i32 + dx;
            let ny = demolished_pos.y as i32 + dy;

            if nx >= 0 && nx < map_size.x as i32 && ny >= 0 && ny < map_size.y as i32 {
                let check_pos = TilePos { x: nx as u32, y: ny as u32 };

                if let Some(entity) = tile_storage.get(&check_pos)
                    && let Ok(texture) = tile_texture_q.get(entity)
                {
                    if texture.0 >= 2 {
                        continue;
                    }

                    let should_be_placeable = is_within_range_of_placed_tile(
                        &check_pos,
                        tile_storage,
                        tile_texture_q,
                        map_size,
                    );

                    if should_be_placeable {
                        placeable_map.mark_placeable(check_pos);
                    } else {
                        placeable_map.placeable_tiles.remove(&check_pos);
                    }
                }
            }
        }
    }

    info!("Incrementally updated placeable area around {:?}", demolished_pos);
}