//! Season system

use crate::tech::PlayerTechs;
use crate::building::BuildingType;
use crate::grid::PixelCoord;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Season data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeasonData {
    pub id: u32,
    pub started_tick: u64,
    pub ended_tick: Option<u64>,
    pub map_seed: u64,
    pub leaderboard: SeasonLeaderboard,
}

/// Season leaderboard
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SeasonLeaderboard {
    pub territory: Vec<LeaderboardEntry>,
    pub kills: Vec<LeaderboardEntry>,
    pub economy: Vec<LeaderboardEntry>,
    pub tech: Vec<LeaderboardEntry>,
}

/// Leaderboard entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderboardEntry {
    pub player_id: u32,
    pub username: String,
    pub score: u64,
    pub rank: u32,
}

/// Persisted player data across seasons
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerPersistence {
    pub techs: PlayerTechs,
    pub cosmetics: Vec<Cosmetic>,
    pub gold: u64,
    pub guild_id: Option<u32>,
    pub season_stats: HashMap<u32, SeasonStats>, // season_id -> stats
}

/// Season stats (archived per season)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SeasonStats {
    pub territory_peak: u32,
    pub kills: u32,
    pub economy_score: u64,
    pub techs_unlocked: u32,
    pub final_rank: Option<u32>,
}

/// Cosmetic items
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cosmetic {
    pub id: u32,
    pub kind: CosmeticKind,
    pub unlocked_tick: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CosmeticKind {
    Skin,
    Title,
    Emote,
    Badge,
}

/// Season system
pub struct SeasonSystem;

impl SeasonSystem {
    pub const SEASON_DURATION_TICKS: u64 = 14 * 24 * 60 * 60 * 20; // 14 days at 20Hz
    pub const SPAWN_WINDOW_TICKS: u64 = 30 * 60 * 20; // 30 minutes
    pub const TRANSITION_DOWNTIME_TICKS: u64 = 5 * 60 * 20; // 5 minutes

    /// Check if season should end
    pub fn should_end(current_tick: u64, season_start: u64) -> bool {
        current_tick >= season_start + Self::SEASON_DURATION_TICKS
    }

    /// Generate migration plan
    pub fn migration_plan(
        all_players: &HashMap<u32, PlayerPersistence>,
    ) -> MigrationPlan {
        MigrationPlan {
            players: all_players.iter().map(|(id, p)| {
                (
                    *id,
                    PlayerMigration {
                        techs: p.techs.clone(),
                        cosmetics: p.cosmetics.clone(),
                        gold: p.gold,
                        guild_id: p.guild_id,
                        mmr: 1000, // reset
                    },
                )
            }).collect(),
        }
    }
}

/// Migration plan for season transition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationPlan {
    pub players: HashMap<u32, PlayerMigration>,
}

/// Per-player migration data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerMigration {
    pub techs: PlayerTechs,
    pub cosmetics: Vec<Cosmetic>,
    pub gold: u64,
    pub guild_id: Option<u32>,
    pub mmr: u32, // reset to 1000
}