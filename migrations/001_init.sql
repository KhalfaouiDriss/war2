-- War2 Database Schema
-- Migration 001: Initial schema

-- Users table
CREATE TABLE IF NOT EXISTS users (
    id BIGSERIAL PRIMARY KEY,
    username VARCHAR(20) UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    last_login TIMESTAMPTZ,
    is_banned BOOLEAN DEFAULT FALSE,
    ban_reason TEXT
);

CREATE INDEX IF NOT EXISTS idx_users_username ON users(username);

-- Seasons table
CREATE TABLE IF NOT EXISTS seasons (
    id INT PRIMARY KEY,
    started_at TIMESTAMPTZ NOT NULL,
    ended_at TIMESTAMPTZ,
    map_seed BIGINT NOT NULL
);

-- Player per-season data
CREATE TABLE IF NOT EXISTS player_seasons (
    user_id BIGINT REFERENCES users(id) ON DELETE CASCADE,
    season_id INT REFERENCES seasons(id) ON DELETE CASCADE,
    gold BIGINT DEFAULT 0,
    mmr INT DEFAULT 1000,
    stats JSONB DEFAULT '{}',
    techs JSONB DEFAULT '[]',
    cosmetics JSONB DEFAULT '[]',
    PRIMARY KEY (user_id, season_id)
);

CREATE INDEX IF NOT EXISTS idx_player_seasons_season ON player_seasons(season_id);

-- Guilds table
CREATE TABLE IF NOT EXISTS guilds (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(50) UNIQUE NOT NULL,
    tag VARCHAR(5) UNIQUE NOT NULL,
    owner_id BIGINT REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    bank_gold BIGINT DEFAULT 0
);

-- Guild members
CREATE TABLE IF NOT EXISTS guild_members (
    guild_id BIGINT REFERENCES guilds(id) ON DELETE CASCADE,
    user_id BIGINT REFERENCES users(id) ON DELETE CASCADE,
    role VARCHAR(20) DEFAULT 'member', -- owner, officer, member
    joined_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (guild_id, user_id)
);

-- Market offers
CREATE TABLE IF NOT EXISTS market_offers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    seller_id BIGINT REFERENCES users(id) ON DELETE CASCADE,
    resource VARCHAR(20) NOT NULL,
    quantity BIGINT NOT NULL,
    price_per_unit BIGINT NOT NULL,
    total_gold BIGINT NOT NULL,
    duration_hours SMALLINT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    market_building_level SMALLINT DEFAULT 0,
    remaining_quantity BIGINT NOT NULL,
    is_active BOOLEAN DEFAULT TRUE
);

CREATE INDEX IF NOT EXISTS idx_market_offers_active ON market_offers(is_active, expires_at);
CREATE INDEX IF NOT EXISTS idx_market_offers_seller ON market_offers(seller_id);

-- Trade history
CREATE TABLE IF NOT EXISTS trade_history (
    id BIGSERIAL PRIMARY KEY,
    buyer_id BIGINT REFERENCES users(id) ON DELETE SET NULL,
    seller_id BIGINT REFERENCES users(id) ON DELETE SET NULL,
    offer_id UUID,
    resource VARCHAR(20) NOT NULL,
    quantity BIGINT NOT NULL,
    gold_paid BIGINT NOT NULL,
    fee BIGINT NOT NULL,
    seller_received BIGINT NOT NULL,
    executed_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_trade_history_buyer ON trade_history(buyer_id);
CREATE INDEX IF NOT EXISTS idx_trade_history_seller ON trade_history(seller_id);

-- Replays
CREATE TABLE IF NOT EXISTS replays (
    id BIGSERIAL PRIMARY KEY,
    season_id INT REFERENCES seasons(id) ON DELETE CASCADE,
    map_seed BIGINT NOT NULL,
    started_tick BIGINT NOT NULL,
    ended_tick BIGINT NOT NULL,
    players JSONB NOT NULL,
    initial_snapshot_hash BYTEA NOT NULL,
    final_state_hash BYTEA NOT NULL,
    data BYTEA NOT NULL, -- compressed replay data
    verified BOOLEAN DEFAULT FALSE,
    verified_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_replays_season ON replays(season_id);
CREATE INDEX IF NOT EXISTS idx_replays_verified ON replays(verified);

-- Season leaderboards
CREATE TABLE IF NOT EXISTS season_leaderboards (
    season_id INT REFERENCES seasons(id) ON DELETE CASCADE,
    category VARCHAR(20) NOT NULL, -- territory, kills, economy, tech
    player_id BIGINT REFERENCES users(id) ON DELETE CASCADE,
    username VARCHAR(20) NOT NULL,
    score BIGINT NOT NULL,
    rank INT NOT NULL,
    PRIMARY KEY (season_id, category, player_id)
);

CREATE INDEX IF NOT EXISTS idx_leaderboards_season_category ON season_leaderboards(season_id, category);

-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";