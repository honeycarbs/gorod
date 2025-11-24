use bevy::prelude::*;

mod resources;
mod placement;
mod demolition;
mod placeable_area;
mod highlighting;
mod helpers;

pub use resources::*;

pub struct TilePlacementPlugin;

impl Plugin for TilePlacementPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CursorWorldPos>()
            .init_resource::<CurrentTileType>()
            .init_resource::<PlaceableMap>()
            .add_systems(First, resources::update_cursor_world_pos)
            .add_systems(Update, (
                placement::change_tile_type,
                highlighting::highlight_hovered_tile,
                placement::place_tile_on_click,
                demolition::demolish_tile_on_click,
                placeable_area::expand_placeable_area,
                placeable_area::update_placeable_indicators,
            ).chain());
    }
}