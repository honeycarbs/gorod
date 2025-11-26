use bevy::prelude::*;

#[derive(Message)]
pub struct BuildingPlaced;

#[derive(Message)]
pub struct BuildingDemolished;

#[derive(Message)]
pub struct TransactionFailed;
