use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::TilePos;

use super::resources::BuildingType;

#[derive(Message)]
pub struct BuildingPlaced {
    pub building_type: BuildingType,
    pub tile_pos: TilePos,
}

#[derive(Message)]
pub struct BuildingDemolished {
    pub building_type: BuildingType,
    pub tile_pos: TilePos,
}

#[derive(Message)]
pub struct TransactionFailed;
