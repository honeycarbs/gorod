use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::TilePos;
use std::collections::{HashMap, HashSet};

use crate::budget::{BuildingDemolished, BuildingPlaced, BuildingType};

const CELL_SIZE: i32 = 8;

fn cell_key(pos: &TilePos) -> (i32, i32) {
    (pos.x as i32 / CELL_SIZE, pos.y as i32 / CELL_SIZE)
}

fn cell_keys_in_radius(center: &TilePos, radius: i32) -> impl Iterator<Item = (i32, i32)> {
    let (cx, cy) = cell_key(center);
    let cell_radius = (radius + CELL_SIZE - 1) / CELL_SIZE;

    let keys: Vec<(i32, i32)> = (-cell_radius..=cell_radius)
        .flat_map(move |dx| (-cell_radius..=cell_radius).map(move |dy| (cx + dx, cy + dy)))
        .collect();
    keys.into_iter()
}

#[derive(Default)]
struct TypedSpatialGrid {
    cells: HashMap<(i32, i32), HashSet<TilePos>>,
}

impl TypedSpatialGrid {
    fn insert(&mut self, pos: TilePos) {
        let key = cell_key(&pos);
        self.cells.entry(key).or_default().insert(pos);
    }

    fn remove(&mut self, pos: &TilePos) {
        let key = cell_key(pos);
        if let Some(set) = self.cells.get_mut(&key) {
            set.remove(pos);
            if set.is_empty() {
                self.cells.remove(&key);
            }
        }
    }

    fn query_chebyshev(&self, center: &TilePos, radius: i32) -> impl Iterator<Item = &TilePos> {
        let center_x = center.x as i32;
        let center_y = center.y as i32;

        cell_keys_in_radius(center, radius)
            .filter_map(|key| self.cells.get(&key))
            .flatten()
            .filter(move |pos| {
                let dx = (pos.x as i32 - center_x).abs();
                let dy = (pos.y as i32 - center_y).abs();
                dx <= radius && dy <= radius
            })
    }
}

#[derive(Resource, Default)]
pub struct SpatialGrid {
    residential: TypedSpatialGrid,
    commercial: TypedSpatialGrid,
    industry: TypedSpatialGrid,
    roads: TypedSpatialGrid,
    all_buildings: TypedSpatialGrid,
}

impl SpatialGrid {
    pub fn insert(&mut self, pos: TilePos, building_type: BuildingType) {
        self.all_buildings.insert(pos);
        match building_type {
            BuildingType::Residential => self.residential.insert(pos),
            BuildingType::Commercial => self.commercial.insert(pos),
            BuildingType::Industry => self.industry.insert(pos),
            BuildingType::Road => self.roads.insert(pos),
            BuildingType::Decorative => {}
        }
    }

    pub fn remove(&mut self, pos: &TilePos, building_type: BuildingType) {
        self.all_buildings.remove(pos);
        match building_type {
            BuildingType::Residential => self.residential.remove(pos),
            BuildingType::Commercial => self.commercial.remove(pos),
            BuildingType::Industry => self.industry.remove(pos),
            BuildingType::Road => self.roads.remove(pos),
            BuildingType::Decorative => {}
        }
    }

    pub fn count_residential_in_radius(&self, center: &TilePos, radius: i32) -> u32 {
        self.residential
            .query_chebyshev(center, radius)
            .filter(|pos| **pos != *center)
            .count() as u32
    }

    pub fn has_road_in_radius(&self, center: &TilePos, radius: i32) -> bool {
        self.roads.query_chebyshev(center, radius).next().is_some()
    }

    pub fn has_building_in_radius(&self, center: &TilePos, radius: i32) -> bool {
        self.all_buildings
            .query_chebyshev(center, radius)
            .any(|pos| *pos != *center)
    }

    pub fn buildings_in_radius(&self, center: &TilePos, radius: i32) -> Vec<TilePos> {
        self.all_buildings
            .query_chebyshev(center, radius)
            .copied()
            .collect()
    }
}

pub fn sync_spatial_grid_on_placement(
    mut spatial_grid: ResMut<SpatialGrid>,
    mut placed_reader: MessageReader<BuildingPlaced>,
) {
    for event in placed_reader.read() {
        spatial_grid.insert(event.tile_pos, event.building_type);
    }
}

pub fn sync_spatial_grid_on_demolition(
    mut spatial_grid: ResMut<SpatialGrid>,
    mut demolished_reader: MessageReader<BuildingDemolished>,
) {
    for event in demolished_reader.read() {
        spatial_grid.remove(&event.tile_pos, event.building_type);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tile(x: u32, y: u32) -> TilePos {
        TilePos { x, y }
    }

    #[test]
    fn cell_key_maps_positions_to_grid_cells() {
        assert_eq!(cell_key(&tile(0, 0)), (0, 0));
        assert_eq!(cell_key(&tile(7, 7)), (0, 0)); // Still in first cell
        assert_eq!(cell_key(&tile(8, 8)), (1, 1)); // Next cell
        assert_eq!(
            cell_key(&tile(100, 200)),
            (100 / CELL_SIZE, 200 / CELL_SIZE)
        );
    }

    #[test]
    fn typed_grid_insert_query_remove() {
        let mut grid = TypedSpatialGrid::default();

        grid.insert(tile(10, 10));
        grid.insert(tile(11, 10));
        grid.insert(tile(20, 20)); // Far away

        // Query within radius 2 should find 2 positions
        let results: Vec<_> = grid.query_chebyshev(&tile(10, 10), 2).collect();
        assert_eq!(results.len(), 2);

        // After removal, only 1 remains
        grid.remove(&tile(10, 10));
        let results: Vec<_> = grid.query_chebyshev(&tile(10, 10), 2).collect();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn spatial_grid_tracks_building_types_separately() {
        let mut grid = SpatialGrid::default();

        grid.insert(tile(5, 5), BuildingType::Residential);
        grid.insert(tile(6, 6), BuildingType::Road);
        grid.insert(tile(7, 7), BuildingType::Commercial);

        assert_eq!(grid.count_residential_in_radius(&tile(10, 10), 10), 1);
        assert!(grid.has_road_in_radius(&tile(10, 10), 10));
        assert_eq!(grid.buildings_in_radius(&tile(6, 6), 5).len(), 3);
    }

    #[test]
    fn queries_exclude_center_position() {
        let mut grid = SpatialGrid::default();
        let center = tile(10, 10);

        grid.insert(center, BuildingType::Residential);
        grid.insert(tile(11, 10), BuildingType::Residential);

        // count_residential excludes center
        assert_eq!(grid.count_residential_in_radius(&center, 5), 1);
        // has_building excludes center
        assert!(!grid.has_building_in_radius(&center, 0));
    }

    #[test]
    fn remove_clears_from_all_relevant_grids() {
        let mut grid = SpatialGrid::default();
        let pos = tile(5, 5);

        grid.insert(pos, BuildingType::Residential);
        grid.remove(&pos, BuildingType::Residential);

        assert_eq!(grid.count_residential_in_radius(&tile(6, 6), 5), 0);
        assert!(!grid.has_building_in_radius(&tile(6, 6), 5));
    }
}
