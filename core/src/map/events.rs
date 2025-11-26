use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::TilePos;
use crate::budget::BuildingType;

#[derive(Message)]
pub struct PlacementIntent {
    pub tile_pos: TilePos,
    pub building_type: BuildingType,
}