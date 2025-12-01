use bevy::prelude::*;

use crate::budget::{Budget, BuildingDemolished, BuildingPlaced};
use crate::time::GameClock;

use super::display::{setup_city_stats_display, update_city_stats_display};
use super::resources::{CityStats, apply_demolition_happiness, building_contribution, apply_placement_happiness};

pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CityStats>()
            .add_systems(Startup, setup_city_stats_display)
            .add_systems(Update, handle_building_events)
            .add_systems(Update, (update_population, update_demands_and_happiness))
            .add_systems(
                Update,
                apply_placement_happiness.after(update_demands_and_happiness),
            )
            .add_systems(
                Update,
                apply_demolition_happiness.after(apply_placement_happiness),
            )
            .add_systems(Update, (update_income_on_day_tick, update_city_stats_display));
    }
}

/// reacts to `BuildingPlaced` / `BuildingDemolished` to keep city capacities in sync
pub fn handle_building_events(
    mut stats: ResMut<CityStats>,
    mut placed_reader: MessageReader<BuildingPlaced>,
    mut demolished_reader: MessageReader<BuildingDemolished>,
) {
    for event in placed_reader.read() {
        let contrib = building_contribution(event.building_type);
        stats.housing_capacity += contrib.housing;
        stats.job_capacity += contrib.jobs;
        stats.entertainment_capacity += contrib.entertainment;
    }

    for event in demolished_reader.read() {
        let contrib = building_contribution(event.building_type);
        stats.housing_capacity -= contrib.housing;
        stats.job_capacity -= contrib.jobs;
        stats.entertainment_capacity -= contrib.entertainment;
    }

    // clamp to avoid negative capacities.
    stats.housing_capacity = stats.housing_capacity.max(0);
    stats.job_capacity = stats.job_capacity.max(0);
    stats.entertainment_capacity = stats.entertainment_capacity.max(0);
}

/// Adjust population based primarily on housing capacity once per in‑game day.
///
/// - Housing drives how many people can potentially live in the city.
/// - If there are not enough jobs/entertainment for that many people,
///   demands will become positive and happiness will drop.
pub fn update_population(
    clock: Res<GameClock>,
    mut stats: ResMut<CityStats>,
    mut last_processed_day: Local<u32>,
) {
    if *last_processed_day == clock.day {
        return;
    }
    *last_processed_day = clock.day;

    // drive population towards available housing, independent of jobs
    let target = stats.housing_capacity.max(0);
    let diff = target - stats.population;

    if diff == 0 {
        return;
    }

    let step = ((diff as f32) * 0.25).round() as i64;
    let step = if step == 0 { diff.signum() } else { step };

    stats.population += step;
    stats.population = stats.population.max(0);
}

/// recompute demand and happiness from `CityStats`
pub fn update_demands_and_happiness(mut stats: ResMut<CityStats>) {
    let pop = stats.population.max(0);

    stats.housing_demand = (pop - stats.housing_capacity).max(0);
    stats.job_demand = (pop - stats.job_capacity).max(0);
    stats.entertainment_demand = (pop - stats.entertainment_capacity).max(0);

    let pop_f = pop as f32;
    let housing_pressure = if pop > 0 {
        stats.housing_demand as f32 / pop_f
    } else {
        0.0
    };
    let job_pressure = if pop > 0 {
        stats.job_demand as f32 / pop_f
    } else {
        0.0
    };
    let entertainment_pressure = if pop > 0 {
        stats.entertainment_demand as f32 / pop_f
    } else {
        0.0
    };

    let pressure = 0.5 * housing_pressure + 0.3 * job_pressure + 0.2 * entertainment_pressure;

    let happiness = (1.0 - pressure).clamp(0.0, 1.0);
    stats.happiness = happiness;
}

/// derive periodic income/upkeep and modify `Budget` once per in‑game day
pub fn update_income_on_day_tick(
    clock: Res<GameClock>,
    stats: Res<CityStats>,
    mut budget: ResMut<Budget>,
    mut last_income_day: Local<u32>,
) {
    if *last_income_day == clock.day {
        return;
    }
    *last_income_day = clock.day;

    let pop = stats.population.max(0);
    let jobs = stats.job_capacity.max(0);

    let income_from_population = pop * 5;
    let income_from_jobs = jobs * 2;

    let total_capacity = stats.housing_capacity + stats.job_capacity + stats.entertainment_capacity;
    let upkeep = (total_capacity / 10).max(0);

    let net = income_from_population + income_from_jobs - upkeep;
    budget.money += net;
}
