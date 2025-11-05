/// Represents the city state
#[derive(Debug, Clone)]
pub struct City {
    pub name: String,
    pub time: crate::time::GameTime,
}

impl City {
    /// Create a new city with a given name
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            time: crate::time::GameTime::new(),
        }
    }

    /// Update the city simulation
    ///
    /// `delta_seconds`: amount of in-game time that has passed
    pub fn update(&mut self, delta_seconds: f64) {
        // Advance game time
        self.time.tick(delta_seconds);

        // TODO add population updates, happiness calculations
    }

    /// Get a display string for the city's current state
    pub fn display_status(&self) -> String {
        format!("{}/{}", self.name, self.time.display())
    }
}
