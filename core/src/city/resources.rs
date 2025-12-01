use bevy::prelude::*;
use crate::budget::BuildingType;

#[derive(Resource, Debug)]
pub struct CityStats {
    pub population: i64,

    pub housing_capacity: i64,
    pub job_capacity: i64,
    pub entertainment_capacity: i64,

    pub housing_demand: i64,
    pub job_demand: i64,
    pub entertainment_demand: i64,

    // from 0.0 as "everyone is sad" to 1.0 as "everyone is happy"
    pub happiness: f32,
}

impl Default for CityStats {
    fn default() -> Self {
        Self {
            population: 0,
            housing_capacity: 0,
            job_capacity: 0,
            entertainment_capacity: 0,
            housing_demand: 0,
            job_demand: 0,
            entertainment_demand: 0,
            happiness: 1.0,
        }
    }
}

/// Simple, hard-coded contributions for each building type.
pub struct BuildingContribution {
    pub housing: i64,
    pub jobs: i64,
    pub entertainment: i64,
}

pub fn building_contribution(building_type: BuildingType) -> BuildingContribution {
    match building_type {
        BuildingType::Residential => BuildingContribution {
            housing: 10,
            jobs: 0,
            entertainment: 0,
        },
        BuildingType::Commercial => BuildingContribution {
            housing: 0,
            jobs: 8,
            entertainment: 5,
        },
        BuildingType::Industry => BuildingContribution {
            housing: 0,
            jobs: 15,
            entertainment: 0,
        },
        BuildingType::Road => BuildingContribution {
            housing: 0,
            jobs: 0,
            entertainment: 0,
        },
    }
}