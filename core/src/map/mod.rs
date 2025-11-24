use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use std::collections::HashSet;

#[derive(Resource, Default)]
pub struct CursorWorldPos(pub Vec2);

#[derive(Component)]
pub struct HighlightedTile;

#[derive(Resource)]
pub struct CurrentTileType {
    pub texture_index: u32,
}

impl Default for CurrentTileType {
    fn default() -> Self {
        Self {
            texture_index: 5,
        }
    }
}

pub struct TilePlacementPlugin;

impl Plugin for TilePlacementPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CursorWorldPos>()
            .init_resource::<CurrentTileType>()
            .init_resource::<PlaceableMap>()
            .add_systems(First, update_cursor_world_pos)
            .add_systems(Update, (
                highlight_hovered_tile,
                place_tile_on_click,
                expand_placeable_area,
                update_placeable_indicators,
            ).chain());
    }
}

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

fn cursor_to_map_pos(cursor_pos: Vec2, map_transform: &Transform) -> Vec2 {
    let cursor_pos = Vec4::from((cursor_pos, 0.0, 1.0));
    let cursor_in_map_pos = map_transform.to_matrix().inverse() * cursor_pos;
    cursor_in_map_pos.xy()
}

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
            color.0 = Color::WHITE; // Reset to default color
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
        )
            && let Some(tile_entity) = tile_storage.get(&tile_pos) {
                commands.entity(tile_entity).insert(HighlightedTile);
                if let Ok(mut color) = tile_color_q.get_mut(tile_entity) {
                    color.0 = Color::srgba(1.0, 1.0, 0.8, 1.0); // Light yellow highlight
                }
            }
    }
}

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
    if keyboard.just_pressed(KeyCode::Digit1) {
        current_tile_type.texture_index = 2; // Housing
        info!("Selected: Housing");
    } else if keyboard.just_pressed(KeyCode::Digit2) {
        current_tile_type.texture_index = 3; // Commercial
        info!("Selected: Commercial");
    } else if keyboard.just_pressed(KeyCode::Digit3) {
        current_tile_type.texture_index = 4; // Industry
        info!("Selected: Industry");
    } else if keyboard.just_pressed(KeyCode::Digit4) {
        current_tile_type.texture_index = 5; // Road
        info!("Selected: Road");
    }
}

#[derive(Resource, Default)]
pub struct PlaceableMap {
    pub placeable_tiles: HashSet<TilePos>,
}

impl PlaceableMap {
    pub fn is_placeable(&self, pos: &TilePos) -> bool {
        self.placeable_tiles.contains(pos)
    }

    pub fn mark_placeable(&mut self, pos: TilePos) {
        self.placeable_tiles.insert(pos);
    }
}

pub fn update_placeable_indicators(
    placeable_map: Res<PlaceableMap>,
    tile_storage_q: Query<&TileStorage>,
    mut tile_texture_q: Query<&mut TileTextureIndex>,
) {
    if !placeable_map.is_changed() {
        return;
    }

    for tile_storage in tile_storage_q.iter() {
        for tile_pos in placeable_map.placeable_tiles.iter() {
            if let Some(tile_entity) = tile_storage.get(tile_pos) {
                if let Ok(mut texture) = tile_texture_q.get_mut(tile_entity) {
                    // Only update if not placeable
                    if texture.0 == 0 {
                        texture.0 = 1;
                    }
                }
            }
        }
    }
}

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

            if let Some(tile_entity) = tile_storage.get(&tile_pos) {
                if let Ok(texture) = tile_texture_q.get(tile_entity) {
                    if texture.0 >= 2 {
                        for dx in -2..=2 {
                            for dy in -2..=2 {
                                if dx == 0 && dy == 0 {
                                    continue; // Skip center
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
        }
    }

    for pos in newly_placeable {
        placeable_map.mark_placeable(pos);
    }
}