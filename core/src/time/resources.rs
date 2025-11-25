use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TimeSpeed {
    Paused,
    #[default]
    Normal,
    Fast,
    UltraFast,
}

#[derive(Resource, Default)]
pub struct GameTime {
    pub elapsed_seconds: f64,
    pub speed: TimeSpeed,
}

#[derive(Resource, Default)]
pub struct GameClock {
    pub day: u32,
    pub hour: u8,    // 0-23
    pub minute: u8,  // 0-59
    pub second: u8,  // 0-59
}