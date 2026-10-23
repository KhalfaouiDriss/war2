use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct ShardConfig {
    pub shard: ShardSettings,
    #[allow(dead_code)]
    pub region: RegionSettings,
    #[allow(dead_code)]
    pub redis: RedisSettings,
    #[allow(dead_code)]
    pub postgres: PostgresSettings,
    #[allow(dead_code)]
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
    #[allow(dead_code)]
    pub min_x: u32,
    #[allow(dead_code)]
    pub max_x: u32,
    #[allow(dead_code)]
    pub min_y: u32,
    #[allow(dead_code)]
    pub max_y: u32,
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
pub struct SimulationSettings {
    #[allow(dead_code)]
    pub tick_rate_hz: u32,
    #[allow(dead_code)]
    pub input_buffer_ticks: usize,
}