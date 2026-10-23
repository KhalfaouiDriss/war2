use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct ShardConfig {
    pub shard: ShardSettings,
    pub region: RegionSettings,
    pub redis: RedisSettings,
    pub postgres: PostgresSettings,
    pub simulation: SimulationSettings,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ShardSettings {
    pub id: String,
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RegionSettings {
    pub min_x: u32,
    pub max_x: u32,
    pub min_y: u32,
    pub max_y: u32,
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
pub struct SimulationSettings {
    pub tick_rate_hz: u32,
    pub input_buffer_ticks: usize,
}