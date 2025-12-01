use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use super::events::*;
use super::helpers::*;
use super::resources::*;
use crate::budget::{Budget, BuildingPlaced, BuildingType, TransactionFailed};

pub fn collect_placement_intents(
    mouse_button: Res<ButtonInput<MouseButton>>,
    cursor_pos: Res<CursorWorldPos>,
    current_tile_type: Res<CurrentTileType>,
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
    if !mouse_button.just_pressed(MouseButton::Left) {
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
            BuildingType::from_texture_index(current_tile_type.texture_index)
        {
            intent_writer.write(PlacementIntent {
                tile_pos,
                building_type,
            });
        }
    }
}

pub fn execute_placement_intents(
    placeable_map: Res<PlaceableMap>,
    mut current_budget: ResMut<Budget>,
    mut building_events: MessageWriter<BuildingPlaced>,
    mut failed_events: MessageWriter<TransactionFailed>,
    mut intent_reader: MessageReader<PlacementIntent>,
    mut tile_q: Query<(&TilePos, &mut TileTextureIndex)>,
) {
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

            let new_texture_index = match intent.building_type {
                BuildingType::Residential => 2,
                BuildingType::Commercial => 3,
                BuildingType::Industry => 4,
                BuildingType::Road => 5,
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

            break;
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
