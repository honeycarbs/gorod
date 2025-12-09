use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::TilePos;

use crate::budget::{BuildingDemolished, BuildingPlaced, BuildingType};
use crate::spatial::SpatialGrid;

const RESIDENTIAL_NEIGHBOR_RADIUS: i32 = 3;
const ROAD_NEIGHBOR_RADIUS: i32 = 4;

#[derive(Resource, Debug)]
pub struct CityPopulation {
    /// Total number of citizens currently living in the city
    pub population: i64,
    /// from 0.0 as "everyone is sad" to 1.0 as "everyone is happy"
    pub happiness: f32,
}

impl Default for CityPopulation {
    fn default() -> Self {
        Self {
            population: 0,
            happiness: 1.0,
        }
    }
}

/// Describes how much housing, jobs and entertainment the city provides
#[derive(Resource, Debug, Default)]
pub struct CityServices {
    pub housing_capacity: i64,
    pub job_capacity: i64,
    pub entertainment_capacity: i64,

    pub housing_demand: i64,
    pub job_demand: i64,
    pub entertainment_demand: i64,
}

/// Tracks coarse infrastructure statistics that drive upkeep and income
#[derive(Resource, Debug, Default)]
pub struct CityInfrastructure {
    pub residential_count: i64,
    pub commercial_count: i64,
    pub industry_count: i64,
    pub road_count: i64,

    pub industry_job_capacity: i64,
    pub commercial_job_capacity: i64,
}

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
            jobs: 5,
            entertainment: 15,
        },
        BuildingType::Industry => BuildingContribution {
            housing: 0,
            jobs: 15,
            entertainment: 3,
        },
        BuildingType::Road => BuildingContribution {
            housing: 0,
            jobs: 0,
            entertainment: 0,
        },
    }
}

fn count_nearby_residential(center: &TilePos, spatial_grid: &SpatialGrid) -> u32 {
    spatial_grid.count_residential_in_radius(center, RESIDENTIAL_NEIGHBOR_RADIUS)
}

pub fn apply_demolition_happiness(
    mut population: ResMut<CityPopulation>,
    services: Res<CityServices>,
    spatial_grid: Res<SpatialGrid>,
    mut demolished_reader: MessageReader<BuildingDemolished>,
) {
    for event in demolished_reader.read() {
        let nearby_residential = count_nearby_residential(&event.tile_pos, &spatial_grid);

        let pop = population.population.max(1) as f32;
        let housing_need = (services.housing_demand.max(0) as f32 / pop).clamp(0.0, 1.0);

        let mut delta = match event.building_type {
            BuildingType::Residential => {
                -0.05 * housing_need.max(0.1)
            }
            BuildingType::Commercial => -0.02 * nearby_residential as f32,
            BuildingType::Industry => {
                let positive = 0.005 * nearby_residential as f32;

                let contrib = building_contribution(event.building_type);
                let jobs_lost = contrib.jobs as f32;

                let pop = population.population.max(1) as f32;
                let job_pressure = services.job_demand.max(0) as f32 / pop;

                let negative = -0.03 * job_pressure * (jobs_lost / 10.0);

                positive + negative
            }
            BuildingType::Road => 0.0,
        };

        delta = delta.clamp(-0.05, 0.05);
        if delta != 0.0 {
            if delta < 0.0 {
                info!(
                    "Happiness decreased by {:.3} due to demolition of {:?} at {:?} (nearby_residential={})",
                    delta, event.building_type, event.tile_pos, nearby_residential
                );
            }
            population.happiness = (population.happiness + delta).clamp(0.0, 1.0);
        }
    }
}

fn is_accessible(center: &TilePos, spatial_grid: &SpatialGrid) -> bool {
    spatial_grid.has_road_in_radius(center, ROAD_NEIGHBOR_RADIUS)
}

pub fn apply_placement_happiness(
    mut population: ResMut<CityPopulation>,
    services: Res<CityServices>,
    spatial_grid: Res<SpatialGrid>,
    mut placed_reader: MessageReader<BuildingPlaced>,
) {
    for event in placed_reader.read() {
        if !is_accessible(&event.tile_pos, &spatial_grid) {
            continue;
        }

        let nearby_residential = count_nearby_residential(&event.tile_pos, &spatial_grid);

        let pop = population.population.max(1) as f32;
        let housing_need = (services.housing_demand.max(0) as f32 / pop).clamp(0.0, 1.0);
        let job_need = (services.job_demand.max(0) as f32 / pop).clamp(0.0, 1.0);

        let mut delta = match event.building_type {
            BuildingType::Residential => {
                let base = 0.01 * (nearby_residential as f32 + 1.0);
                base * housing_need
            }
            BuildingType::Commercial => {
                let base = 0.015 * nearby_residential as f32;
                base * job_need
            }
            BuildingType::Industry => {
                let contrib = building_contribution(event.building_type);
                let jobs_gained = contrib.jobs as f32;

                let positive = 0.01 * job_need * (jobs_gained / 10.0);
                let negative = -0.005 * nearby_residential as f32;
                let isolation_penalty = if nearby_residential == 0 { -0.01 } else { 0.0 };

                positive + negative + isolation_penalty
            }
            BuildingType::Road => {
                let base = 0.003 * nearby_residential as f32;
                base * housing_need
            }
        };

        delta = delta.clamp(-0.05, 0.05);
        if delta != 0.0 {
            if delta < 0.0 {
                info!(
                    "Happiness decreased by {:.3} due to placement of {:?} at {:?} (nearby_residential={})",
                    delta, event.building_type, event.tile_pos, nearby_residential
                );
            }
            population.happiness = (population.happiness + delta).clamp(0.0, 1.0);
        }
    }
}
