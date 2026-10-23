//! Tech tree system

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Tech branches
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TechBranch {
    Military,
    Economy,
    Defense,
    Utility,
}

/// Tech node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechNode {
    pub id: u32,
    pub branch: TechBranch,
    pub tier: u8, // 1-10
    pub name: String,
    pub description: String,
    pub cost: u64,
    pub prerequisites: Vec<u32>, // tech IDs
    pub unlocks: TechUnlocks,
}

/// What a tech unlocks
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TechUnlocks {
    pub buildings: Vec<super::building::BuildingType>,
    pub units: Vec<UnitType>,
    pub upgrades: Vec<UpgradeType>,
    pub abilities: Vec<AbilityType>,
}

/// Unit types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UnitType {
    Infantry,
    HeavyInfantry,
    LightVehicle,
    HeavyVehicle,
    Aircraft,
    SiegeUnit,
}

/// Upgrade types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UpgradeType {
    ArmorPlating,
    WeaponUpgrade,
    SpeedBoost,
    RangeIncrease,
}

/// Ability types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AbilityType {
    OrbitalStrike,
    ForcedTrade,
    SpySatellite,
    ShieldOvercharge,
}

/// Tech tree with all nodes
pub struct TechTree {
    nodes: HashMap<u32, TechNode>,
    by_branch_tier: HashMap<(TechBranch, u8), Vec<u32>>,
}

impl Default for TechTree {
    fn default() -> Self {
        Self::new()
    }
}

impl TechTree {
    pub fn new() -> Self {
        let mut tree = TechTree {
            nodes: HashMap::new(),
            by_branch_tier: HashMap::new(),
        };
        tree.populate();
        tree
    }

    fn populate(&mut self) {
        // Military branch
        self.add_node(TechNode {
            id: 101, branch: TechBranch::Military, tier: 1,
            name: "Basic Training".into(),
            description: "Unlocks Infantry".into(),
            cost: 100,
            prerequisites: vec![],
            unlocks: TechUnlocks { units: vec![UnitType::Infantry], ..Default::default() },
        });
        self.add_node(TechNode {
            id: 102, branch: TechBranch::Military, tier: 2,
            name: "Mechanization".into(),
            description: "Unlocks Light Vehicles, Barracks branch choice".into(),
            cost: 300,
            prerequisites: vec![101],
            unlocks: TechUnlocks {
                units: vec![UnitType::LightVehicle],
                upgrades: vec![UpgradeType::SpeedBoost],
                ..Default::default()
            },
        });
        self.add_node(TechNode {
            id: 103, branch: TechBranch::Military, tier: 3,
            name: "Air Superiority".into(),
            description: "Unlocks Aircraft (requires Radar T1)".into(),
            cost: 900,
            prerequisites: vec![102, 401], // Military T2 + Utility T1
            unlocks: TechUnlocks {
                units: vec![UnitType::Aircraft],
                buildings: vec![super::building::BuildingType::Airport],
                ..Default::default()
            },
        });
        // ... more tiers up to 10

        // Economy branch
        self.add_node(TechNode {
            id: 201, branch: TechBranch::Economy, tier: 1,
            name: "Resource Extraction".into(),
            description: "Unlocks all Extractors".into(),
            cost: 100,
            prerequisites: vec![],
            unlocks: TechUnlocks {
                buildings: vec![
                    super::building::BuildingType::WoodExtractor,
                    super::building::BuildingType::StoneExtractor,
                    super::building::BuildingType::IronExtractor,
                    super::building::BuildingType::FoodExtractor,
                ],
                ..Default::default()
            },
        });
        self.add_node(TechNode {
            id: 202, branch: TechBranch::Economy, tier: 2,
            name: "Industrial Processing".into(),
            description: "Unlocks Refineries".into(),
            cost: 300,
            prerequisites: vec![201],
            unlocks: TechUnlocks {
                buildings: vec![
                    super::building::BuildingType::LumberMill,
                    super::building::BuildingType::Stonecutter,
                    super::building::BuildingType::Foundry,
                    super::building::BuildingType::Farm,
                ],
                ..Default::default()
            },
        });
        // ... more tiers

        // Defense branch
        self.add_node(TechNode {
            id: 301, branch: TechBranch::Defense, tier: 1,
            name: "Fortification".into(),
            description: "Unlocks Walls and Towers".into(),
            cost: 100,
            prerequisites: vec![],
            unlocks: TechUnlocks {
                buildings: vec![
                    super::building::BuildingType::Wall,
                    super::building::BuildingType::Tower,
                ],
                ..Default::default()
            },
        });

        // Utility branch
        self.add_node(TechNode {
            id: 401, branch: TechBranch::Utility, tier: 1,
            name: "Surveillance".into(),
            description: "Unlocks Radar".into(),
            cost: 100,
            prerequisites: vec![],
            unlocks: TechUnlocks {
                buildings: vec![super::building::BuildingType::Radar],
                ..Default::default()
            },
        });
        self.add_node(TechNode {
            id: 402, branch: TechBranch::Utility, tier: 2,
            name: "Trade Networks".into(),
            description: "Unlocks Market".into(),
            cost: 300,
            prerequisites: vec![401],
            unlocks: TechUnlocks {
                buildings: vec![super::building::BuildingType::Market],
                abilities: vec![AbilityType::ForcedTrade],
                ..Default::default()
            },
        });
        self.add_node(TechNode {
            id: 403, branch: TechBranch::Utility, tier: 3,
            name: "Research Methods".into(),
            description: "Unlocks Laboratory".into(),
            cost: 900,
            prerequisites: vec![402],
            unlocks: TechUnlocks {
                buildings: vec![super::building::BuildingType::Laboratory],
                ..Default::default()
            },
        });
    }

    fn add_node(&mut self, node: TechNode) {
        self.by_branch_tier
            .entry((node.branch, node.tier))
            .or_default()
            .push(node.id);
        self.nodes.insert(node.id, node);
    }

    pub fn get(&self, id: u32) -> Option<&TechNode> {
        self.nodes.get(&id)
    }

    pub fn tier_nodes(&self, branch: TechBranch, tier: u8) -> Vec<&TechNode> {
        self.by_branch_tier
            .get(&(branch, tier))
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|id| self.nodes.get(id))
            .collect()
    }

    pub fn cost_for_tier(tier: u8) -> u64 {
        100 * 3_u64.pow(tier as u32 - 1)
    }
}

/// Player's tech state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerTechs {
    pub unlocked: HashSet<u32>,
    pub researching: Option<u32>,
    pub progress: u64,
}

impl Default for PlayerTechs {
    fn default() -> Self {
        Self::new()
    }
}

impl PlayerTechs {
    pub fn new() -> Self {
        PlayerTechs {
            unlocked: HashSet::new(),
            researching: None,
            progress: 0,
        }
    }

    pub fn can_research(&self, tree: &TechTree, tech_id: u32) -> bool {
        let Some(node) = tree.get(tech_id) else { return false; };
        if self.unlocked.contains(&tech_id) { return false; }
        if self.researching.is_some() { return false; }
        node.prerequisites.iter().all(|p| self.unlocked.contains(p))
    }

    pub fn start_research(&mut self, tech_id: u32) {
        self.researching = Some(tech_id);
        self.progress = 0;
    }

    pub fn add_progress(&mut self, amount: u64, tree: &TechTree) -> Option<u32> {
        let tech_id = self.researching?;
        let node = tree.get(tech_id)?;
        self.progress += amount;
        if self.progress >= node.cost {
            self.unlocked.insert(tech_id);
            self.researching = None;
            self.progress = 0;
            Some(tech_id)
        } else {
            None
        }
    }
}