use bevy::prelude::*;

use super::resources::*;

// Update game time based on speed mode
pub fn update_game_time(
    time: Res<Time>,
    mut game_time: ResMut<GameTime>,
) {
    let multiplier = match game_time.speed {
        TimeSpeed::Paused => 0.0,
        TimeSpeed::Normal => 60.0,
        TimeSpeed::Fast => 120.0,
        TimeSpeed::UltraFast => 240.0,
    };

    game_time.elapsed_seconds += time.delta_secs_f64() * multiplier;
}

pub fn update_game_clock(
    game_time: Res<GameTime>,
    mut clock: ResMut<GameClock>,
) {
    let total_seconds = game_time.elapsed_seconds as u64;
    clock.second = (total_seconds % 60) as u8;
    clock.minute = ((total_seconds / 60) % 60) as u8;
    clock.hour = ((total_seconds / 3600) % 24) as u8;
    clock.day = (total_seconds / 86400) as u32 + 1;
}

pub fn handle_time_speed_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut game_time: ResMut<GameTime>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        game_time.speed = if game_time.speed == TimeSpeed::Paused {
            TimeSpeed::Normal
        } else {
            TimeSpeed::Paused
        };
    }

    if keyboard.just_pressed(KeyCode::Digit1) {
        game_time.speed = TimeSpeed::Normal;
    }
    if keyboard.just_pressed(KeyCode::Digit2) {
        game_time.speed = TimeSpeed::Fast;
    }
    if keyboard.just_pressed(KeyCode::Digit3) {
        game_time.speed = TimeSpeed::UltraFast;
    }
}