//! Gateway configuration

use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize)]
pub struct GatewayConfig {
    pub gateway: GatewaySettings,
    pub tls: TlsSettings,
    pub jwt: JwtSettings,
    pub shards: HashMap<String, String>,
    pub redis: RedisSettings,
    pub postgres: PostgresSettings,
    pub rate_limits: RateLimitSettings,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GatewaySettings {
    pub host: String,
    pub port: u16,
    pub workers: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TlsSettings {
    pub cert_path: String,
    pub key_path: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JwtSettings {
    pub secret: String,
    pub algorithm: String,
    pub access_token_ttl_hours: u64,
    pub refresh_token_ttl_days: u64,
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
pub struct RateLimitSettings {
    pub input_per_second: u32,
    pub chat_per_second: u32,
    pub market_per_second: u32,
    pub auth_per_minute: u32,
}