use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use bevy_image::TextureAtlas;

use super::events::*;
use super::helpers::*;
use super::resources::*;
use crate::budget::{Budget, BuildingPlaced, BuildingType, TransactionFailed};
use crate::time::HelpOverlayState;

pub fn collect_placement_intents(
    mouse_button: Res<ButtonInput<MouseButton>>,
    cursor_pos: Res<CursorWorldPos>,
    current_tile_type: Res<CurrentTileType>,
    ui_click_blocker: Res<UiClickBlocker>,
    help_state: Option<Res<HelpOverlayState>>,
    tilemap_q: Query<(
        &TilemapSize,
        &TilemapGridSize,
        &TilemapTileSize,
        &TilemapType,
        &Transform,
        &TilemapAnchor,
    )>,
    mut intent_writer: MessageWriter<PlacementIntent>,
) {
    if let Some(state) = help_state {
        if state.active {
            return;
        }
    }

    if !mouse_button.just_pressed(MouseButton::Left) {
        return;
    }

    if ui_click_blocker.just_clicked_ui {
        return;
    }

    for (map_size, grid_size, tile_size, map_type, map_transform, anchor) in tilemap_q.iter() {
        let cursor_in_map_pos = cursor_to_map_pos(cursor_pos.0, map_transform);

        if let Some(tile_pos) = TilePos::from_world_pos(
            &cursor_in_map_pos,
            map_size,
            grid_size,
            tile_size,
            map_type,
            anchor,
        ) && let Some(building_type) =
            BuildingType::from_selection_index(current_tile_type.texture_index)
        {
            intent_writer.write(PlacementIntent {
                tile_pos,
                building_type,
            });
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn execute_placement_intents(
    placeable_map: Res<PlaceableMap>,
    mut current_budget: ResMut<Budget>,
    mut building_events: MessageWriter<BuildingPlaced>,
    mut failed_events: MessageWriter<TransactionFailed>,
    mut intent_reader: MessageReader<PlacementIntent>,
    mut tile_q: Query<(&TilePos, &mut TileTextureIndex)>,
    residential_atlas: Res<ResidentialBuildingAtlas>,
    commercial_atlas: Res<CommercialBuildingAtlas>,
    industry_atlas: Res<IndustryBuildingAtlas>,
    road_atlas: Res<RoadAtlas>,
    decorative_atlas: Res<DecorativeBuildingAtlas>,
    preview_variant: Res<PreviewVariant>,
    current_road_variant: Res<CurrentRoadVariant>,
    current_decorative_variant: Res<CurrentDecorativeVariant>,
    mut commands: Commands,
    map_q: Query<(&TilemapSize, &TilemapGridSize, &Transform)>,
) {
    let (map_size, grid_size, map_transform) = if let Some(v) = map_q.iter().next() {
        v
    } else {
        return;
    };

    for intent in intent_reader.read() {
        let desired_pos = intent.tile_pos;

        for (tile_pos, mut texture_index) in tile_q.iter_mut() {
            if *tile_pos != desired_pos {
                continue;
            }

            if !placeable_map.is_placeable(tile_pos) {
                warn!("Cannot place here - tile not placeable!");
                failed_events.write(TransactionFailed);
                break;
            }

            if texture_index.0 != 1 {
                warn!("Cannot place here - tile already occupied!");
                failed_events.write(TransactionFailed);
                break;
            }

            let cost = intent.building_type.cost();

            if !current_budget.can_afford(cost) {
                warn!(
                    "Cannot afford {:?}! Cost: ${}, Balance: ${}",
                    intent.building_type, cost, current_budget.money
                );
                failed_events.write(TransactionFailed);
                break;
            }

            current_budget.spend(cost);

            let (new_texture_index, variant_index) = match intent.building_type {
                BuildingType::Residential => {
                    let variant = preview_variant.residential.unwrap_or(0);
                    // Houses 1-2 (indices 0,1) use tile 3; houses 3-5 (indices 2,3,4) use tile 2
                    let tile_index = if variant <= 1 { 3 } else { 2 };
                    (tile_index, variant)
                }
                BuildingType::Commercial => {
                    let variant = preview_variant.commercial.unwrap_or(0);
                    (4, variant)
                }
                BuildingType::Industry => {
                    let variant = preview_variant.industry.unwrap_or(0);
                    (4, variant)
                }
                BuildingType::Road => {
                    let variant = (current_road_variant.index as usize) % road_atlas.variants.max(1);
                    (4, variant)
                }
                BuildingType::Decorative => {
                    let variant = (current_decorative_variant.index as usize) % decorative_atlas.variants.max(1);
                    (4, variant)
                }
            };

            texture_index.0 = new_texture_index;

            info!(
                "Built {:?} for ${}. Balance: ${}",
                intent.building_type, cost, current_budget.money
            );

            building_events.write(BuildingPlaced {
                building_type: intent.building_type,
                tile_pos: *tile_pos,
            });

            if intent.building_type == BuildingType::Residential && residential_atlas.variants > 0 {
                let world_pos = tile_center_to_world(tile_pos, map_size, grid_size, map_transform);

                let sprite = Sprite::from_atlas_image(
                    residential_atlas.texture.clone(),
                    TextureAtlas {
                        layout: residential_atlas.layout.clone(),
                        index: variant_index as usize,
                    },
                );

                commands.spawn((
                    sprite,
                    Transform::from_xyz(world_pos.x, world_pos.y, 10.0),
                    ResidentialBuilding {
                        tile_pos: *tile_pos,
                    },
                ));
            } else if intent.building_type == BuildingType::Commercial
                && commercial_atlas.variants > 0
            {
                let world_pos = tile_center_to_world(tile_pos, map_size, grid_size, map_transform);

                let sprite = Sprite::from_atlas_image(
                    commercial_atlas.texture.clone(),
                    TextureAtlas {
                        layout: commercial_atlas.layout.clone(),
                        index: variant_index,
                    },
                );

                commands.spawn((
                    sprite,
                    Transform::from_xyz(world_pos.x, world_pos.y, 10.0),
                    CommercialBuilding {
                        tile_pos: *tile_pos,
                    },
                ));
            } else if intent.building_type == BuildingType::Industry && industry_atlas.variants > 0
            {
                let world_pos = tile_center_to_world(tile_pos, map_size, grid_size, map_transform);

                let sprite = Sprite::from_atlas_image(
                    industry_atlas.texture.clone(),
                    TextureAtlas {
                        layout: industry_atlas.layout.clone(),
                        index: variant_index,
                    },
                );

                commands.spawn((
                    sprite,
                    Transform::from_xyz(world_pos.x, world_pos.y, 10.0),
                    IndustryBuilding {
                        tile_pos: *tile_pos,
                    },
                ));
            } else if intent.building_type == BuildingType::Road && road_atlas.variants > 0 {
                let world_pos = tile_center_to_world(tile_pos, map_size, grid_size, map_transform);
                let variant_index = (current_road_variant.index as usize) % road_atlas.variants.max(1);

                let sprite = Sprite::from_atlas_image(
                    road_atlas.texture.clone(),
                    TextureAtlas {
                        layout: road_atlas.layout.clone(),
                        index: variant_index,
                    },
                );

                commands.spawn((
                    sprite,
                    Transform::from_xyz(world_pos.x, world_pos.y, 5.0),
                    RoadSegment {
                        tile_pos: *tile_pos,
                    },
                ));
            } else if intent.building_type == BuildingType::Decorative && decorative_atlas.variants > 0 {
                let world_pos = tile_center_to_world(tile_pos, map_size, grid_size, map_transform);
                let variant_index = (current_decorative_variant.index as usize) % decorative_atlas.variants.max(1);

                let sprite = Sprite::from_atlas_image(
                    decorative_atlas.texture.clone(),
                    TextureAtlas {
                        layout: decorative_atlas.layout.clone(),
                        index: variant_index,
                    },
                );

                commands.spawn((
                    sprite,
                    Transform::from_xyz(world_pos.x, world_pos.y, 10.0),
                    DecorativeBuilding {
                        tile_pos: *tile_pos,
                    },
                ));
            }

            break;
        }
    }
}

pub fn change_tile_type(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut current_tile_type: ResMut<CurrentTileType>,
    mut current_commercial_variant: ResMut<CurrentCommercialVariant>,
    mut current_industry_variant: ResMut<CurrentIndustryVariant>,
    mut current_road_variant: ResMut<CurrentRoadVariant>,
    mut current_decorative_variant: ResMut<CurrentDecorativeVariant>,
    mut preview_variant: ResMut<PreviewVariant>,
    residential_atlas: Option<Res<ResidentialBuildingAtlas>>,
    road_atlas: Option<Res<RoadAtlas>>,
    decorative_atlas: Option<Res<DecorativeBuildingAtlas>>,
    help_state: Option<Res<HelpOverlayState>>,
) {
    if let Some(state) = help_state {
        if state.active {
            return;
        }
    }
    if keyboard.just_pressed(KeyCode::Escape) {
        current_tile_type.texture_index = 0;
        info!("Selected: None");
    } else if keyboard.just_pressed(KeyCode::KeyR) {
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
    } else if keyboard.just_pressed(KeyCode::KeyB) {
        current_tile_type.texture_index = 6;
        info!("Selected: Decorative");
    }

    // When a building type is selected, allow cycling through its variants
    // using ',' (next) and '.' (previous)
    if let Some(active_type) =
        crate::budget::BuildingType::from_selection_index(current_tile_type.texture_index)
    {
        let mut delta: i32 = 0;

        if keyboard.just_pressed(KeyCode::Comma) {
            // ',' => +1 variant
            delta += 1;
        }
        if keyboard.just_pressed(KeyCode::Period) {
            // '.' => -1 variant
            delta -= 1;
        }

        if delta != 0 {
            match active_type {
                crate::budget::BuildingType::Residential => {
                    let variants = residential_atlas
                        .as_ref()
                        .map(|a| a.variants as i32)
                        .unwrap_or(RESIDENTIAL_VARIANT_COUNT as i32);
                    if variants > 0 {
                        let current_preview = preview_variant.residential.unwrap_or(0) as i32;
                        let new_index = current_preview + delta;
                        let final_variant = new_index.rem_euclid(variants) as usize;
                        
                        preview_variant.residential = Some(final_variant);
                        info!("Selected residential variant: {}", final_variant);
                    }
                }
                crate::budget::BuildingType::Commercial => {
                    let current = current_commercial_variant.index as i32;
                    let variants = COMMERCIAL_VARIANT_COUNT as i32;
                    if variants > 0 {
                        let new_index = current + delta;
                        current_commercial_variant.index = new_index.rem_euclid(variants) as u32;
                        info!(
                            "Selected commercial variant: {}",
                            current_commercial_variant.index
                        );
                    }
                }
                crate::budget::BuildingType::Road => {
                    let current = current_road_variant.index as i32;
                    let variants = road_atlas
                        .as_ref()
                        .map(|a| a.variants as i32)
                        .unwrap_or(ROAD_VARIANT_COUNT as i32);

                    if variants > 0 {
                        let new_index = current + delta;
                        current_road_variant.index = new_index.rem_euclid(variants) as u32;
                        info!("Selected road variant: {}", current_road_variant.index);
                    }
                }
                crate::budget::BuildingType::Industry => {
                    let current = current_industry_variant.index as i32;
                    let variants = INDUSTRY_VARIANT_COUNT as i32;
                    if variants > 0 {
                        let new_index = current + delta;
                        current_industry_variant.index = new_index.rem_euclid(variants) as u32;
                        info!(
                            "Selected industry variant: {}",
                            current_industry_variant.index
                        );
                    }
                }
                crate::budget::BuildingType::Decorative => {
                    let current = current_decorative_variant.index as i32;
                    let variants = decorative_atlas
                        .as_ref()
                        .map(|a| a.variants as i32)
                        .unwrap_or(DECORATIVE_VARIANT_COUNT as i32);

                    if variants > 0 {
                        let new_index = current + delta;
                        current_decorative_variant.index = new_index.rem_euclid(variants) as u32;
                        info!("Selected decorative variant: {}", current_decorative_variant.index);
                    }
                }
            }
        }
    }
}
