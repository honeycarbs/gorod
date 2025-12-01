use bevy::prelude::*;
mod display;
mod resources;
mod systems;

pub use resources::GameClock;

pub struct GameTimePlugin;

impl Plugin for GameTimePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<resources::GameTime>()
            .init_resource::<resources::GameClock>()
            .add_systems(Startup, display::setup_time_display)
            .add_systems(
                Update,
                (
                    systems::update_game_time,
                    systems::update_game_clock.after(systems::update_game_time),
                    systems::handle_time_speed_input,
                    display::update_time_display.after(systems::update_game_clock),
                ),
            );
    }
}
