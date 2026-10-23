use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct StateServiceConfig {
    pub state_service: StateServiceSettings,
    pub redis: RedisSettings,
    pub postgres: PostgresSettings,
    pub season: SeasonSettings,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StateServiceSettings {
    pub snapshot_interval_ticks: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RedisSettings {
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PostgresSettings {
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SeasonSettings {
    pub duration_days: u32,
    pub transition_downtime_minutes: u32,
    pub spawn_window_minutes: u32,
}