use super::helpers::*;
use super::resources::*;
use crate::budget::BuildingType;
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use bevy_image::TextureAtlas;
use rand::Rng;

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
    mut preview_variant: ResMut<PreviewVariant>,
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
    mut cached_tile: Local<Option<(TilePos, BuildingType)>>,
) {
    // Only show preview while some building type is active
    let active_type = BuildingType::from_selection_index(current_tile_type.texture_index);

    if active_type.is_none() {
        for entity in preview_q.iter() {
            commands.entity(entity).despawn();
        }
        *cached_tile = None;
        return;
    }

    // Find the tile under the cursor and its world-space center
    let mut target_world_and_grid: Option<(Vec3, Vec2, TilePos)> = None;

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
            target_world_and_grid = Some((center, cell_size, tile_pos));
            break;
        }
    }

    let Some((world_pos, _cell_size, tile_pos)) = target_world_and_grid else {
        for entity in preview_q.iter() {
            commands.entity(entity).despawn();
        }
        *cached_tile = None;
        return;
    };

    let active_type = active_type.unwrap();

    // Check if we need to generate a new random variant (when tile or building type changes)
    let needs_new_variant = cached_tile
        .as_ref()
        .map(|(cached_pos, cached_type)| {
            *cached_pos != tile_pos || *cached_type != active_type
        })
        .unwrap_or(true);

    // Clear any existing preview entity (there should be at most one)
    for entity in preview_q.iter() {
        commands.entity(entity).despawn();
    }

    match active_type {
        BuildingType::Road => {
            let Some(road_atlas) = road_atlas else {
                return;
            };

            let variants = road_atlas.variants.max(1);
            let variant_index = (current_road_variant.index as usize) % variants;
            *cached_tile = Some((tile_pos, active_type));

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

            let variants = residential_atlas.variants.max(1);
            
            // Generate random variant once when tile or building type changes
            let variant_index = if needs_new_variant {
                let mut rng = rand::thread_rng();
                // Randomly choose tile 2 or 3, then select compatible variant
                let use_tile_2 = rng.gen_bool(0.5);
                let variant = if use_tile_2 && variants >= 3 {
                    // Tile 2: houses 3-5 (indices 2, 3, 4)
                    rng.gen_range(2..variants.min(5))
                } else if variants >= 2 {
                    // Tile 3: houses 1-2 (indices 0, 1)
                    rng.gen_range(0..variants.min(2))
                } else {
                    0
                };
                preview_variant.residential = Some(variant);
                *cached_tile = Some((tile_pos, active_type));
                variant
            } else {
                preview_variant.residential.unwrap_or(0)
            };
            
            // Houses 1-2 (indices 0,1) use tile 3; houses 3-5 (indices 2,3,4) use tile 2
            let tile_index = if variant_index <= 1 { 3 } else { 2 };
            
            let mut tile_sprite = Sprite::from_atlas_image(
                tile_preview_atlas.texture.clone(),
                TextureAtlas {
                    layout: tile_preview_atlas.layout.clone(),
                    index: tile_index,
                },
            );
            tile_sprite.color = Color::srgba(193.0 / 255.0, 231.0 / 255.0, 110.0 / 255.0, 0.2); // #c1e76e at ~20% opacity

            commands.spawn((
                tile_sprite,
                Transform::from_xyz(world_pos.x, world_pos.y, 7.5),
                RoadHoverPreview,
            ));

            // Preview random residential building variant
            let mut building_sprite = Sprite::from_atlas_image(
                residential_atlas.texture.clone(),
                TextureAtlas {
                    layout: residential_atlas.layout.clone(),
                    index: variant_index,
                },
            );
            building_sprite.color = Color::srgba(1.0, 1.0, 0.8, 0.5);

            commands.spawn((
                building_sprite,
                Transform::from_xyz(world_pos.x, world_pos.y, 10.0),
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

            let variants = commercial_atlas.variants.max(1);
            
            // Generate random variant once when tile or building type changes
            let variant_index = if needs_new_variant {
                let mut rng = rand::thread_rng();
                let variant = rng.gen_range(0..variants);
                preview_variant.commercial = Some(variant);
                *cached_tile = Some((tile_pos, active_type));
                variant
            } else {
                preview_variant.commercial.unwrap_or(0)
            };

            // Tinted base tile preview (commercial color)
            let mut tile_sprite = Sprite::from_atlas_image(
                tile_preview_atlas.texture.clone(),
                TextureAtlas {
                    layout: tile_preview_atlas.layout.clone(),
                    index: 4, // commercial/industry/road tile index
                },
            );
            tile_sprite.color = Color::srgba(123.0 / 255.0, 194.0 / 255.0, 212.0 / 255.0, 0.2); // #7bc2d4 at ~20% opacity

            commands.spawn((
                tile_sprite,
                Transform::from_xyz(world_pos.x, world_pos.y, 7.5),
                RoadHoverPreview,
            ));

            let mut building_sprite = Sprite::from_atlas_image(
                commercial_atlas.texture.clone(),
                TextureAtlas {
                    layout: commercial_atlas.layout.clone(),
                    index: variant_index,
                },
            );
            building_sprite.color = Color::srgba(1.0, 1.0, 0.8, 0.5);

            commands.spawn((
                building_sprite,
                Transform::from_xyz(world_pos.x, world_pos.y, 10.0),
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

            let variants = industry_atlas.variants.max(1);
            
            // Generate random variant once when tile or building type changes
            let variant_index = if needs_new_variant {
                let mut rng = rand::thread_rng();
                let variant = rng.gen_range(0..variants);
                preview_variant.industry = Some(variant);
                *cached_tile = Some((tile_pos, active_type));
                variant
            } else {
                preview_variant.industry.unwrap_or(0)
            };

            // Tinted base tile preview
            let mut tile_sprite = Sprite::from_atlas_image(
                tile_preview_atlas.texture.clone(),
                TextureAtlas {
                    layout: tile_preview_atlas.layout.clone(),
                    index: 4, // commercial/industry/road tile index
                },
            );
            tile_sprite.color = Color::srgba(123.0 / 255.0, 194.0 / 255.0, 212.0 / 255.0, 0.2); // #7bc2d4 at ~20% opacity

            commands.spawn((
                tile_sprite,
                Transform::from_xyz(world_pos.x, world_pos.y, 7.5),
                RoadHoverPreview,
            ));

            let mut building_sprite = Sprite::from_atlas_image(
                industry_atlas.texture.clone(),
                TextureAtlas {
                    layout: industry_atlas.layout.clone(),
                    index: variant_index,
                },
            );
            building_sprite.color = Color::srgba(1.0, 1.0, 0.8, 0.5);

            commands.spawn((
                building_sprite,
                Transform::from_xyz(world_pos.x, world_pos.y, 10.0),
                RoadHoverPreview,
            ));
        }
    }
}
