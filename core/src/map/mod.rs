use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

/// Resource to track the current cursor position in world space
#[derive(Resource, Default)]
pub struct CursorWorldPos(pub Vec2);

/// Component to mark the currently highlighted tile
#[derive(Component)]
pub struct HighlightedTile;

/// Resource to store the current tile type that will be placed
#[derive(Resource, Default)]
pub struct CurrentTileType {
    pub texture_index: u32,
}

/// Plugin that adds tile placement functionality
pub struct TilePlacementPlugin;

impl Plugin for TilePlacementPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CursorWorldPos>()
            .init_resource::<CurrentTileType>()
            .add_systems(First, update_cursor_world_pos)
            .add_systems(Update, (highlight_hovered_tile, place_tile_on_click).chain());
    }
}

/// System to update cursor position from screen space to world space
pub fn update_cursor_world_pos(
    camera_q: Query<(&GlobalTransform, &Camera)>,
    mut cursor_moved_events: MessageReader<CursorMoved>,
    mut cursor_pos: ResMut<CursorWorldPos>,
) {
    for cursor_moved in cursor_moved_events.read() {
        for (cam_transform, cam) in camera_q.iter() {
            if let Ok(world_pos) = cam.viewport_to_world_2d(cam_transform, cursor_moved.position) {
                cursor_pos.0 = world_pos;
            }
        }
    }
}

/// Helper function to convert cursor position to map space
fn cursor_to_map_pos(cursor_pos: Vec2, map_transform: &Transform) -> Vec2 {
    let cursor_pos = Vec4::from((cursor_pos, 0.0, 1.0));
    let cursor_in_map_pos = map_transform.to_matrix().inverse() * cursor_pos;
    cursor_in_map_pos.xy()
}

/// System to highlight the tile under the cursor
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
    // Remove highlight from previously highlighted tiles
    for entity in highlighted_tiles_q.iter() {
        commands.entity(entity).remove::<HighlightedTile>();
        if let Ok(mut color) = tile_color_q.get_mut(entity) {
            color.0 = Color::WHITE; // Reset to default color
        }
    }

    // Find and highlight the tile under the cursor
    for (map_size, grid_size, tile_size, map_type, tile_storage, map_transform, anchor) in
        tilemap_q.iter()
    {
        let cursor_in_map_pos = cursor_to_map_pos(cursor_pos.0, map_transform);

        // Convert map position to tile position
        if let Some(tile_pos) = TilePos::from_world_pos(
            &cursor_in_map_pos,
            map_size,
            grid_size,
            tile_size,
            map_type,
            anchor,
        ) {
            if let Some(tile_entity) = tile_storage.get(&tile_pos) {
                // Highlight the tile
                commands.entity(tile_entity).insert(HighlightedTile);
                if let Ok(mut color) = tile_color_q.get_mut(tile_entity) {
                    color.0 = Color::srgba(1.0, 1.0, 0.8, 1.0); // Light yellow highlight
                }
            }
        }
    }
}

/// System to place tiles on mouse click
pub fn place_tile_on_click(
    mouse_button: Res<ButtonInput<MouseButton>>,
    cursor_pos: Res<CursorWorldPos>,
    current_tile_type: Res<CurrentTileType>,
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

        // Get the tile position from cursor
        if let Some(tile_pos) = TilePos::from_world_pos(
            &cursor_in_map_pos,
            map_size,
            grid_size,
            tile_size,
            map_type,
            anchor,
        ) {
            // Check if a tile already exists at this position
            if let Some(existing_tile_entity) = tile_storage.get(&tile_pos) {
                // Update existing tile's texture
                if let Ok(mut texture_index) = tile_texture_q.get_mut(existing_tile_entity) {
                    texture_index.0 = current_tile_type.texture_index;
                    info!("Set tile at {:?} to texture index {}", tile_pos, current_tile_type.texture_index);
                }
            } else {
                warn!("No tile found at position {:?}", tile_pos);
            }
        }
    }
}

pub fn change_tile_type(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut current_tile_type: ResMut<CurrentTileType>,
) {
    if keyboard.just_pressed(KeyCode::Digit1) {
        current_tile_type.texture_index = 0;
    } else if keyboard.just_pressed(KeyCode::Digit2) {
        current_tile_type.texture_index = 1;
    } 
}