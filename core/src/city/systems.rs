use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::{TileStorage, TileTextureIndex, TilemapSize};

use crate::budget::{BuildingDemolished, BuildingPlaced, BuildingType};
use crate::map::{
    ABANDONED_TEXTURE_INDEX, CommercialBuilding, IndustryBuilding, ResidentialBuilding,
};
use crate::spatial::{SpatialGrid, sync_spatial_grid_on_demolition, sync_spatial_grid_on_placement};
use crate::time::GameClock;

use super::display::{setup_city_stats_display, update_city_stats_display};
use super::resources::{
    CityInfrastructure, CityPopulation, CityServices, apply_demolition_happiness,
    apply_placement_happiness, building_contribution,
};

pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CityPopulation>()
            .init_resource::<CityServices>()
            .init_resource::<CityInfrastructure>()
            .init_resource::<SpatialGrid>()
            .add_systems(Startup, setup_city_stats_display)
            .add_systems(
                Update,
                (
                    sync_spatial_grid_on_placement,
                    sync_spatial_grid_on_demolition,
                    update_capacities_from_building_events,
                    update_infrastructure_from_building_events,
                )
                    .chain(),
            )
            .add_systems(Update, update_population)
            .add_systems(Update, (update_demands, update_happiness_from_demands))
            .add_systems(
                Update,
                apply_abandonment.after(update_happiness_from_demands),
            )
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

    services.housing_capacity = services.housing_capacity.max(0);
    services.job_capacity = services.job_capacity.max(0);
    services.entertainment_capacity = services.entertainment_capacity.max(0);
}

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

fn calculate_population_target(
    housing_capacity: i64,
    current_population: i64,
    happiness: f32,
    job_capacity: i64,
) -> i64 {
    let base_target = housing_capacity.max(0);

    if happiness < 0.7 || current_population == 0 {
        return base_target;
    }

    let job_availability = if current_population > 0 {
        (job_capacity as f32 / current_population as f32).min(1.0)
    } else {
        1.0
    };

    if job_availability < 0.5 {
        return base_target;
    }

    let happiness_factor = ((happiness - 0.7) / 0.3).clamp(0.0, 1.0);
    let base_rate = 0.005;
    let pop_factor = (current_population as f32 / 1000.0).min(1.0);
    let growth_reduction = pop_factor * 0.3;
    let effective_rate = base_rate * happiness_factor * (1.0 - growth_reduction) * job_availability;
    let immigration_bonus = (current_population as f32 * effective_rate) as i64;

    base_target + immigration_bonus
}

/// Adjust population once per in‑game day based on available housing, happiness, and jobs
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

    let housing_cap = services.housing_capacity.max(0);
    let current_pop = population.population.max(0);

    let target_population = calculate_population_target(
        housing_cap,
        current_pop,
        population.happiness,
        services.job_capacity,
    );

    let max_population = if housing_cap == 0 { 0 } else { target_population };

    let diff = max_population - current_pop;

    if diff == 0 {
        return;
    }

    let mut step = ((diff as f32) * 0.35).round() as i64;
    if step == 0 {
        step = diff.signum();
    }

    let max_daily_growth = if diff > 0 && population.happiness > 0.8 {
        8
    } else {
        5
    };

    step = step.clamp(-max_daily_growth * 2, max_daily_growth);

    let old_pop = current_pop;
    population.population += step;
    population.population = population.population.max(0);

    let immigration_driven = target_population > housing_cap;
    if immigration_driven {
        info!(
            "Population changed from {} to {} (target {} includes {} immigration bonus, housing_capacity {}, job_capacity {}, happiness {:.2})",
            old_pop,
            population.population,
            target_population,
            target_population - housing_cap,
            housing_cap,
            services.job_capacity,
            population.happiness
        );
    } else {
        info!(
            "Population changed from {} to {} (target {}, housing_capacity {}, job_capacity {})",
            old_pop,
            population.population,
            max_population,
            housing_cap,
            services.job_capacity
        );
    }
}

pub fn update_demands(mut services: ResMut<CityServices>, population: Res<CityPopulation>) {
    let pop = population.population.max(0);

    let mut happy_growth_bonus: i64 = 0;
    if pop > 0 && population.happiness >= 0.9 {
        let happiness_factor = ((population.happiness - 0.9) / 0.1).clamp(0.0, 1.0);
        happy_growth_bonus = ((pop as f32 * 0.02) * happiness_factor).ceil() as i64;
    }

    services.housing_demand =
        (pop + happy_growth_bonus - services.housing_capacity).max(0);
    services.job_demand = (pop - services.job_capacity).max(0);
    services.entertainment_demand = (pop - services.entertainment_capacity).max(0);
}

/// Recompute happiness from current demands once per in‑game day.
/// This nudges happiness toward a target instead of overwriting it,
/// so short‑term events (demolition, budget issues, etc.) can have
/// a visible effect that slowly recovers.
pub fn update_happiness_from_demands(
    clock: Res<GameClock>,
    services: Res<CityServices>,
    mut population: ResMut<CityPopulation>,
    mut last_processed_day: Local<u32>,
) {
    if *last_processed_day == clock.day {
        return;
    }
    *last_processed_day = clock.day;

    let pop = population.population.max(0);
    let old_happiness = population.happiness;

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
    let entertainment_shortfall = if pop > 0 {
        services.entertainment_demand as f32 / pop_f
    } else {
        0.0
    };

    // Weight shortages; housing and jobs hurt more than entertainment.
    let mut pressure = 0.8 * housing_pressure
        + 0.6 * job_pressure
        + 0.25 * entertainment_shortfall;
    // Cap extreme pressure so it doesn't explode numerically
    if pressure > 1.5 {
        pressure = 1.5;
    }

    let target_happiness = (1.0 - pressure).clamp(0.0, 1.0);
    let new_happiness = old_happiness + (target_happiness - old_happiness) * 0.375;
    population.happiness = new_happiness.clamp(0.0, 1.0);

    if new_happiness + 1e-4 < old_happiness {
        info!(
            "Happiness decreased from {:.3} to {:.3} due to service pressures: housing={:.3}, jobs={:.3}, entertainment_shortfall={:.3}",
            old_happiness, new_happiness, housing_pressure, job_pressure, entertainment_shortfall
        );
    }
}

/// Periodically abandon buildings based on happiness and service pressures
pub fn apply_abandonment(
    clock: Res<GameClock>,
    population: Res<CityPopulation>,
    services: Res<CityServices>,
    tile_storage_q: Query<(&TileStorage, &TilemapSize)>,
    mut tile_texture_q: Query<&mut TileTextureIndex>,
    mut demolished_writer: MessageWriter<BuildingDemolished>,
    mut last_abandonment_day: Local<u32>,
    mut commands: Commands,
    building_sprites_q: Query<
        (
            Entity,
            Option<&'static ResidentialBuilding>,
            Option<&'static CommercialBuilding>,
            Option<&'static IndustryBuilding>,
        ),
    >,
) {
    const ABANDONMENT_INTERVAL_DAYS: u32 = 3;
    // If people are reasonably happy skip abandonment
    const MIN_HAPPINESS_FOR_ABANDON: f32 = 0.7;

    if clock.day < *last_abandonment_day + ABANDONMENT_INTERVAL_DAYS {
        return;
    }
    *last_abandonment_day = clock.day;

    let (tile_storage, _) = if let Some(v) = tile_storage_q.iter().next() {
        v
    } else {
        return;
    };

    let pop = population.population.max(0);
    if pop == 0 {
        return;
    }
    if population.happiness >= MIN_HAPPINESS_FOR_ABANDON {
        return;
    }

    let housing_shortage = services.housing_demand > 0;

    let job_cap = services.job_capacity.max(0);
    let jobs_f = job_cap as f32;
    let pop_f = pop as f32;
    let employed = pop_f.min(jobs_f);
    // Unhappy workers effectively "quit", reducing staffing
    let effective_workers = employed * population.happiness.clamp(0.0, 1.0);
    let staffing_ratio = if jobs_f > 0.0 {
        effective_workers / jobs_f
    } else {
        1.0
    };
    let job_understaffed = job_cap > 0 && staffing_ratio < 0.6;

    if !housing_shortage && !job_understaffed {
        return;
    }

    info!(
        "Abandonment tick: pop={}, happiness={:.2}, housing_demand={}, job_capacity={}, effective_workers={:.1}, staffing_ratio={:.2}",
        pop,
        population.happiness,
        services.housing_demand,
        job_cap,
        effective_workers,
        staffing_ratio
    );

    let residential_to_abandon: usize = if housing_shortage { 1 } else { 0 };
    let job_capacity_to_remove: i64 = if job_understaffed { 1 } else { 0 };

    if residential_to_abandon > 0 {
        let mut remaining = residential_to_abandon;

        for (entity, residential, _commercial, _industry) in building_sprites_q.iter() {
            if remaining == 0 {
                break;
            }

            if let Some(building) = residential {
                let pos = building.tile_pos;
                if let Some(tile_entity) = tile_storage.get(&pos)
                    && let Ok(mut texture) = tile_texture_q.get_mut(tile_entity)
                {
                    texture.0 = ABANDONED_TEXTURE_INDEX;
                    info!("Abandoned residential at {:?}", pos);
                    demolished_writer.write(BuildingDemolished {
                        building_type: BuildingType::Residential,
                        tile_pos: pos,
                    });
                    commands.entity(entity).despawn();
                    remaining -= 1;
                }
            }
        }
    }

    // abandon commercial/industry tiles until we've removed enough job capacity
    if job_capacity_to_remove > 0 {
        let mut remaining_jobs = job_capacity_to_remove;

        for (entity, _residential, commercial, industry) in building_sprites_q.iter() {
            if remaining_jobs <= 0 {
                break;
            }

            let (pos, btype) = if let Some(b) = commercial {
                (b.tile_pos, BuildingType::Commercial)
            } else if let Some(b) = industry {
                (b.tile_pos, BuildingType::Industry)
            } else {
                continue;
            };

            let contrib = building_contribution(btype);
            if contrib.jobs <= 0 {
                continue;
            }

            if let Some(tile_entity) = tile_storage.get(&pos)
                && let Ok(mut texture) = tile_texture_q.get_mut(tile_entity)
            {
                texture.0 = ABANDONED_TEXTURE_INDEX;
                info!("Abandoned {:?} at {:?}", btype, pos);
                demolished_writer.write(BuildingDemolished {
                    building_type: btype,
                    tile_pos: pos,
                });
                commands.entity(entity).despawn();
                remaining_jobs -= contrib.jobs;
            }
        }
    }
}
