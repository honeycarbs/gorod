use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use super::resources::*;
use super::helpers::*;

pub fn place_tile_on_click(
    mouse_button: Res<ButtonInput<MouseButton>>,
    cursor_pos: Res<CursorWorldPos>,
    current_tile_type: Res<CurrentTileType>,
    placeable_map: Res<PlaceableMap>,
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
        )
            && let Some(existing_tile_entity) = tile_storage.get(&tile_pos)
            && let Ok(mut texture_index) = tile_texture_q.get_mut(existing_tile_entity)
        {
            if !placeable_map.is_placeable(&tile_pos) {
                warn!("Cannot place here - tile not placeable!");
                return;
            }

            if texture_index.0 != 1 {
                warn!("Cannot place here - tile already occupied!");
                return;
            }

            texture_index.0 = current_tile_type.texture_index;
            info!("Set tile at {:?} to texture index {}", tile_pos, current_tile_type.texture_index);
        }
    }
}

pub fn change_tile_type(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut current_tile_type: ResMut<CurrentTileType>,
) {
    if keyboard.just_pressed(KeyCode::KeyR) {
        current_tile_type.texture_index = 2;
        info!("Selected: Residential/Housing");
    } else if keyboard.just_pressed(KeyCode::KeyC) {
        current_tile_type.texture_index = 3;
        info!("Selected: Commercial");
    } else if keyboard.just_pressed(KeyCode::KeyI) {
        current_tile_type.texture_index = 4;
        info!("Selected: Industry");
    } else if keyboard.just_pressed(KeyCode::KeyO) {
        current_tile_type.texture_index = 5;
        info!("Selected: Road");
    }
}