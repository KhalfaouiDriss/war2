//! War2 Core Simulation Library
//!
//! This crate contains the deterministic simulation logic for War2,
//! including the pixel grid, capture mechanics, buildings, tech tree,
//! and market systems.

pub mod grid;
pub mod capture;
pub mod building;
pub mod tech;
pub mod market;
pub mod season;
pub mod replay;
pub mod input;
pub mod types;

pub use grid::{WorldGrid, ChunkCoord, PixelCoord, Tile, Chunk};
pub use capture::{CaptureSystem, CaptureResult};
pub use building::{BuildingSystem, BuildingType, Building};
pub use tech::{TechTree, TechBranch, TechNode};
pub use market::{MarketSystem, SellOffer};
pub use season::{SeasonSystem, SeasonData};
pub use replay::{Replay, ReplayHeader, ReplayChunk};
pub use input::{Input, InputKind, InputBatch};
pub use types::{PlayerId, SeasonId, ShardId, GameResult};
pub use building::ResourceType;

/// Simulation configuration constants
pub mod config {
    pub const TICK_RATE_HZ: u32 = 20;
    pub const TICK_MS: u64 = 50;
    pub const CHUNK_SIZE: u32 = 64;
    pub const CHUNK_TILES: usize = (CHUNK_SIZE * CHUNK_SIZE) as usize;
    pub const WORLD_SIZE: u32 = 50_000;
    pub const INPUT_BUFFER_TICKS: usize = 2;
    pub const CAPTURE_COOLDOWN_TICKS: u16 = 600; // 30s at 20Hz
    pub const MAX_FATIGUE: u8 = 100;
    pub const FATIGUE_ATTACK: u8 = 2;
    pub const FATIGUE_REPELLED: u8 = 5;
    pub const FATIGUE_RECOVERY: u8 = 1;
}