use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct StateServiceConfig {
    #[allow(dead_code)]
    pub state_service: StateServiceSettings,
    #[allow(dead_code)]
    pub redis: RedisSettings,
    #[allow(dead_code)]
    pub postgres: PostgresSettings,
    #[allow(dead_code)]
    pub season: SeasonSettings,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StateServiceSettings {
    #[allow(dead_code)]
    pub snapshot_interval_ticks: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RedisSettings {
    #[allow(dead_code)]
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PostgresSettings {
    #[allow(dead_code)]
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SeasonSettings {
    #[allow(dead_code)]
    pub duration_days: u32,
    #[allow(dead_code)]
    pub transition_downtime_minutes: u32,
    #[allow(dead_code)]
    pub spawn_window_minutes: u32,
}