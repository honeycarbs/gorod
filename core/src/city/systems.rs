use bevy::prelude::*;

use crate::budget::{BuildingDemolished, BuildingPlaced, BuildingType};
use crate::time::GameClock;

use super::display::{setup_city_stats_display, update_city_stats_display};
use super::resources::{
    apply_demolition_happiness, apply_placement_happiness, building_contribution,
    CityInfrastructure, CityPopulation, CityServices,
};

pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CityPopulation>()
            .init_resource::<CityServices>()
            .init_resource::<CityInfrastructure>()
            .add_systems(Startup, setup_city_stats_display)
            .add_systems(
                Update,
                (
                    update_capacities_from_building_events,
                    update_infrastructure_from_building_events,
                ),
            )
            .add_systems(Update, update_population)
            .add_systems(Update, (update_demands, update_happiness_from_demands))
            .add_systems(
                Update,
                apply_placement_happiness.after(update_happiness_from_demands),
            )
            .add_systems(
                Update,
                apply_demolition_happiness.after(apply_placement_happiness),
            )
            .add_systems(Update, update_city_stats_display);
    }
}

/// reacts to `BuildingPlaced` / `BuildingDemolished` to keep city capacities in sync
pub fn update_capacities_from_building_events(
    mut services: ResMut<CityServices>,
    mut placed_reader: MessageReader<BuildingPlaced>,
    mut demolished_reader: MessageReader<BuildingDemolished>,
) {
    for event in placed_reader.read() {
        let contrib = building_contribution(event.building_type);
        services.housing_capacity += contrib.housing;
        services.job_capacity += contrib.jobs;
        services.entertainment_capacity += contrib.entertainment;
    }

    for event in demolished_reader.read() {
        let contrib = building_contribution(event.building_type);
        services.housing_capacity -= contrib.housing;
        services.job_capacity -= contrib.jobs;
        services.entertainment_capacity -= contrib.entertainment;
    }

    // clamp to avoid negative capacities
    services.housing_capacity = services.housing_capacity.max(0);
    services.job_capacity = services.job_capacity.max(0);
    services.entertainment_capacity = services.entertainment_capacity.max(0);
}

/// reacts to `BuildingPlaced` / `BuildingDemolished` to keep coarse infrastructure stats in sync
pub fn update_infrastructure_from_building_events(
    mut infra: ResMut<CityInfrastructure>,
    mut placed_reader: MessageReader<BuildingPlaced>,
    mut demolished_reader: MessageReader<BuildingDemolished>,
) {
    for event in placed_reader.read() {
        let contrib = building_contribution(event.building_type);
        match event.building_type {
            BuildingType::Residential => {
                infra.residential_count += 1;
            }
            BuildingType::Commercial => {
                infra.commercial_count += 1;
                infra.commercial_job_capacity += contrib.jobs;
            }
            BuildingType::Industry => {
                infra.industry_count += 1;
                infra.industry_job_capacity += contrib.jobs;
            }
            BuildingType::Road => {
                infra.road_count += 1;
            }
        }
    }

    for event in demolished_reader.read() {
        let contrib = building_contribution(event.building_type);
        match event.building_type {
            BuildingType::Residential => {
                infra.residential_count -= 1;
            }
            BuildingType::Commercial => {
                infra.commercial_count -= 1;
                infra.commercial_job_capacity -= contrib.jobs;
            }
            BuildingType::Industry => {
                infra.industry_count -= 1;
                infra.industry_job_capacity -= contrib.jobs;
            }
            BuildingType::Road => {
                infra.road_count -= 1;
            }
        }
    }

    infra.residential_count = infra.residential_count.max(0);
    infra.commercial_count = infra.commercial_count.max(0);
    infra.industry_count = infra.industry_count.max(0);
    infra.road_count = infra.road_count.max(0);
    infra.industry_job_capacity = infra.industry_job_capacity.max(0);
    infra.commercial_job_capacity = infra.commercial_job_capacity.max(0);
}

/// Adjust population based primarily on housing capacity once per in‑game day.
///
/// - Housing drives how many people can potentially live in the city.
/// - If there are not enough jobs/entertainment for that many people,
///   demands will become positive and happiness will drop.
pub fn update_population(
    clock: Res<GameClock>,
    services: Res<CityServices>,
    mut population: ResMut<CityPopulation>,
    mut last_processed_day: Local<u32>,
) {
    if *last_processed_day == clock.day {
        return;
    }
    *last_processed_day = clock.day;

    // drive population towards available housing, independent of jobs
    let target = services.housing_capacity.max(0);
    let diff = target - population.population;

    if diff == 0 {
        return;
    }

    let step = ((diff as f32) * 0.25).round() as i64;
    let step = if step == 0 { diff.signum() } else { step };

    population.population += step;
    population.population = population.population.max(0);
}

/// recompute demand from capacities and population
pub fn update_demands(
    mut services: ResMut<CityServices>,
    population: Res<CityPopulation>,
) {
    let pop = population.population.max(0);

    services.housing_demand = (pop - services.housing_capacity).max(0);
    services.job_demand = (pop - services.job_capacity).max(0);
    services.entertainment_demand = (pop - services.entertainment_capacity).max(0);
}

/// recompute happiness from current demands
pub fn update_happiness_from_demands(
    services: Res<CityServices>,
    mut population: ResMut<CityPopulation>,
) {
    let pop = population.population.max(0);

    let pop_f = pop as f32;
    let housing_pressure = if pop > 0 {
        services.housing_demand as f32 / pop_f
    } else {
        0.0
    };
    let job_pressure = if pop > 0 {
        services.job_demand as f32 / pop_f
    } else {
        0.0
    };
    let entertainment_pressure = if pop > 0 {
        services.entertainment_demand as f32 / pop_f
    } else {
        0.0
    };

    let pressure = 0.5 * housing_pressure + 0.3 * job_pressure + 0.2 * entertainment_pressure;

    let happiness = (1.0 - pressure).clamp(0.0, 1.0);
    population.happiness = happiness;
}
