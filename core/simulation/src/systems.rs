use crate::ecs::World;
use crate::resources::{CityStats, GameTime};

pub fn clock_system(world: &mut World, delta_time: f32) {
    if let Some(clock) = world.get_resource_mut::<GameTime>()
        && !clock.is_paused
    {
        clock.time += (delta_time * clock.speed) as f64;
    }
}

pub fn city_stats_system(world: &mut World) {
    let stats = CityStats::default();
    // TODO: all the gameplay stuff
    world.insert_resource(stats);
}
