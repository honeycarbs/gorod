use bevy::ecs::system::SystemParam;
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
    if let Some(state) = help_state
        && state.active
    {
        return;
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

#[derive(SystemParam)]
pub struct PlacementExecutionInputs<'w, 's> {
    placeable_map: Res<'w, PlaceableMap>,
    current_budget: ResMut<'w, Budget>,
    building_events: MessageWriter<'w, BuildingPlaced>,
    failed_events: MessageWriter<'w, TransactionFailed>,
    intent_reader: MessageReader<'w, 's, PlacementIntent>,
    tile_q: Query<'w, 's, (&'static TilePos, &'static mut TileTextureIndex)>,
    atlases: PlacementAtlasResources<'w>,
    variants: PlacementVariantResources<'w>,
    commands: Commands<'w, 's>,
    map_q: Query<
        'w,
        's,
        (
            &'static TilemapSize,
            &'static TilemapGridSize,
            &'static Transform,
        ),
    >,
}

#[derive(SystemParam)]
pub struct PlacementAtlasResources<'w> {
    residential: Res<'w, ResidentialBuildingAtlas>,
    commercial: Res<'w, CommercialBuildingAtlas>,
    industry: Res<'w, IndustryBuildingAtlas>,
    road: Res<'w, RoadAtlas>,
    decorative: Res<'w, DecorativeBuildingAtlas>,
}

#[derive(SystemParam)]
pub struct PlacementVariantResources<'w> {
    preview: Res<'w, PreviewVariant>,
    road: Res<'w, CurrentRoadVariant>,
    decorative: Res<'w, CurrentDecorativeVariant>,
}

pub fn execute_placement_intents(mut inputs: PlacementExecutionInputs) {
    let (map_size, grid_size, map_transform) = if let Some(v) = inputs.map_q.iter().next() {
        v
    } else {
        return;
    };

    for intent in inputs.intent_reader.read() {
        let desired_pos = intent.tile_pos;

        for (tile_pos, mut texture_index) in inputs.tile_q.iter_mut() {
            if *tile_pos != desired_pos {
                continue;
            }

            if !inputs.placeable_map.is_placeable(tile_pos) {
                warn!("Cannot place here - tile not placeable!");
                inputs.failed_events.write(TransactionFailed);
                break;
            }

            if texture_index.0 != 1 {
                warn!("Cannot place here - tile already occupied!");
                inputs.failed_events.write(TransactionFailed);
                break;
            }

            let cost = intent.building_type.cost();

            if !inputs.current_budget.can_afford(cost) {
                warn!(
                    "Cannot afford {:?}! Cost: ${}, Balance: ${}",
                    intent.building_type, cost, inputs.current_budget.money
                );
                inputs.failed_events.write(TransactionFailed);
                break;
            }

            inputs.current_budget.spend(cost);

            let (new_texture_index, variant_index) = match intent.building_type {
                BuildingType::Residential => {
                    let variant = inputs.variants.preview.residential.unwrap_or(0);
                    // Houses 1-2 (indices 0,1) use tile 3; houses 3-5 (indices 2,3,4) use tile 2
                    let tile_index = if variant <= 1 { 3 } else { 2 };
                    (tile_index, variant)
                }
                BuildingType::Commercial => {
                    let variant = inputs.variants.preview.commercial.unwrap_or(0);
                    (4, variant)
                }
                BuildingType::Industry => {
                    let variant = inputs.variants.preview.industry.unwrap_or(0);
                    (4, variant)
                }
                BuildingType::Road => {
                    let variant =
                        (inputs.variants.road.index as usize) % inputs.atlases.road.variants.max(1);
                    (4, variant)
                }
                BuildingType::Decorative => {
                    let variant = (inputs.variants.decorative.index as usize)
                        % inputs.atlases.decorative.variants.max(1);
                    (4, variant)
                }
            };

            texture_index.0 = new_texture_index;

            info!(
                "Built {:?} for ${}. Balance: ${}",
                intent.building_type, cost, inputs.current_budget.money
            );

            inputs.building_events.write(BuildingPlaced {
                building_type: intent.building_type,
                tile_pos: *tile_pos,
            });

            if intent.building_type == BuildingType::Residential
                && inputs.atlases.residential.variants > 0
            {
                let world_pos = tile_center_to_world(tile_pos, map_size, grid_size, map_transform);

                let sprite = Sprite::from_atlas_image(
                    inputs.atlases.residential.texture.clone(),
                    TextureAtlas {
                        layout: inputs.atlases.residential.layout.clone(),
                        index: variant_index,
                    },
                );

                inputs.commands.spawn((
                    sprite,
                    Transform::from_xyz(world_pos.x, world_pos.y, 10.0),
                    ResidentialBuilding {
                        tile_pos: *tile_pos,
                    },
                ));
            } else if intent.building_type == BuildingType::Commercial
                && inputs.atlases.commercial.variants > 0
            {
                let world_pos = tile_center_to_world(tile_pos, map_size, grid_size, map_transform);

                let sprite = Sprite::from_atlas_image(
                    inputs.atlases.commercial.texture.clone(),
                    TextureAtlas {
                        layout: inputs.atlases.commercial.layout.clone(),
                        index: variant_index,
                    },
                );

                inputs.commands.spawn((
                    sprite,
                    Transform::from_xyz(world_pos.x, world_pos.y, 10.0),
                    CommercialBuilding {
                        tile_pos: *tile_pos,
                    },
                ));
            } else if intent.building_type == BuildingType::Industry
                && inputs.atlases.industry.variants > 0
            {
                let world_pos = tile_center_to_world(tile_pos, map_size, grid_size, map_transform);

                let sprite = Sprite::from_atlas_image(
                    inputs.atlases.industry.texture.clone(),
                    TextureAtlas {
                        layout: inputs.atlases.industry.layout.clone(),
                        index: variant_index,
                    },
                );

                inputs.commands.spawn((
                    sprite,
                    Transform::from_xyz(world_pos.x, world_pos.y, 10.0),
                    IndustryBuilding {
                        tile_pos: *tile_pos,
                    },
                ));
            } else if intent.building_type == BuildingType::Road && inputs.atlases.road.variants > 0
            {
                let world_pos = tile_center_to_world(tile_pos, map_size, grid_size, map_transform);
                let variant_index =
                    (inputs.variants.road.index as usize) % inputs.atlases.road.variants.max(1);

                let sprite = Sprite::from_atlas_image(
                    inputs.atlases.road.texture.clone(),
                    TextureAtlas {
                        layout: inputs.atlases.road.layout.clone(),
                        index: variant_index,
                    },
                );

                inputs.commands.spawn((
                    sprite,
                    Transform::from_xyz(world_pos.x, world_pos.y, 5.0),
                    RoadSegment {
                        tile_pos: *tile_pos,
                    },
                ));
            } else if intent.building_type == BuildingType::Decorative
                && inputs.atlases.decorative.variants > 0
            {
                let world_pos = tile_center_to_world(tile_pos, map_size, grid_size, map_transform);
                let variant_index = (inputs.variants.decorative.index as usize)
                    % inputs.atlases.decorative.variants.max(1);

                let sprite = Sprite::from_atlas_image(
                    inputs.atlases.decorative.texture.clone(),
                    TextureAtlas {
                        layout: inputs.atlases.decorative.layout.clone(),
                        index: variant_index,
                    },
                );

                inputs.commands.spawn((
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

#[derive(SystemParam)]
pub struct TileTypeChangeInputs<'w> {
    keyboard: Res<'w, ButtonInput<KeyCode>>,
    current_tile_type: ResMut<'w, CurrentTileType>,
    variants: TileVariantResources<'w>,
    preview_variant: ResMut<'w, PreviewVariant>,
    atlases: TileAtlasResources<'w>,
    help_state: Option<Res<'w, HelpOverlayState>>,
}

#[derive(SystemParam)]
pub struct TileVariantResources<'w> {
    commercial: ResMut<'w, CurrentCommercialVariant>,
    industry: ResMut<'w, CurrentIndustryVariant>,
    road: ResMut<'w, CurrentRoadVariant>,
    decorative: ResMut<'w, CurrentDecorativeVariant>,
}

#[derive(SystemParam)]
pub struct TileAtlasResources<'w> {
    residential: Option<Res<'w, ResidentialBuildingAtlas>>,
    road: Option<Res<'w, RoadAtlas>>,
    decorative: Option<Res<'w, DecorativeBuildingAtlas>>,
}

pub fn change_tile_type(mut inputs: TileTypeChangeInputs) {
    if let Some(state) = inputs.help_state.as_ref()
        && state.active
    {
        return;
    }
    if inputs.keyboard.just_pressed(KeyCode::Escape) {
        inputs.current_tile_type.texture_index = 0;
        info!("Selected: None");
    } else if inputs.keyboard.just_pressed(KeyCode::KeyR) {
        inputs.current_tile_type.texture_index = 2;
        info!("Selected: Residential/Housing");
    } else if inputs.keyboard.just_pressed(KeyCode::KeyC) {
        inputs.current_tile_type.texture_index = 3;
        info!("Selected: Commercial");
    } else if inputs.keyboard.just_pressed(KeyCode::KeyI) {
        inputs.current_tile_type.texture_index = 4;
        info!("Selected: Industry");
    } else if inputs.keyboard.just_pressed(KeyCode::KeyO) {
        inputs.current_tile_type.texture_index = 5;
        info!("Selected: Road");
    } else if inputs.keyboard.just_pressed(KeyCode::KeyB) {
        inputs.current_tile_type.texture_index = 6;
        info!("Selected: Decorative");
    }

    // When a building type is selected, allow cycling through its variants
    // using ',' (next) and '.' (previous)
    if let Some(active_type) =
        crate::budget::BuildingType::from_selection_index(inputs.current_tile_type.texture_index)
    {
        let mut delta: i32 = 0;

        if inputs.keyboard.just_pressed(KeyCode::Comma) {
            // ',' => +1 variant
            delta += 1;
        }
        if inputs.keyboard.just_pressed(KeyCode::Period) {
            // '.' => -1 variant
            delta -= 1;
        }

        if delta != 0 {
            match active_type {
                crate::budget::BuildingType::Residential => {
                    let variants = inputs
                        .atlases
                        .residential
                        .as_ref()
                        .map(|a| a.variants as i32)
                        .unwrap_or(RESIDENTIAL_VARIANT_COUNT as i32);
                    if variants > 0 {
                        let current_preview =
                            inputs.preview_variant.residential.unwrap_or(0) as i32;
                        let new_index = current_preview + delta;
                        let final_variant = new_index.rem_euclid(variants) as usize;

                        inputs.preview_variant.residential = Some(final_variant);
                        info!("Selected residential variant: {}", final_variant);
                    }
                }
                crate::budget::BuildingType::Commercial => {
                    let current = inputs.variants.commercial.index as i32;
                    let variants = COMMERCIAL_VARIANT_COUNT as i32;
                    if variants > 0 {
                        let new_index = current + delta;
                        inputs.variants.commercial.index = new_index.rem_euclid(variants) as u32;
                        info!(
                            "Selected commercial variant: {}",
                            inputs.variants.commercial.index
                        );
                    }
                }
                crate::budget::BuildingType::Road => {
                    let current = inputs.variants.road.index as i32;
                    let variants = inputs
                        .atlases
                        .road
                        .as_ref()
                        .map(|a| a.variants as i32)
                        .unwrap_or(ROAD_VARIANT_COUNT as i32);

                    if variants > 0 {
                        let new_index = current + delta;
                        inputs.variants.road.index = new_index.rem_euclid(variants) as u32;
                        info!("Selected road variant: {}", inputs.variants.road.index);
                    }
                }
                crate::budget::BuildingType::Industry => {
                    let current = inputs.variants.industry.index as i32;
                    let variants = INDUSTRY_VARIANT_COUNT as i32;
                    if variants > 0 {
                        let new_index = current + delta;
                        inputs.variants.industry.index = new_index.rem_euclid(variants) as u32;
                        info!(
                            "Selected industry variant: {}",
                            inputs.variants.industry.index
                        );
                    }
                }
                crate::budget::BuildingType::Decorative => {
                    let current = inputs.variants.decorative.index as i32;
                    let variants = inputs
                        .atlases
                        .decorative
                        .as_ref()
                        .map(|a| a.variants as i32)
                        .unwrap_or(DECORATIVE_VARIANT_COUNT as i32);

                    if variants > 0 {
                        let new_index = current + delta;
                        inputs.variants.decorative.index = new_index.rem_euclid(variants) as u32;
                        info!(
                            "Selected decorative variant: {}",
                            inputs.variants.decorative.index
                        );
                    }
                }
            }
        }
    }
}
