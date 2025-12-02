use bevy::prelude::*;

#[derive(Resource)]
pub struct Budget {
    pub money: i64,
}

impl Default for Budget {
    fn default() -> Self {
        Self { money: 50000 }
    }
}

impl Budget {
    pub fn can_afford(&self, cost: i64) -> bool {
        self.money >= cost
    }

    pub fn spend(&mut self, amount: i64) -> bool {
        if self.can_afford(amount) {
            self.money -= amount;
            true
        } else {
            false
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildingType {
    Residential,
    Commercial,
    Industry,
    Road,
}

impl BuildingType {
    // Construction costs
    pub fn cost(&self) -> i64 {
        match self {
            BuildingType::Residential => 1000,
            BuildingType::Commercial => 1300,
            BuildingType::Industry => 2000,
            BuildingType::Road => 50,
        }
    }

    pub fn from_texture_index(index: u32) -> Option<Self> {
        match index {
            2 => Some(BuildingType::Residential),
            3 => Some(BuildingType::Commercial),
            4 => Some(BuildingType::Industry),
            5 => Some(BuildingType::Road),
            _ => None,
        }
    }
}
