//! Building system

use crate::grid::{WorldGrid, PixelCoord, Tile};
use crate::config::CHUNK_SIZE;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Building types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u16)]
pub enum BuildingType {
    None = 0,
    CommandCenter = 1,
    WoodExtractor = 2,
    StoneExtractor = 3,
    IronExtractor = 4,
    FoodExtractor = 5,
    LumberMill = 6,
    Stonecutter = 7,
    Foundry = 8,
    Farm = 9,
    Barracks = 10,
    Factory = 11,
    Airport = 12,
    Wall = 13,
    Tower = 14,
    Market = 15,
    Laboratory = 16,
    Radar = 17,
}

impl BuildingType {
    pub fn footprint(&self) -> (u32, u32) {
        match self {
            BuildingType::CommandCenter => (3, 3),
            BuildingType::Airport => (3, 3),
            BuildingType::LumberMill
            | BuildingType::Stonecutter
            | BuildingType::Foundry
            | BuildingType::Farm
            | BuildingType::Barracks
            | BuildingType::Factory
            | BuildingType::Market
            | BuildingType::Laboratory
            | BuildingType::Radar => (2, 2),
            BuildingType::WoodExtractor
            | BuildingType::StoneExtractor
            | BuildingType::IronExtractor
            | BuildingType::FoodExtractor
            | BuildingType::Wall
            | BuildingType::Tower => (1, 1),
            BuildingType::None => (0, 0),
        }
    }

    pub fn is_extractor(&self) -> bool {
        matches!(
            self,
            BuildingType::WoodExtractor
                | BuildingType::StoneExtractor
                | BuildingType::IronExtractor
                | BuildingType::FoodExtractor
        )
    }

    pub fn is_refinery(&self) -> bool {
        matches!(
            self,
            BuildingType::LumberMill
                | BuildingType::Stonecutter
                | BuildingType::Foundry
                | BuildingType::Farm
        )
    }

    pub fn base_output(&self) -> Option<(ResourceType, u32)> {
        match self {
            BuildingType::WoodExtractor => Some((ResourceType::Wood, 10)),
            BuildingType::StoneExtractor => Some((ResourceType::Stone, 8)),
            BuildingType::IronExtractor => Some((ResourceType::Iron, 5)),
            BuildingType::FoodExtractor => Some((ResourceType::Food, 12)),
            BuildingType::LumberMill => Some((ResourceType::Lumber, 5)),
            BuildingType::Stonecutter => Some((ResourceType::Bricks, 4)),
            BuildingType::Foundry => Some((ResourceType::Steel, 3)),
            BuildingType::Farm => Some((ResourceType::Population, 2)),
            _ => None,
        }
    }
}

/// Building upgrade branches
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UpgradeBranch {
    None,
    // Extractor branches
    Deep,
    Wide,
    // Refinery branches
    Efficient,
    Fast,
    // Barracks branches
    Siege,
    Mobile,
    // Wall branches
    Stone,
    Reinforced,
    // Tower branches
    Cannon,
    Missile,
    // Lab branches
    Quantum,
    Network,
}

/// Building instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Building {
    pub building_type: BuildingType,
    pub level: u8, // 1, 2, 3
    pub branch: UpgradeBranch,
    pub workers_assigned: u16,
    pub max_workers: u16,
    pub construction_tick: u64,
    pub hp: u16,
    pub max_hp: u16,
}

impl Building {
    pub fn new(building_type: BuildingType, tick: u64) -> Self {
        let (max_workers, max_hp) = match building_type {
            BuildingType::CommandCenter => (0, 5000),
            BuildingType::WoodExtractor | BuildingType::StoneExtractor | BuildingType::IronExtractor | BuildingType::FoodExtractor => (10, 500),
            BuildingType::LumberMill | BuildingType::Stonecutter | BuildingType::Foundry | BuildingType::Farm => (20, 800),
            BuildingType::Barracks | BuildingType::Factory => (30, 1000),
            BuildingType::Airport => (20, 1200),
            BuildingType::Wall => (0, 2000),
            BuildingType::Tower => (0, 800),
            BuildingType::Market => (10, 600),
            BuildingType::Laboratory => (20, 600),
            BuildingType::Radar => (5, 400),
            _ => (0, 100),
        };

        Building {
            building_type,
            level: 1,
            branch: UpgradeBranch::None,
            workers_assigned: 0,
            max_workers,
            construction_tick: tick,
            hp: max_hp,
            max_hp,
        }
    }

    pub fn construction_time(&self) -> u64 {
        match self.level {
            1 => 0,
            2 => 1200, // 60s at 20Hz
            3 => 3600, // 180s at 20Hz
            _ => 0,
        }
    }

    pub fn output_multiplier(&self) -> f32 {
        match (self.level, self.branch) {
            (1, _) => 1.0,
            (2, UpgradeBranch::Deep) => 2.0,
            (2, UpgradeBranch::Wide) => 1.5,
            (2, UpgradeBranch::Efficient) => 1.2,
            (2, UpgradeBranch::Fast) => 2.0,
            (2, UpgradeBranch::Siege) => 1.0,
            (2, UpgradeBranch::Mobile) => 1.0,
            (2, UpgradeBranch::Stone) => 1.0,
            (2, UpgradeBranch::Reinforced) => 1.0,
            (2, UpgradeBranch::Cannon) => 1.0,
            (2, UpgradeBranch::Missile) => 1.0,
            (2, UpgradeBranch::Quantum) => 2.0,
            (2, UpgradeBranch::Network) => 1.0,
            (3, _) => 2.5,
            _ => 1.0,
        }
    }
}

/// Building system
pub struct BuildingSystem;

impl BuildingSystem {
    /// Check if building can be placed at position
    pub fn can_place(
        world: &mut WorldGrid,
        player: u32,
        pos: PixelCoord,
        building_type: BuildingType,
    ) -> bool {
        let (fw, fh) = building_type.footprint();
        if fw == 0 || fh == 0 {
            return false;
        }

        // Check all footprint pixels
        for dy in 0..fh {
            for dx in 0..fw {
                let px = pos.x + dx;
                let py = pos.y + dy;
                if px >= 50_000 || py >= 50_000 {
                    return false;
                }
                let tile = world.get_tile(PixelCoord { x: px, y: py });
                if tile.owner != player || tile.building != 0 {
                    return false;
                }
            }
        }
        true
    }

    /// Place building on grid
    pub fn place(
        world: &mut WorldGrid,
        pos: PixelCoord,
        building: Building,
    ) {
        let (fw, fh) = building.building_type.footprint();
        let building_id = building.building_type as u16;

        for dy in 0..fh {
            for dx in 0..fw {
                let px = pos.x + dx;
                let py = pos.y + dy;
                let mut tile = world.get_tile(PixelCoord { x: px, y: py });
                tile.building = building_id;
                world.set_tile(PixelCoord { x: px, y: py }, tile);
            }
        }
    }

    /// Remove building (on capture)
    pub fn remove(world: &mut WorldGrid, pos: PixelCoord) -> Vec<(PixelCoord, BuildingType)> {
        let mut removed = Vec::new();
        let center_tile = world.get_tile(pos);
        let building_type = BuildingType::from(center_tile.building);
        let (fw, fh) = building_type.footprint();

        // Find top-left of building
        let (mut min_x, mut min_y) = (pos.x, pos.y);
        for dy in 0..fh {
            for dx in 0..fw {
                let px = pos.x + dx;
                let py = pos.y + dy;
                let tile = world.get_tile(PixelCoord { x: px, y: py });
                if tile.building == building_type as u16 {
                    if px < min_x { min_x = px; }
                    if py < min_y { min_y = py; }
                }
            }
        }

        for dy in 0..fh {
            for dx in 0..fw {
                let px = min_x + dx;
                let py = min_y + dy;
                let mut tile = world.get_tile(PixelCoord { x: px, y: py });
                if tile.building == building_type as u16 {
                    removed.push((PixelCoord { x: px, y: py }, building_type));
                    tile.building = 0;
                    world.set_tile(PixelCoord { x: px, y: py }, tile);
                }
            }
        }
        removed
    }

    /// Calculate resource loot (50% of cost)
    pub fn loot(building: &Building) -> HashMap<ResourceType, u32> {
        let base_cost = match building.building_type {
            BuildingType::CommandCenter => vec![(ResourceType::Gold, 1000)],
            BuildingType::WoodExtractor => vec![(ResourceType::Wood, 200), (ResourceType::Gold, 100)],
            BuildingType::StoneExtractor => vec![(ResourceType::Stone, 200), (ResourceType::Gold, 100)],
            BuildingType::IronExtractor => vec![(ResourceType::Iron, 200), (ResourceType::Gold, 100)],
            BuildingType::FoodExtractor => vec![(ResourceType::Food, 200), (ResourceType::Gold, 100)],
            BuildingType::LumberMill => vec![(ResourceType::Wood, 500), (ResourceType::Gold, 300)],
            BuildingType::Stonecutter => vec![(ResourceType::Stone, 500), (ResourceType::Gold, 300)],
            BuildingType::Foundry => vec![(ResourceType::Iron, 500), (ResourceType::Gold, 300)],
            BuildingType::Farm => vec![(ResourceType::Food, 500), (ResourceType::Gold, 300)],
            BuildingType::Barracks => vec![(ResourceType::Gold, 800), (ResourceType::Lumber, 400)],
            BuildingType::Factory => vec![(ResourceType::Gold, 1200), (ResourceType::Steel, 400)],
            BuildingType::Airport => vec![(ResourceType::Gold, 2000), (ResourceType::Steel, 800)],
            BuildingType::Wall => vec![(ResourceType::Bricks, 300)],
            BuildingType::Tower => vec![(ResourceType::Gold, 500), (ResourceType::Steel, 200)],
            BuildingType::Market => vec![(ResourceType::Gold, 1000), (ResourceType::Lumber, 500)],
            BuildingType::Laboratory => vec![(ResourceType::Gold, 800), (ResourceType::Lumber, 300)],
            BuildingType::Radar => vec![(ResourceType::Gold, 600), (ResourceType::Steel, 200)],
            _ => vec![],
        };

        let mut loot = HashMap::new();
        for (res, cost) in base_cost {
            loot.insert(res, cost / 2 * building.level as u32);
        }
        loot
    }
}

impl From<u16> for BuildingType {
    fn from(v: u16) -> Self {
        match v {
            1 => BuildingType::CommandCenter,
            2 => BuildingType::WoodExtractor,
            3 => BuildingType::StoneExtractor,
            4 => BuildingType::IronExtractor,
            5 => BuildingType::FoodExtractor,
            6 => BuildingType::LumberMill,
            7 => BuildingType::Stonecutter,
            8 => BuildingType::Foundry,
            9 => BuildingType::Farm,
            10 => BuildingType::Barracks,
            11 => BuildingType::Factory,
            12 => BuildingType::Airport,
            13 => BuildingType::Wall,
            14 => BuildingType::Tower,
            15 => BuildingType::Market,
            16 => BuildingType::Laboratory,
            17 => BuildingType::Radar,
            _ => BuildingType::None,
        }
    }
}

/// Resource types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceType {
    Wood,
    Stone,
    Iron,
    Food,
    Lumber,
    Bricks,
    Steel,
    Population,
    Gold,
}