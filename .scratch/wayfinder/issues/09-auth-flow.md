# Ticket: Auth Flow & Session Management — RESOLVED

## Decision

**Auth Method**: **Username/Password + JWT** (email optional/username only)

**Registration Flow**:
```
1. Client → Gateway: POST /auth/register { username, password }
2. Gateway: validate username (unique, 3-20 chars, alphanumeric)
3. Gateway: hash password with bcrypt (cost=12, min 8 chars)
4. Gateway: create user in PG (users table)
5. Gateway: generate JWT pair → return to client
6. Client: store access_token (memory) + refresh_token (secure httpOnly cookie)
```

**Login Flow**:
```
1. Client → Gateway: POST /auth/login { username, password }
2. Gateway: lookup user in PG
3. Gateway: verify bcrypt hash
4. Gateway: generate JWT pair → return
5. Client: store tokens
```

**JWT Access Token** (1 hour, stateless):
```json
{
  "sub": "user_12345",
  "username": "player1",
  "season_id": 42,
  "shard_id": "shard-7",
  "guild_id": "guild-99",
  "permissions": ["play", "chat", "trade"],
  "iat": 1699999999,
  "exp": 1700003599
}
```

**JWT Refresh Token** (7 days, httpOnly cookie):
```json
{
  "sub": "user_12345",
  "type": "refresh",
  "iat": 1699999999,
  "exp": 1700604799
}
```

**Token Refresh**:
```
1. Client → Gateway: POST /auth/refresh (cookie sent automatically)
2. Gateway: verify refresh token signature + not expired
3. Gateway: generate new access + refresh pair (rotate refresh)
4. Gateway: set new refresh cookie, return access token
5. Client: use new access token
```

**Gateway Validation (Stateless)**:
- Verify JWT signature (RS256, public key from config)
- Check `exp` not expired
- Extract `season_id`, `shard_id` → route WebSocket to shard
- No Redis lookup (stateless, scalable)
- Revocation: short access token (1h) = natural expiry; emergency = key rotation

**Password Policy**:
- Bcrypt cost=12
- Min 8 chars, max 72 (bcrypt limit)
- No complexity rules (length + bcrypt sufficient)
- Rate limit: 5 attempts/15min per IP

**Account Recovery**: **None** (username only, no email)
- "Forgot password" = not supported
- User must remember credentials
- Simplifies: no email infra, no verification flow, no PII

**Logout**:
- Client deletes access token from memory
- Optional: call POST /auth/logout → adds refresh token to Redis blacklist (TTL = remaining expiry)
- Blacklist checked only on refresh (not access)

**Gateway → Shard Auth**:
- WebSocket handshake: client sends `Authorization: Bearer <access_token>`
- Gateway validates JWT, extracts `user_id`, `season_id`, `shard_id`
- Gateway forwards to shard: `Authenticated { user_id, season_id, permissions }`
- Shard trusts gateway (internal network)

**Season Transition** (Ticket 08):
- New season → new `season_id` in JWT
- Gateway issues new tokens on next refresh
- Old tokens valid until expiry (max 1h)

**Database Schema** (PG):
```sql
CREATE TABLE users (
    id BIGSERIAL PRIMARY KEY,
    username VARCHAR(20) UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,  -- bcrypt
    created_at TIMESTAMPTZ DEFAULT NOW(),
    last_login TIMESTAMPTZ,
    is_banned BOOLEAN DEFAULT FALSE,
    ban_reason TEXT
);

CREATE TABLE seasons (
    id INT PRIMARY KEY,           -- 1, 2, 3...
    started_at TIMESTAMPTZ,
    ended_at TIMESTAMPTZ,
    map_seed BIGINT
);

CREATE TABLE player_seasons (
    user_id BIGINT REFERENCES users(id),
    season_id INT REFERENCES seasons(id),
    gold BIGINT DEFAULT 0,        -- persisted across seasons
    mmr INT DEFAULT 1000,         -- reset each season
    stats JSONB,                  -- kills, territory, economy...
    PRIMARY KEY (user_id, season_id)
);

CREATE TABLE guilds (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(50) UNIQUE NOT NULL,
    tag VARCHAR(5) UNIQUE NOT NULL,
    owner_id BIGINT REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    bank_gold BIGINT DEFAULT 0
);

CREATE TABLE guild_members (
    guild_id BIGINT REFERENCES guilds(id),
    user_id BIGINT REFERENCES users(id),
    role VARCHAR(20) DEFAULT 'member',  -- owner, officer, member
    joined_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (guild_id, user_id)
);
```

## Blocking → Unblocked

- ✅ Unblocks: **10-replay-verification** (user_id in replay)
- ✅ Unblocks: **11-client-server-protocol** (auth messages)
- ✅ Unblocks: **13-local-dev-environment** (auth service in docker-compose)