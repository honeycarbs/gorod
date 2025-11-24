use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
mod map;
mod camera;
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(TilemapPlugin)
        .add_plugins(map::TilePlacementPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, (map::change_tile_type))
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Spawn a controllable camera
    // Camera2d automatically includes required components like OrthographicProjection
    let camera_entity = commands.spawn((
        Camera2d,
        camera::CameraController {
            move_speed: 500.0,
            zoom_speed: 1.5,
            min_zoom: 0.1,
            max_zoom: 3.0,
        },
    )).id();

    let texture_handle: Handle<Image> = asset_server.load("tiles.png");
    let map_size = TilemapSize { x: 32, y: 32 };
    let tile_size = TilemapTileSize { x: 32.0, y: 32.0 }; // Larger tiles
    let grid_size = tile_size.into();

    let tilemap_entity = commands.spawn_empty().id();
    let mut tile_storage = TileStorage::empty(map_size);

    // Pre-fill with tiles - starting with texture index 0
    for x in 0..map_size.x {
        for y in 0..map_size.y {
            let tile_pos = TilePos { x, y };
            let tile_entity = commands
                .spawn(TileBundle {
                    position: tile_pos,
                    tilemap_id: TilemapId(tilemap_entity),
                    texture_index: TileTextureIndex(0),
                    ..Default::default()
                })
                .id();
            tile_storage.set(&tile_pos, tile_entity);
        }
    }

    commands.entity(tilemap_entity).insert(TilemapBundle {
        grid_size,
        size: map_size,
        storage: tile_storage,
        texture: TilemapTexture::Single(texture_handle),
        tile_size,
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..Default::default()
    });
}