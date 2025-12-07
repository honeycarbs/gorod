use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::{TilePos, TileStorage, TileTextureIndex, TilemapSize};

use crate::budget::{BuildingDemolished, BuildingPlaced, BuildingType};
use crate::map::ABANDONED_TEXTURE_INDEX;
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

/// Adjust population once per in‑game day based on available housing and jobs
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
    let job_cap = services.job_capacity.max(0);

    // If there are no jobs at all people gradually leave the city
    let max_population = if job_cap == 0 {
        0
    } else {
        housing_cap.min(job_cap)
    };

    let diff = max_population - population.population;

    if diff == 0 {
        return;
    }

    // Move population faster toward the target (about 40% of the gap per day)
    let mut step = ((diff as f32) * 0.40).round() as i64;
    if step == 0 {
        step = diff.signum();
    }

    let old_pop = population.population;
    population.population += step;
    population.population = population.population.max(0);
    info!(
        "Population changed from {} to {} (target {}, housing_capacity {}, job_capacity {})",
        old_pop, population.population, max_population, services.housing_capacity, services.job_capacity
    );
}

pub fn update_demands(mut services: ResMut<CityServices>, population: Res<CityPopulation>) {
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
    let entertainment_pressure = (entertainment_shortfall - 0.15).max(0.0);

    let pressure = 0.50 * housing_pressure + 0.35 * job_pressure + 0.15 * entertainment_pressure;

    let happiness = (1.0 - pressure).clamp(0.0, 1.0);
    population.happiness = happiness;

    if happiness + 1e-4 < old_happiness {
        info!(
            "Happiness decreased from {:.3} to {:.3} due to service pressures: housing={:.3}, jobs={:.3}, entertainment={:.3}",
            old_happiness, happiness, housing_pressure, job_pressure, entertainment_pressure
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
) {
    const ABANDONMENT_INTERVAL_DAYS: u32 = 7;
    // If people are reasonably happy skip abandonment
    const MIN_HAPPINESS_FOR_ABANDON: f32 = 0.5;

    if clock.day < *last_abandonment_day + ABANDONMENT_INTERVAL_DAYS {
        return;
    }
    *last_abandonment_day = clock.day;

    let (tile_storage, map_size) = if let Some(v) = tile_storage_q.iter().next() {
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
        pop, population.happiness, services.housing_demand, job_cap, effective_workers, staffing_ratio
    );

    let residential_to_abandon: usize = if housing_shortage { 1 } else { 0 };
    let job_capacity_to_remove: i64 = if job_understaffed { 1 } else { 0 };

    if residential_to_abandon > 0 {
        let mut remaining = residential_to_abandon;

        'outer_res: for x in 0..map_size.x {
            for y in 0..map_size.y {
                if remaining == 0 {
                    break 'outer_res;
                }

                let pos = TilePos { x, y };

                if let Some(entity) = tile_storage.get(&pos)
                    && let Ok(mut texture) = tile_texture_q.get_mut(entity)
                {
                    if texture.0 == ABANDONED_TEXTURE_INDEX {
                        continue;
                    }

                    if let Some(btype) = BuildingType::from_texture_index(texture.0)
                        && matches!(btype, BuildingType::Residential)
                    {
                        texture.0 = ABANDONED_TEXTURE_INDEX;
                        info!("Abandoned residential at {:?}", pos);
                        demolished_writer.write(BuildingDemolished {
                            building_type: btype,
                            tile_pos: pos,
                        });
                        remaining -= 1;
                    }
                }
            }
        }
    }

    // abandon commercial/industry tiles until we've removed enough job capacity
    if job_capacity_to_remove > 0 {
        let mut remaining_jobs = job_capacity_to_remove;

        'outer_jobs: for x in 0..map_size.x {
            for y in 0..map_size.y {
                if remaining_jobs <= 0 {
                    break 'outer_jobs;
                }

                let pos = TilePos { x, y };

                if let Some(entity) = tile_storage.get(&pos)
                    && let Ok(mut texture) = tile_texture_q.get_mut(entity)
                {
                    if texture.0 == ABANDONED_TEXTURE_INDEX {
                        continue;
                    }

                    if let Some(btype) = BuildingType::from_texture_index(texture.0)
                        && matches!(btype, BuildingType::Commercial | BuildingType::Industry)
                    {
                        let contrib = building_contribution(btype);
                        let jobs_here = contrib.jobs;

                        if jobs_here <= 0 {
                            continue;
                        }

                        texture.0 = ABANDONED_TEXTURE_INDEX;
                        info!("Abandoned {:?} at {:?}", btype, pos);
                        demolished_writer.write(BuildingDemolished {
                            building_type: btype,
                            tile_pos: pos,
                        });

                        remaining_jobs -= jobs_here;
                    }
                }
            }
        }
    }
}
