#[derive(Debug, Clone)]
pub struct GameTime {
    pub time: f64,
    pub speed: f32,
    pub day_length: f32,
    pub is_paused: bool,
}

impl GameTime {
    pub fn new(day_length: f32) -> Self {
        Self {
            time: 0.0,
            speed: 1.0,
            day_length,
            is_paused: false,
        }
    }

    pub fn toggle_pause(&mut self) {
        self.is_paused = !self.is_paused;
    }

    pub fn set_speed(&mut self, speed: f32) {
        self.speed = speed.max(0.0);
    }

    pub fn current_day(&self) -> u32 {
        (self.time / self.day_length as f64) as u32
    }

    pub fn time_of_day(&self) -> f32 {
        (self.time % self.day_length as f64) as f32 / self.day_length
    }
}

#[derive(Debug, Clone, Default)]
pub struct CityStats {
    // TODO
}

#[derive(Debug, Clone)]
pub struct CityGrid {
    pub width: u32,
    pub height: u32,
}
