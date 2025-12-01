use bevy::prelude::*;
use super::resources::BuildingType;

#[derive(Message)]
pub struct BuildingPlaced {
    pub building_type: BuildingType,
}

#[derive(Message)]
pub struct BuildingDemolished {
    pub building_type: BuildingType,
}

#[derive(Message)]
pub struct TransactionFailed;