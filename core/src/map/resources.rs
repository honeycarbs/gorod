use bevy::math::UVec2;
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use bevy_image::TextureAtlasLayout;
use std::collections::HashSet;

/// Texture index for abandoned/demolished tiles in `tiles.png`
pub const ABANDONED_TEXTURE_INDEX: u32 = 4;

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
        Self { texture_index: 0 }
    }
}

/// Tracks which road sprite in `roads.png` is currently selected
#[derive(Resource, Default)]
pub struct CurrentRoadVariant {
    pub index: u32,
}

/// Tracks which commercial building sprite in `commercial.png` is currently selected
#[derive(Resource, Default)]
pub struct CurrentCommercialVariant {
    pub index: u32,
}

/// Tracks which industry building sprite in `factory.png` is currently selected
#[derive(Resource, Default)]
pub struct CurrentIndustryVariant {
    pub index: u32,
}

/// Tracks which decorative building sprite in `decorative.png` is currently selected
#[derive(Resource, Default)]
pub struct CurrentDecorativeVariant {
    pub index: u32,
}

/// Stores the random variant shown in preview for each building type
#[derive(Resource, Default)]
pub struct PreviewVariant {
    pub residential: Option<usize>,
    pub commercial: Option<usize>,
    pub industry: Option<usize>,
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

/// Number of residential building variants in the residential sprite sheet
pub const RESIDENTIAL_VARIANT_COUNT: usize = 5;

/// Number of commercial building variants in the commercial sprite sheet
pub const COMMERCIAL_VARIANT_COUNT: usize = 4;

/// Number of industry/factory building variants in the factory sprite sheet
pub const INDUSTRY_VARIANT_COUNT: usize = 2;

/// Number of road variants in the `roads.png` sprite sheet
pub const ROAD_VARIANT_COUNT: usize = 11;

/// Number of decorative building variants in the `decorative.png` sprite sheet
pub const DECORATIVE_VARIANT_COUNT: usize = 4;

/// Number of tile variants in the `tiles.png` sprite sheet used for previews
pub const TILE_PREVIEW_VARIANT_COUNT: usize = 5;

#[derive(Resource)]
pub struct ResidentialBuildingAtlas {
    pub texture: Handle<Image>,
    pub layout: Handle<TextureAtlasLayout>,
    pub variants: usize,
}

#[derive(Resource)]
pub struct CommercialBuildingAtlas {
    pub texture: Handle<Image>,
    pub layout: Handle<TextureAtlasLayout>,
    pub variants: usize,
}

#[derive(Resource)]
pub struct IndustryBuildingAtlas {
    pub texture: Handle<Image>,
    pub layout: Handle<TextureAtlasLayout>,
    pub variants: usize,
}

#[derive(Resource)]
pub struct RoadAtlas {
    pub texture: Handle<Image>,
    pub layout: Handle<TextureAtlasLayout>,
    pub variants: usize,
}

#[derive(Resource)]
pub struct DecorativeBuildingAtlas {
    pub texture: Handle<Image>,
    pub layout: Handle<TextureAtlasLayout>,
    pub variants: usize,
}

#[derive(Resource)]
pub struct TilePreviewAtlas {
    pub texture: Handle<Image>,
    pub layout: Handle<TextureAtlasLayout>,
}

#[derive(Component)]
pub struct ResidentialBuilding {
    pub tile_pos: TilePos,
}

#[derive(Component)]
pub struct CommercialBuilding {
    pub tile_pos: TilePos,
}

#[derive(Component)]
pub struct IndustryBuilding {
    pub tile_pos: TilePos,
}

#[derive(Component)]
pub struct RoadSegment {
    pub tile_pos: TilePos,
}

#[derive(Component)]
pub struct DecorativeBuilding {
    pub tile_pos: TilePos,
}

/// Marker component for the semi-transparent road preview shown under the cursor
#[derive(Component)]
pub struct RoadHoverPreview;

pub fn setup_residential_building_atlas(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let texture: Handle<Image> = asset_server.load("sprites/houses.png");

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

pub fn setup_commercial_building_atlas(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let texture: Handle<Image> = asset_server.load("sprites/commercial.png");

    let layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(96, 96),
        1,
        COMMERCIAL_VARIANT_COUNT as u32,
        None,
        None,
    ));

    commands.insert_resource(CommercialBuildingAtlas {
        texture,
        layout,
        variants: COMMERCIAL_VARIANT_COUNT,
    });
}

pub fn setup_industry_building_atlas(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let texture: Handle<Image> = asset_server.load("sprites/factory.png");

    let layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(96, 96),
        1,
        INDUSTRY_VARIANT_COUNT as u32,
        None,
        None,
    ));

    commands.insert_resource(IndustryBuildingAtlas {
        texture,
        layout,
        variants: INDUSTRY_VARIANT_COUNT,
    });
}

pub fn setup_roads_atlas(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let texture: Handle<Image> = asset_server.load("sprites/roads.png");

    let layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(96, 96),
        1,
        ROAD_VARIANT_COUNT as u32,
        None,
        None,
    ));

    commands.insert_resource(RoadAtlas {
        texture,
        layout,
        variants: ROAD_VARIANT_COUNT,
    });
}

pub fn setup_decorative_building_atlas(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let texture: Handle<Image> = asset_server.load("sprites/decorative.png");

    let layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(96, 96),
        1,
        DECORATIVE_VARIANT_COUNT as u32,
        None,
        None,
    ));

    commands.insert_resource(DecorativeBuildingAtlas {
        texture,
        layout,
        variants: DECORATIVE_VARIANT_COUNT,
    });
}

pub fn setup_tile_preview_atlas(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let texture: Handle<Image> = asset_server.load("sprites/tiles.png");

    // `tiles.png` is arranged as a single column of 96x96 sprites (5 rows)
    let layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(96, 96),
        1,
        TILE_PREVIEW_VARIANT_COUNT as u32,
        None,
        None,
    ));

    commands.insert_resource(TilePreviewAtlas { texture, layout });
}
