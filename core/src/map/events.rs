use crate::budget::BuildingType;
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::TilePos;

#[derive(Message)]
pub struct PlacementIntent {
    pub tile_pos: TilePos,
    pub building_type: BuildingType,
}
