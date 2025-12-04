use bevy::math::UVec2;
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use bevy_image::TextureAtlasLayout;
use std::collections::HashSet;

/// Texture index for the abandoned (grey) tile in `tiles.png`.
pub const ABANDONED_TEXTURE_INDEX: u32 = 6;

#[derive(Resource, Default)]
pub struct CursorWorldPos(pub Vec2);

#[derive(Component)]
pub struct HighlightedTile;

#[derive(Resource, Default)]
pub struct UiClickBlocker {
    pub just_clicked_ui: bool,
}

#[derive(Resource)]
pub struct CurrentTileType {
    pub texture_index: u32,
}

impl Default for CurrentTileType {
    fn default() -> Self {
        Self {
            texture_index: 5, // Road by default
        }
    }
}

#[derive(Resource, Default)]
pub struct PlaceableMap {
    pub placeable_tiles: HashSet<TilePos>,
}

impl PlaceableMap {
    pub fn is_placeable(&self, pos: &TilePos) -> bool {
        self.placeable_tiles.contains(pos)
    }

    pub fn mark_placeable(&mut self, pos: TilePos) {
        self.placeable_tiles.insert(pos);
    }
}

pub fn update_cursor_world_pos(
    camera_q: Query<(&GlobalTransform, &Camera)>,
    mut cursor_moved_events: MessageReader<CursorMoved>,
    mut cursor_pos: ResMut<CursorWorldPos>,
) {
    for cursor_moved in cursor_moved_events.read() {
        for (cam_transform, cam) in camera_q.iter() {
            if let Ok(world_pos) = cam.viewport_to_world_2d(cam_transform, cursor_moved.position) {
                cursor_pos.0 = world_pos;
            }
        }
    }
}

pub fn reset_ui_click_blocker(mut blocker: ResMut<UiClickBlocker>) {
    blocker.just_clicked_ui = false;
}

/// Number of residential building variants in the residential sprite sheet.
pub const RESIDENTIAL_VARIANT_COUNT: usize = 5;

#[derive(Resource)]
pub struct ResidentialBuildingAtlas {
    pub texture: Handle<Image>,
    pub layout: Handle<TextureAtlasLayout>,
    pub variants: usize,
}

#[derive(Component)]
pub struct ResidentialBuilding {
    pub tile_pos: TilePos,
}

pub fn setup_residential_building_atlas(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let texture: Handle<Image> = asset_server.load("houses.png");

    let layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(96, 96),
        1,
        RESIDENTIAL_VARIANT_COUNT as u32,
        None,
        None,
    ));

    commands.insert_resource(ResidentialBuildingAtlas {
        texture,
        layout,
        variants: RESIDENTIAL_VARIANT_COUNT,
    });
}
