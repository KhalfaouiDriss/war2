//! Common type definitions

use serde::{Deserialize, Serialize};

/// Player identifier
pub type PlayerId = u32;

/// Season identifier
pub type SeasonId = u32;

/// Shard identifier
pub type ShardId = String;

/// Game result for API responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameResult<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> GameResult<T> {
    pub fn ok(data: T) -> Self {
        GameResult { success: true, data: Some(data), error: None }
    }

    pub fn err(error: String) -> Self {
        GameResult { success: false, data: None, error: Some(error) }
    }
}

/// Pagination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pagination {
    pub page: u32,
    pub per_page: u32,
    pub total: u64,
}

/// Health check response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    pub status: String,
    pub version: String,
    pub uptime_seconds: u64,
}

/// Configuration for services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    pub host: String,
    pub port: u16,
    pub workers: usize,
}

/// JWT claims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,        // user_id
    pub username: String,
    pub season_id: SeasonId,
    pub shard_id: ShardId,
    pub guild_id: Option<u32>,
    pub permissions: Vec<String>,
    pub exp: usize,
    pub iat: usize,
}

/// Region assignment for sharding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionAssignment {
    pub min_x: u32,
    pub max_x: u32,
    pub min_y: u32,
    pub max_y: u32,
    pub shard_id: ShardId,
}