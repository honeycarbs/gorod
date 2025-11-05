/// Manages in-game time with scaling support
#[derive(Debug, Clone)]
pub struct GameTime {
    elapsed_game_seconds: f64,

    /// How fast game time progresses relative to real time
    /// 1.0 = real-time, 2.0 = 2x speed, 0.5 = half speed
    time_scale: f64,
    /// Whether time is currently paused
    paused: bool,
}

impl GameTime {
    /// Create a new game time manager starting at day 0
    pub fn new() -> Self {
        Self {
            elapsed_game_seconds: 0.0,
            time_scale: 60.0,
            paused: false,
        }
    }

    /// Create a new game time with a specific time scale
    pub fn with_scale(time_scale: f64) -> Self {
        Self {
            elapsed_game_seconds: 0.0,
            time_scale,
            paused: false,
        }
    }

    /// Advance time by the given delta
    pub fn tick(&mut self, delta_seconds: f64) {
        if !self.paused {
            self.elapsed_game_seconds += delta_seconds * self.time_scale;
        }
    }

    /// Set the time scale
    pub fn set_time_scale(&mut self, scale: f64) {
        self.time_scale = scale.max(0.0);
    }

    /// Get the current time scale
    pub fn time_scale(&self) -> f64 {
        self.time_scale
    }

    /// Pause time progression
    pub fn pause(&mut self) {
        self.paused = true;
    }

    /// Resume time progression
    pub fn resume(&mut self) {
        self.paused = false;
    }

    /// Toggle pause state
    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }

    /// Check if time is paused
    pub fn is_paused(&self) -> bool {
        self.paused
    }

    /// Get total elapsed game time in seconds
    pub fn elapsed_seconds(&self) -> f64 {
        self.elapsed_game_seconds
    }

    /// Get elapsed time as days, hours, minutes
    pub fn as_dhm(&self) -> (u32, u32, u32) {
        let total_seconds = self.elapsed_game_seconds as u64;
        let days = total_seconds / 86400; // 24 * 60 * 60
        let hours = (total_seconds % 86400) / 3600;
        let minutes = (total_seconds % 3600) / 60;
        (days as u32, hours as u32, minutes as u32)
    }

    /// Format the current game time for display
    pub fn display(&self) -> String {
        let (days, hours, minutes) = self.as_dhm();
        let status = if self.paused { " [PAUSED]" } else { "" };
        format!(
            "Day {}, {:02}:{:02} ({}x speed){}",
            days, hours, minutes, self.time_scale, status
        )
    }
}

impl Default for GameTime {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_progression() {
        let mut time = GameTime::new();

        // Simulate 1 hour of game time (3600 seconds)
        time.tick(3600.0);

        let (days, hours, minutes) = time.as_dhm();
        assert_eq!(days, 0);
        assert_eq!(hours, 1);
        assert_eq!(minutes, 0);
    }

    #[test]
    fn test_time_scale() {
        let mut time = GameTime::with_scale(2.0); // 2x speed

        // 1 second real time = 2 seconds game time
        time.tick(1800.0); // 30 minutes real time

        let (_days, hours, _minutes) = time.as_dhm();
        assert_eq!(hours, 1); // Should be 1 hour game time
    }

    #[test]
    fn test_pause() {
        let mut time = GameTime::new();
        time.pause();

        time.tick(3600.0);
        assert_eq!(time.elapsed_seconds(), 0.0);

        time.resume();
        time.tick(3600.0);
        assert_eq!(time.elapsed_seconds(), 3600.0);
    }
}
