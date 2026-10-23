//! Replay system

use crate::input::Input;
use crate::types::PlayerId;
use serde::{Deserialize, Serialize};

/// Replay file format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Replay {
    pub header: ReplayHeader,
    pub tick_index: Vec<TickIndexEntry>,
    pub chunks: Vec<ReplayChunk>,
}

/// Replay header
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayHeader {
    pub magic: u32, // 0x57415232 ("WAR2")
    pub version: u16,
    pub season_id: u32,
    pub map_seed: u64,
    pub started_tick: u64,
    pub ended_tick: u64,
    pub players: Vec<PlayerInfo>,
    pub initial_snapshot_hash: [u8; 32], // Blake3
    pub final_state_hash: [u8; 32],      // Blake3
}

/// Player info in replay
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerInfo {
    pub id: PlayerId,
    pub username: String,
    pub initial_state_hash: [u8; 32],
}

/// Tick index entry for seeking
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TickIndexEntry {
    pub tick: u32,
    pub offset: u32,
}

/// Compressed tick batch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayChunk {
    pub start_tick: u32,
    pub end_tick: u32,
    pub data: Vec<u8>, // zstd compressed TickBatch[]
}

/// Batch of inputs for a single tick
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TickBatch {
    pub tick: u32,
    pub inputs: Vec<PlayerInput>,
}

/// Single player input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerInput {
    pub player: PlayerId,
    pub input: Input,
}

/// Replay storage tier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReplayTier {
    Hot,   // Redis, last 1000, 30 days
    Warm,  // Redis Stream, last 10k, 1 year
    Cold,  // PostgreSQL, forever
}

/// Verification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    pub replay_id: u64,
    pub expected_hash: [u8; 32],
    pub actual_hash: [u8; 32],
    pub matched: bool,
    pub verified_tick: u64,
}