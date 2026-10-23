//! Auth module - JWT handling, password hashing, auth endpoints

use anyhow::Result;
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

/// JWT claims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,        // user_id
    pub username: String,
    pub season_id: u32,
    pub shard_id: String,
    pub guild_id: Option<u32>,
    pub permissions: Vec<String>,
    pub exp: usize,
    pub iat: usize,
}

/// Token pair (access + refresh)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: u64,
}

/// Auth configuration
#[derive(Clone)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub algorithm: Algorithm,
    pub access_token_ttl_hours: u64,
    pub refresh_token_ttl_days: u64,
}

impl AuthConfig {
    pub fn from_settings(settings: &crate::settings::JwtSettings) -> Self {
        let algorithm = match settings.algorithm.as_str() {
            "RS256" => Algorithm::RS256,
            "HS256" => Algorithm::HS256,
            _ => Algorithm::HS256,
        };
        
        Self {
            jwt_secret: settings.secret.clone(),
            algorithm,
            access_token_ttl_hours: settings.access_token_ttl_hours,
            refresh_token_ttl_days: settings.refresh_token_ttl_days,
        }
    }
}

/// Password hashing
pub async fn hash_password(password: &str) -> Result<String> {
    let hashed = tokio::task::spawn_blocking({
        let password = password.to_string();
        move || hash(password, DEFAULT_COST)
    }).await??;
    Ok(hashed)
}

pub async fn verify_password(password: &str, hash: &str) -> Result<bool> {
    let password = password.to_string();
    let hash = hash.to_string();
    let valid = tokio::task::spawn_blocking(move || verify(password, &hash)).await??;
    Ok(valid)
}

/// JWT token generation
pub fn create_access_token(
    config: &AuthConfig,
    user_id: i64,
    username: &str,
    season_id: u32,
    shard_id: &str,
    guild_id: Option<u32>,
) -> Result<String> {
    let now = Utc::now();
    let exp = now + Duration::hours(config.access_token_ttl_hours as i64);
    
    let claims = Claims {
        sub: user_id.to_string(),
        username: username.to_string(),
        season_id,
        shard_id: shard_id.to_string(),
        guild_id,
        permissions: vec!["play".to_string(), "chat".to_string(), "trade".to_string()],
        exp: exp.timestamp() as usize,
        iat: now.timestamp() as usize,
    };
    
    let encoding_key = EncodingKey::from_secret(config.jwt_secret.as_bytes());
    let header = Header::new(config.algorithm);
    let token = encode(&header, &claims, &encoding_key)?;
    Ok(token)
}

pub fn create_refresh_token(config: &AuthConfig, user_id: i64) -> Result<String> {
    let now = Utc::now();
    let exp = now + Duration::days(config.refresh_token_ttl_days as i64);
    
    let claims = Claims {
        sub: user_id.to_string(),
        username: "".to_string(),
        season_id: 0,
        shard_id: "".to_string(),
        guild_id: None,
        permissions: vec!["refresh".to_string()],
        exp: exp.timestamp() as usize,
        iat: now.timestamp() as usize,
    };
    
    let encoding_key = EncodingKey::from_secret(config.jwt_secret.as_bytes());
    let header = Header::new(config.algorithm);
    let token = encode(&header, &claims, &encoding_key)?;
    Ok(token)
}

pub fn validate_token(config: &AuthConfig, token: &str) -> Result<Claims> {
    let decoding_key = DecodingKey::from_secret(config.jwt_secret.as_bytes());
    let mut validation = Validation::new(config.algorithm);
    validation.validate_exp = true;
    
    let token_data = decode::<Claims>(token, &decoding_key, &validation)?;
    Ok(token_data.claims)
}

/// Generate token pair for user
pub fn generate_token_pair(
    config: &AuthConfig,
    user_id: i64,
    username: &str,
    season_id: u32,
    shard_id: &str,
    guild_id: Option<u32>,
) -> Result<TokenPair> {
    let access_token = create_access_token(config, user_id, username, season_id, shard_id, guild_id)?;
    let refresh_token = create_refresh_token(config, user_id)?;
    
    Ok(TokenPair {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: config.access_token_ttl_hours * 3600,
    })
}

/// Database operations
pub async fn create_user(pool: &PgPool, username: &str, password_hash: &str) -> Result<i64> {
    let user_id: i64 = sqlx::query_scalar(
        "INSERT INTO users (username, password_hash) VALUES ($1, $2) RETURNING id",
    )
    .bind(username)
    .bind(password_hash)
    .fetch_one(pool)
    .await?;
    Ok(user_id)
}

pub async fn get_user_by_username(pool: &PgPool, username: &str) -> Result<Option<UserRow>> {
    let user = sqlx::query_as::<_, UserRow>(
        "SELECT id, username, password_hash, is_banned FROM users WHERE username = $1",
    )
    .bind(username)
    .fetch_optional(pool)
    .await?;
    Ok(user)
}

#[derive(Debug, sqlx::FromRow)]
pub struct UserRow {
    pub id: i64,
    pub username: String,
    pub password_hash: String,
    pub is_banned: bool,
}

pub async fn update_last_login(pool: &PgPool, user_id: i64) -> Result<()> {
    sqlx::query(
        "UPDATE users SET last_login = NOW() WHERE id = $1",
    )
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_player_season(pool: &PgPool, user_id: i64, season_id: u32) -> Result<Option<PlayerSeasonRow>> {
    let row = sqlx::query_as::<_, PlayerSeasonRow>(
        "SELECT user_id, season_id, gold, mmr, stats, techs, cosmetics FROM player_seasons WHERE user_id = $1 AND season_id = $2",
    )
    .bind(user_id)
    .bind(season_id as i32)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

#[derive(Debug, sqlx::FromRow)]
pub struct PlayerSeasonRow {
    pub user_id: i64,
    pub season_id: i32,
    pub gold: i64,
    pub mmr: i32,
    pub stats: serde_json::Value,
    pub techs: serde_json::Value,
    pub cosmetics: serde_json::Value,
}