use bevy::prelude::*;

pub mod display;
mod events;
mod resources;
pub mod spending;

pub use events::{BuildingDemolished, BuildingPlaced, TransactionFailed};
pub use resources::{Budget, BuildingType};

pub struct BudgetPlugin;

impl Plugin for BudgetPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Budget>()
            .add_message::<BuildingPlaced>()
            .add_message::<BuildingDemolished>()
            .add_message::<TransactionFailed>()
            .add_systems(Startup, display::setup_budget_display)
            .add_systems(Update, display::update_budget_display);
    }
}
