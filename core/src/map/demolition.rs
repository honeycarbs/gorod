use super::helpers::*;
use super::placeable_area::incremental_update_placeable_area;
use super::resources::*;
use crate::budget::{BuildingDemolished, BuildingType};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

type BuildingSpritesQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        Option<&'static ResidentialBuilding>,
        Option<&'static CommercialBuilding>,
        Option<&'static IndustryBuilding>,
        Option<&'static RoadSegment>,
    ),
>;

#[derive(SystemParam)]
pub struct DemolitionInputs<'w, 's> {
    mouse_button: Res<'w, ButtonInput<MouseButton>>,
    keyboard: Res<'w, ButtonInput<KeyCode>>,
    cursor_pos: Res<'w, CursorWorldPos>,
    ui_click_blocker: Res<'w, UiClickBlocker>,
    tilemap_q: Query<
        'w,
        's,
        (
            &'static TilemapSize,
            &'static TilemapGridSize,
            &'static TilemapTileSize,
            &'static TilemapType,
            &'static TileStorage,
            &'static Transform,
            &'static TilemapAnchor,
        ),
    >,
    tile_texture_q: Query<'w, 's, &'static mut TileTextureIndex>,
}

pub fn demolish_tile_on_click(
    mut inputs: DemolitionInputs,
    mut placeable_map: ResMut<PlaceableMap>,
    mut demolished_writer: MessageWriter<BuildingDemolished>,
    mut commands: Commands,
    building_sprites_q: BuildingSpritesQuery<'_, '_>,
) {
    if !inputs.mouse_button.just_pressed(MouseButton::Left) {
        return;
    }

    if inputs.ui_click_blocker.just_clicked_ui {
        return;
    }

    if !inputs.keyboard.pressed(KeyCode::ShiftLeft) && !inputs.keyboard.pressed(KeyCode::ShiftRight)
    {
        return;
    }

    for (map_size, grid_size, tile_size, map_type, tile_storage, map_transform, anchor) in
        inputs.tilemap_q.iter()
    {
        let cursor_in_map_pos = cursor_to_map_pos(inputs.cursor_pos.0, map_transform);

        if let Some(tile_pos) = TilePos::from_world_pos(
            &cursor_in_map_pos,
            map_size,
            grid_size,
            tile_size,
            map_type,
            anchor,
        ) && let Some(tile_entity) = tile_storage.get(&tile_pos)
        {
            let current_texture = if let Ok(texture) = inputs.tile_texture_q.get(tile_entity) {
                texture.0
            } else {
                return;
            };

            if current_texture >= 2 {
                let placed_tile_count =
                    count_placed_tiles(tile_storage, &inputs.tile_texture_q, map_size);

                if placed_tile_count <= 1 {
                    warn!("Cannot demolish the last tile!");
                    return;
                }

                let should_be_placeable =
                    is_within_range_of_placed_tile(&tile_pos, tile_storage, &inputs.tile_texture_q, map_size);

                if let Ok(mut texture_index) = inputs.tile_texture_q.get_mut(tile_entity) {
                    if should_be_placeable {
                        texture_index.0 = 1;
                    } else {
                        texture_index.0 = 0;
                        placeable_map.placeable_tiles.remove(&tile_pos);
                    }

                    if let Some(building_type) = BuildingType::from_texture_index(current_texture) {
                        demolished_writer.write(BuildingDemolished {
                            building_type,
                            tile_pos,
                        });
                    }

                    info!("Demolished tile at {:?}", tile_pos);

                    incremental_update_placeable_area(
                        tile_pos,
                        &mut placeable_map,
                        tile_storage,
                        &inputs.tile_texture_q,
                        map_size,
                    );

                    for (entity, residential, commercial, industry, road) in
                        building_sprites_q.iter()
                    {
                        let mut should_despawn = false;

                        if let Some(building) = residential
                            && building.tile_pos == tile_pos
                        {
                            should_despawn = true;
                        }

                        if let Some(building) = commercial
                            && building.tile_pos == tile_pos
                        {
                            should_despawn = true;
                        }

                        if let Some(building) = industry
                            && building.tile_pos == tile_pos
                        {
                            should_despawn = true;
                        }

                        if let Some(road_segment) = road
                            && road_segment.tile_pos == tile_pos
                        {
                            should_despawn = true;
                        }

                        if should_despawn {
                            commands.entity(entity).despawn();
                        }
                    }
                }
            }
        }
    }
}
