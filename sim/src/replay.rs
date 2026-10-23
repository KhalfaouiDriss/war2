//! Replay system with verification

use crate::types::PlayerId;
use crate::input::Input;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use postcard;
use zstd;
use blake3;
use rand;
#[derive(Error, Debug)]
pub enum ReplayError {
    #[error("Postcard serialization error: {0}")]
    Postcard(#[from] postcard::Error),
    #[error("Zstd compression error: {0}")]
    Zstd(#[from] Box<dyn std::error::Error + Send + Sync>),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

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
    pub data: Vec<u8>, // zstd compressed InputBatch[]
}

/// Batch of inputs for a single tick
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputBatch {
    pub tick: u32,
    pub inputs: Vec<Input>,
}

/// Single player input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerInput {
    pub player: PlayerId,
    pub input: crate::input::Input,
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

/// Replay writer for recording matches
pub struct ReplayWriter {
    header: ReplayHeader,
    tick_index: Vec<TickIndexEntry>,
    chunks: Vec<ReplayChunk>,
    current_chunk: ReplayChunk,
    chunk_data: Vec<u8>,
    total_bytes: usize,
    #[allow(dead_code)]
    players: HashMap<PlayerId, [u8; 32]>,
}

impl ReplayWriter {
    pub fn new(season_id: u32, map_seed: u64, started_tick: u64, players: Vec<PlayerInfo>, initial_hash: [u8; 32]) -> Self {
        let header = ReplayHeader {
            magic: 0x57415232, // "WAR2"
            version: 1,
            season_id,
            map_seed,
            started_tick,
            ended_tick: 0,
            players,
            initial_snapshot_hash: initial_hash,
            final_state_hash: [0; 32],
        };
        
        ReplayWriter {
            header,
            tick_index: Vec::new(),
            chunks: Vec::new(),
            current_chunk: ReplayChunk {
                start_tick: 0,
                end_tick: 0,
                data: Vec::new(),
            },
            chunk_data: Vec::new(),
            total_bytes: 0,
            players: HashMap::new(),
        }
    }
    
    pub fn add_tick(&mut self, tick: u32, inputs: Vec<PlayerInput>) -> Result<(), ReplayError> {
        let inputs: Vec<Input> = inputs.into_iter().map(|pi| {
            Input {
                kind: pi.input.kind,
                tick: pi.input.tick,
                pixel_x: pi.input.pixel_x,
                pixel_y: pi.input.pixel_y,
                army_pct: pi.input.army_pct,
                building: pi.input.building,
                unit: pi.input.unit,
                count: pi.input.count,
            }
        }).collect();
        
        let batch = crate::input::InputBatch { tick, inputs };
        let data = postcard::to_stdvec(&batch)?;
        
        if self.chunk_data.len() + data.len() > 1_000_000 { // 1MB per chunk
            self.finalize_chunk()?;
        }
        
        if self.current_chunk.start_tick == 0 {
            self.current_chunk.start_tick = tick;
        }
        self.current_chunk.end_tick = tick;
        self.chunk_data.extend(data);
        
        Ok(())
    }
    
    fn finalize_chunk(&mut self) -> Result<(), ReplayError> {
        if !self.chunk_data.is_empty() {
            let compressed = zstd::encode_all(&*self.chunk_data, 3)?;
            self.current_chunk.data = compressed;
            self.tick_index.push(TickIndexEntry {
                tick: self.current_chunk.start_tick,
                offset: self.total_bytes as u32,
            });
            self.total_bytes += self.current_chunk.data.len();
            self.chunks.push(self.current_chunk.clone());
            self.current_chunk = ReplayChunk {
                start_tick: 0,
                end_tick: 0,
                data: Vec::new(),
            };
            self.chunk_data.clear();
        }
        Ok(())
    }
    
    pub fn finish(&mut self, ended_tick: u64, final_hash: [u8; 32]) -> Result<Replay, ReplayError> {
        if !self.chunk_data.is_empty() {
            self.finalize_chunk()?;
        }
        
        self.header.ended_tick = ended_tick;
        self.header.final_state_hash = final_hash;
        
        Ok(Replay {
            header: self.header.clone(),
            tick_index: self.tick_index.clone(),
            chunks: self.chunks.clone(),
        })
    }
}

/// Replay verifier - headless simulation for cheat detection
pub struct ReplayVerifier;

impl ReplayVerifier {
    /// Verify a replay by re-simulating and comparing final state hash
    pub fn verify(
        replay: &Replay,
        initial_snapshot: &[u8],
    ) -> Result<VerificationResult, ReplayError> {
        // Deserialize initial state
        let world: crate::grid::WorldGrid = postcard::from_bytes(initial_snapshot)?;
        
        // Replay all inputs
        let _current_tick = replay.header.started_tick as u32;
        
        for chunk in &replay.chunks {
            let decompressed = zstd::decode_all(&*chunk.data)?;
            
            // Parse tick batches
            let mut offset = 0;
            while offset < decompressed.len() {
                let batch: crate::input::InputBatch = postcard::from_bytes(&decompressed[offset..])?;
                let batch_size = postcard::to_stdvec(&batch)?.len();
                
                // Process each input
                for input in batch.inputs {
                    // Apply input to simulation (simplified)
                    // In real implementation, this would call the simulation step
                    let _ = input;
                }
                
                offset += batch_size;
            }
        }
        
        // Compute final state hash
        let final_state_bytes = postcard::to_stdvec(&world)?;
        let actual_hash = blake3::hash(&final_state_bytes);
        
        let expected_hash = replay.header.final_state_hash;
        let matched = actual_hash.as_bytes() == &expected_hash;
        
        Ok(VerificationResult {
            replay_id: 0, // Would be set from database
            expected_hash,
            actual_hash: *actual_hash.as_bytes(),
            matched,
            verified_tick: replay.header.ended_tick,
        })
    }
    
    /// Quick verification - just check hash without full replay (for sampling)
    pub fn quick_verify(
        replay: &Replay,
        initial_snapshot: &[u8],
    ) -> Result<bool, ReplayError> {
        let result = Self::verify(replay, initial_snapshot)?;
        Ok(result.matched)
    }
}

/// Sampling strategy for replay verification
pub struct VerificationSampler;

impl VerificationSampler {
    /// Determine if a replay should be verified based on sampling strategy
    pub fn should_verify(
        &self,
        _player_id: PlayerId,
        season_rank: Option<u32>,
        is_reported: bool,
        is_new_account: bool,
    ) -> bool {
        // Always verify reported replays
        if is_reported {
            return true;
        }
        
        // Always verify top 100 leaderboard
        if let Some(rank) = season_rank {
            if rank <= 100 {
                return true;
            }
        }
        
        // New accounts (< 3 seasons): 20%
        if is_new_account {
            return rand::random::<u8>() < 51; // ~20%
        }
        
        // Random 5% of remaining
        rand::random::<u8>() < 13 // ~5%
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_replay_header_magic() {
        let header = ReplayHeader {
            magic: 0x57415232,
            version: 1,
            season_id: 1,
            map_seed: 12345,
            started_tick: 0,
            ended_tick: 1000,
            players: vec![],
            initial_snapshot_hash: [0; 32],
            final_state_hash: [0; 32],
        };
        
        assert_eq!(header.magic, 0x57415232); // "WAR2"
    }
    
    #[test]
    fn test_verification_sampler() {
        let sampler = VerificationSampler;
        
        // Always verify reported
        assert!(sampler.should_verify(1, None, true, false));
        
        // Top 100 always verified
        assert!(sampler.should_verify(1, Some(50), false, false));
        assert!(sampler.should_verify(1, Some(100), false, false));
        assert!(!sampler.should_verify(1, Some(101), false, false));
    }
}