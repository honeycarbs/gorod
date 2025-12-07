use super::helpers::*;
use super::resources::*;
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use bevy_image::TextureAtlas;
use crate::budget::BuildingType;

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

/// Show a semi-transparent preview of the currently selected building under the cursor
#[allow(clippy::too_many_arguments)]
pub fn update_road_hover_preview(
    mut commands: Commands,
    cursor_pos: Res<CursorWorldPos>,
    current_tile_type: Res<CurrentTileType>,
    current_road_variant: Res<CurrentRoadVariant>,
    current_residential_variant: Res<CurrentResidentialVariant>,
    current_commercial_variant: Res<CurrentCommercialVariant>,
    current_industry_variant: Res<CurrentIndustryVariant>,
    road_atlas: Option<Res<RoadAtlas>>,
    residential_atlas: Option<Res<ResidentialBuildingAtlas>>,
    commercial_atlas: Option<Res<CommercialBuildingAtlas>>,
    industry_atlas: Option<Res<IndustryBuildingAtlas>>,
    tile_preview_atlas: Option<Res<TilePreviewAtlas>>,
    tilemap_q: Query<(
        &TilemapSize,
        &TilemapGridSize,
        &TilemapTileSize,
        &TilemapType,
        &TileStorage,
        &Transform,
        &TilemapAnchor,
    )>,
    preview_q: Query<Entity, With<RoadHoverPreview>>,
) {
    // Only show preview while some building type is active
    let active_type = BuildingType::from_texture_index(current_tile_type.texture_index);

    if active_type.is_none() {
        for entity in preview_q.iter() {
            commands.entity(entity).despawn();
        }
        return;
    }

    // Find the tile under the cursor and its world-space center
    let mut target_world_and_grid: Option<(Vec3, Vec2)> = None;

    for (map_size, grid_size, tile_size, map_type, _tile_storage, map_transform, anchor) in
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
        ) {
            let center = tile_center_to_world(&tile_pos, map_size, grid_size, map_transform);
            let cell_size = Vec2::new(grid_size.x, grid_size.y);
            target_world_and_grid = Some((center, cell_size));
            break;
        }
    }

    let Some((world_pos, _cell_size)) = target_world_and_grid else {
        for entity in preview_q.iter() {
            commands.entity(entity).despawn();
        }
        return;
    };

    // Clear any existing preview entity (there should be at most one)
    for entity in preview_q.iter() {
        commands.entity(entity).despawn();
    }

    match active_type.unwrap() {
        BuildingType::Road => {
            let Some(road_atlas) = road_atlas else {
                return;
            };

            let variants = road_atlas.variants.max(1);
            let variant_index = (current_road_variant.index as usize) % variants;

            let mut sprite = Sprite::from_atlas_image(
                road_atlas.texture.clone(),
                TextureAtlas {
                    layout: road_atlas.layout.clone(),
                    index: variant_index,
                },
            );
            // 30% opacity so the user can see both the tile and the preview
            sprite.color = Color::srgba(1.0, 1.0, 1.0, 0.3);

            commands.spawn((
                sprite,
                Transform::from_xyz(world_pos.x, world_pos.y, 8.0),
                RoadHoverPreview,
            ));
        }
        BuildingType::Residential => {
            let Some(tile_preview_atlas) = tile_preview_atlas else {
                return;
            };
            let Some(residential_atlas) = residential_atlas else {
                return;
            };

            // Tinted base tile preview (residential color)
            let mut tile_sprite = Sprite::from_atlas_image(
                tile_preview_atlas.texture.clone(),
                TextureAtlas {
                    layout: tile_preview_atlas.layout.clone(),
                    index: 2, // residential tile index
                },
            );
            tile_sprite.color = Color::srgba(
                193.0 / 255.0,
                231.0 / 255.0,
                110.0 / 255.0,
                0.2,
            ); // #c1e76e at ~20% opacity

            commands.spawn((
                tile_sprite,
                Transform::from_xyz(world_pos.x, world_pos.y, 7.5),
                RoadHoverPreview,
            ));

            // Preview selected residential building variant a few pixels above the tile
            let variants = residential_atlas.variants.max(1);
            let variant_index =
                (current_residential_variant.index as usize) % variants;
            let y_offset = 6.0;

            let building_sprite = Sprite::from_atlas_image(
                residential_atlas.texture.clone(),
                TextureAtlas {
                    layout: residential_atlas.layout.clone(),
                    index: variant_index,
                },
            );

            commands.spawn((
                building_sprite,
                Transform::from_xyz(world_pos.x, world_pos.y + y_offset, 10.0),
                RoadHoverPreview,
            ));
        }
        BuildingType::Commercial => {
            let Some(tile_preview_atlas) = tile_preview_atlas else {
                return;
            };
            let Some(commercial_atlas) = commercial_atlas else {
                return;
            };

            // Tinted base tile preview (commercial color)
            let mut tile_sprite = Sprite::from_atlas_image(
                tile_preview_atlas.texture.clone(),
                TextureAtlas {
                    layout: tile_preview_atlas.layout.clone(),
                    index: 3, // commercial tile index
                },
            );
            tile_sprite.color = Color::srgba(
                123.0 / 255.0,
                194.0 / 255.0,
                212.0 / 255.0,
                0.2,
            ); // #7bc2d4 at ~20% opacity

            commands.spawn((
                tile_sprite,
                Transform::from_xyz(world_pos.x, world_pos.y, 7.5),
                RoadHoverPreview,
            ));

            // Preview selected commercial building variant a few pixels above the tile
            let variants = commercial_atlas.variants.max(1);
            let variant_index =
                (current_commercial_variant.index as usize) % variants;
            let y_offset = 6.0;

            let building_sprite = Sprite::from_atlas_image(
                commercial_atlas.texture.clone(),
                TextureAtlas {
                    layout: commercial_atlas.layout.clone(),
                    index: variant_index,
                },
            );

            commands.spawn((
                building_sprite,
                Transform::from_xyz(world_pos.x, world_pos.y + y_offset, 10.0),
                RoadHoverPreview,
            ));
        }
        BuildingType::Industry => {
            let Some(tile_preview_atlas) = tile_preview_atlas else {
                return;
            };
            let Some(industry_atlas) = industry_atlas else {
                return;
            };

            // Tinted base tile preview
            let mut tile_sprite = Sprite::from_atlas_image(
                tile_preview_atlas.texture.clone(),
                TextureAtlas {
                    layout: tile_preview_atlas.layout.clone(),
                    index: 4, // industry tile index
                },
            );
            tile_sprite.color = Color::srgba(
                123.0 / 255.0,
                194.0 / 255.0,
                212.0 / 255.0,
                0.2,
            ); // #7bc2d4 at ~20% opacity

            commands.spawn((
                tile_sprite,
                Transform::from_xyz(world_pos.x, world_pos.y, 7.5),
                RoadHoverPreview,
            ));

            // Preview selected industry building variant a few pixels above the tile
            let variants = industry_atlas.variants.max(1);
            let variant_index =
                (current_industry_variant.index as usize) % variants;
            let y_offset = 6.0;

            let building_sprite = Sprite::from_atlas_image(
                industry_atlas.texture.clone(),
                TextureAtlas {
                    layout: industry_atlas.layout.clone(),
                    index: variant_index,
                },
            );

            commands.spawn((
                building_sprite,
                Transform::from_xyz(world_pos.x, world_pos.y + y_offset, 10.0),
                RoadHoverPreview,
            ));
        }
    }
}
