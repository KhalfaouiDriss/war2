//! Gateway configuration

use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize)]
pub struct GatewayConfig {
    pub gateway: GatewaySettings,
    #[allow(dead_code)]
    pub tls: TlsSettings,
    pub jwt: JwtSettings,
    #[allow(dead_code)]
    pub shards: HashMap<String, String>,
    #[allow(dead_code)]
    pub redis: RedisSettings,
    pub postgres: PostgresSettings,
    #[allow(dead_code)]
    pub rate_limits: RateLimitSettings,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GatewaySettings {
    pub host: String,
    pub port: u16,
    #[allow(dead_code)]
    pub workers: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TlsSettings {
    #[allow(dead_code)]
    pub cert_path: String,
    #[allow(dead_code)]
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
    #[allow(dead_code)]
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PostgresSettings {
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RateLimitSettings {
    #[allow(dead_code)]
    pub input_per_second: u32,
    #[allow(dead_code)]
    pub chat_per_second: u32,
    #[allow(dead_code)]
    pub market_per_second: u32,
    #[allow(dead_code)]
    pub auth_per_minute: u32,
}