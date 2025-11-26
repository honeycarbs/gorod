use super::helpers::*;
use super::placeable_area::incremental_update_placeable_area;
use super::resources::*;
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

pub fn demolish_tile_on_click(
    mouse_button: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    cursor_pos: Res<CursorWorldPos>,
    mut placeable_map: ResMut<PlaceableMap>,
    tilemap_q: Query<(
        &TilemapSize,
        &TilemapGridSize,
        &TilemapTileSize,
        &TilemapType,
        &TileStorage,
        &Transform,
        &TilemapAnchor,
    )>,
    mut tile_texture_q: Query<&mut TileTextureIndex>,
) {
    if !mouse_button.just_pressed(MouseButton::Left) {
        return;
    }

    if !keyboard.pressed(KeyCode::ShiftLeft) && !keyboard.pressed(KeyCode::ShiftRight) {
        return;
    }

    for (map_size, grid_size, tile_size, map_type, tile_storage, map_transform, anchor) in
        tilemap_q.iter()
    {
        let cursor_in_map_pos = cursor_to_map_pos(cursor_pos.0, map_transform);

        if let Some(tile_pos) = TilePos::from_world_pos(
            &cursor_in_map_pos,
            map_size,
            grid_size,
            tile_size,
            map_type,
            anchor,
        ) && let Some(tile_entity) = tile_storage.get(&tile_pos)
        {
            let current_texture = if let Ok(texture) = tile_texture_q.get(tile_entity) {
                texture.0
            } else {
                return;
            };

            if current_texture >= 2 {
                let placed_tile_count = count_placed_tiles(tile_storage, &tile_texture_q, map_size);

                if placed_tile_count <= 1 {
                    warn!("Cannot demolish the last tile!");
                    return;
                }

                let should_be_placeable = is_within_range_of_placed_tile(
                    &tile_pos,
                    tile_storage,
                    &tile_texture_q,
                    map_size,
                );

                if let Ok(mut texture_index) = tile_texture_q.get_mut(tile_entity) {
                    if should_be_placeable {
                        texture_index.0 = 1;
                    } else {
                        texture_index.0 = 0;
                        placeable_map.placeable_tiles.remove(&tile_pos);
                    }

                    info!("Demolished tile at {:?}", tile_pos);

                    incremental_update_placeable_area(
                        tile_pos,
                        &mut placeable_map,
                        tile_storage,
                        &tile_texture_q,
                        map_size,
                    );
                }
            }
        }
    }
}
