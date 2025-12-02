use bevy::prelude::*;

mod demolition;
mod display;
mod events;
mod helpers;
mod highlighting;
mod placeable_area;
mod placement;
mod resources;

pub use resources::*;

pub struct TilePlacementPlugin;

impl Plugin for TilePlacementPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CursorWorldPos>()
            .init_resource::<CurrentTileType>()
            .init_resource::<PlaceableMap>()
            .add_message::<events::PlacementIntent>()
            .add_systems(Startup, display::setup_selected_tile_display)
            .add_systems(First, resources::update_cursor_world_pos)
            .add_systems(
                Update,
                (
                    placement::change_tile_type,
                    highlighting::highlight_hovered_tile,
                    placement::collect_placement_intents,
                    placement::execute_placement_intents,
                    demolition::demolish_tile_on_click,
                    placeable_area::expand_placeable_area,
                    placeable_area::update_placeable_indicators,
                )
                    .chain(),
            )
            .add_systems(Update, display::update_selected_tile_display);
    }
}

