use super::helpers::*;
use super::resources::*;
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

pub fn highlight_hovered_tile(
    mut commands: Commands,
    cursor_pos: Res<CursorWorldPos>,
    tilemap_q: Query<(
        &TilemapSize,
        &TilemapGridSize,
        &TilemapTileSize,
        &TilemapType,
        &TileStorage,
        &Transform,
        &TilemapAnchor,
    )>,
    highlighted_tiles_q: Query<Entity, With<HighlightedTile>>,
    mut tile_color_q: Query<&mut TileColor>,
) {
    for entity in highlighted_tiles_q.iter() {
        commands.entity(entity).remove::<HighlightedTile>();
        if let Ok(mut color) = tile_color_q.get_mut(entity) {
            color.0 = Color::WHITE;
        }
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
            commands.entity(tile_entity).insert(HighlightedTile);
            if let Ok(mut color) = tile_color_q.get_mut(tile_entity) {
                color.0 = Color::srgba(1.0, 1.0, 0.8, 1.0);
            }
        }
    }
}
