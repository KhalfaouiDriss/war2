//! Input types for lockstep simulation

use crate::grid::PixelCoord;
use crate::building::BuildingType;
use crate::tech::UnitType;
use serde::{Deserialize, Serialize};

/// Input kinds for lockstep
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InputKind {
    Capture,
    Build,
    Train,
    Upgrade,
}

/// Player input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Input {
    pub kind: InputKind,
    pub tick: u32,
    pub pixel_x: u32,
    pub pixel_y: u32,
    pub army_pct: u8,        // for Capture
    pub building: BuildingType, // for Build
    pub unit: UnitType,      // for Train
    pub count: u16,          // for Train
}

impl Input {
    pub fn capture(tick: u32, x: u32, y: u32, army_pct: u8) -> Self {
        Input {
            kind: InputKind::Capture,
            tick,
            pixel_x: x,
            pixel_y: y,
            army_pct,
            building: BuildingType::None,
            unit: UnitType::Infantry,
            count: 0,
        }
    }

    pub fn build(tick: u32, x: u32, y: u32, building: BuildingType) -> Self {
        Input {
            kind: InputKind::Build,
            tick,
            pixel_x: x,
            pixel_y: y,
            army_pct: 0,
            building,
            unit: UnitType::Infantry,
            count: 0,
        }
    }

    pub fn train(tick: u32, x: u32, y: u32, unit: UnitType, count: u16) -> Self {
        Input {
            kind: InputKind::Train,
            tick,
            pixel_x: x,
            pixel_y: y,
            army_pct: 0,
            building: BuildingType::None,
            unit,
            count,
        }
    }

    pub fn upgrade(tick: u32, x: u32, y: u32) -> Self {
        Input {
            kind: InputKind::Upgrade,
            tick,
            pixel_x: x,
            pixel_y: y,
            army_pct: 0,
            building: BuildingType::None,
            unit: UnitType::Infantry,
            count: 0,
        }
    }

    pub fn pixel(&self) -> PixelCoord {
        PixelCoord {
            x: self.pixel_x,
            y: self.pixel_y,
        }
    }
}

/// Batch of inputs for a single tick
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InputBatch {
    pub tick: u32,
    pub inputs: Vec<Input>,
}

/// Result of processing a tick
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TickResult {
    pub dirty_chunks: Vec<crate::grid::ChunkCoord>,
    pub events: Vec<GameEvent>,
    pub resource_deltas: crate::market::PlayerMarket,
}

/// Game events emitted during tick
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameEvent {
    Capture { pixel: PixelCoord, attacker: u32, defender: u32 },
    BuildComplete { pixel: PixelCoord, building: crate::building::BuildingType },
    UnitReady { pixel: PixelCoord, unit: crate::tech::UnitType, count: u16 },
    TradeExecuted { buyer: u32, seller: u32, resource: crate::building::ResourceType, quantity: u32, gold: u32 },
    Combat { pixel: PixelCoord, attacker: u32, defender: u32, damage: u16, captured: bool },
    SeasonEnding { seconds_remaining: u32 },
    SeasonTransition { new_season_id: u32, map_seed: u64 },
}