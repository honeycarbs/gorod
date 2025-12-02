use bevy::prelude::*;

use crate::city::resources::{CityInfrastructure, CityPopulation, CityServices};
use crate::time::GameClock;

use super::resources::Budget;

// income per productive worker per day
const WORKER_TAX_PER_DAY: i64 = 4;

// corporate income per productive worker share
const INDUSTRY_PROFIT_PER_WORKER: f32 = 1.5;
const COMMERCIAL_PROFIT_PER_WORKER: f32 = 1.0;

// upkeep per building per day
const ROAD_UPKEEP_PER_TILE: i64 = 1;
const RES_UPKEEP_PER_BUILDING: i64 = 0;
const COM_UPKEEP_PER_BUILDING: i64 = 3;
const IND_UPKEEP_PER_BUILDING: i64 = 5;

const NEGATIVE_BALANCE_PENALTY_DAYS: u32 = 3;
const NEGATIVE_BALANCE_HAPPINESS_PENALTY: f32 = 0.01;
const HEALTHY_RESERVE_THRESHOLD: i64 = 100_000;
const HEALTHY_RESERVE_HAPPINESS_BONUS: f32 = 0.005;

/// derive periodic income/upkeep and modify `Budget` once per in‑game day
pub fn update_income_on_day_tick(
    clock: Res<GameClock>,
    mut population: ResMut<CityPopulation>,
    services: Res<CityServices>,
    infra: Res<CityInfrastructure>,
    mut budget: ResMut<Budget>,
    mut last_income_day: Local<u32>,
    mut negative_streak: Local<u32>,
) {
    if *last_income_day == clock.day {
        return;
    }
    *last_income_day = clock.day;

    let pop = population.population.max(0);
    let jobs = services.job_capacity.max(0);

    let employed = pop.min(jobs);

    let productive_workers =
        ((employed as f32) * population.happiness.clamp(0.0, 1.0)).round() as i64;

    let income_from_workers = productive_workers * WORKER_TAX_PER_DAY;

    // income sharing between industry and commercial
    let mut corp_income: f32 = 0.0;
    if services.job_capacity > 0 {
        let total_jobs = services.job_capacity as f32;
        let industry_share = (infra.industry_job_capacity as f32 / total_jobs).clamp(0.0, 1.0);
        let commercial_share = (infra.commercial_job_capacity as f32 / total_jobs).clamp(0.0, 1.0);

        corp_income += productive_workers as f32 * industry_share * INDUSTRY_PROFIT_PER_WORKER;
        corp_income += productive_workers as f32 * commercial_share * COMMERCIAL_PROFIT_PER_WORKER;
    }
    let income_from_corporations = corp_income.round() as i64;

    let total_income = income_from_workers + income_from_corporations;

    let road_upkeep = infra.road_count * ROAD_UPKEEP_PER_TILE;
    let residential_upkeep = infra.residential_count * RES_UPKEEP_PER_BUILDING;
    let commercial_upkeep = infra.commercial_count * COM_UPKEEP_PER_BUILDING;
    let industry_upkeep = infra.industry_count * IND_UPKEEP_PER_BUILDING;

    let upkeep = road_upkeep + residential_upkeep + commercial_upkeep + industry_upkeep;

    let net = total_income - upkeep;
    budget.money += net;

    // track negative streak for soft happiness penalties when running deficits
    if budget.money < 0 {
        *negative_streak += 1;
    } else {
        *negative_streak = 0;
    }

    if *negative_streak >= NEGATIVE_BALANCE_PENALTY_DAYS {
        let old = population.happiness;
        population.happiness =
            (population.happiness - NEGATIVE_BALANCE_HAPPINESS_PENALTY).clamp(0.0, 1.0);
        info!(
            "Happiness decreased from {:.3} to {:.3} due to running a budget deficit for {} days",
            old, population.happiness, *negative_streak
        );
    }

    if budget.money > HEALTHY_RESERVE_THRESHOLD
        && services.housing_demand == 0
        && services.job_demand == 0
        && services.entertainment_demand == 0
    {
        let old = population.happiness;
        population.happiness =
            (population.happiness + HEALTHY_RESERVE_HAPPINESS_BONUS).clamp(0.0, 1.0);
        info!(
            "Happiness increased from {:.3} to {:.3} due to healthy budget reserves",
            old, population.happiness
        );
    }
}
