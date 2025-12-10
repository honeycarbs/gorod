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
            .init_resource::<CurrentRoadVariant>()
            .init_resource::<CurrentResidentialVariant>()
            .init_resource::<CurrentCommercialVariant>()
            .init_resource::<CurrentIndustryVariant>()
            .init_resource::<PreviewVariant>()
            .init_resource::<PlaceableMap>()
            .init_resource::<UiClickBlocker>()
            .add_message::<events::PlacementIntent>()
            .add_systems(
                Startup,
                (
                    display::setup_selected_tile_display,
                    display::setup_tile_select_buttons,
                    resources::setup_residential_building_atlas,
                    resources::setup_commercial_building_atlas,
                    resources::setup_industry_building_atlas,
                    resources::setup_roads_atlas,
                    resources::setup_tile_preview_atlas,
                ),
            )
            .add_systems(
                First,
                (
                    resources::update_cursor_world_pos,
                    resources::reset_ui_click_blocker,
                ),
            )
            .add_systems(
                Update,
                (
                    placement::change_tile_type,
                    highlighting::highlight_hovered_tile,
                    highlighting::update_road_hover_preview,
                    placement::collect_placement_intents,
                    placement::execute_placement_intents,
                    demolition::demolish_tile_on_click,
                    placeable_area::expand_placeable_area,
                    placeable_area::update_placeable_indicators,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                (
                    display::update_selected_tile_display,
                    display::handle_tile_select_button_presses,
                    display::update_tile_select_button_colors,
                ),
            );
    }
}
