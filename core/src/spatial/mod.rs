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

