# 09: Auth System

**What to build:** Username/password + bcrypt, JWT (1h access / 7d refresh), stateless gateway validation, RS256, season_id/shard_id in claims, no email/recovery.

**Blocked by:** 01-project-foundation

**Status:** ready-for-agent

- [ ] PostgreSQL schema: `users`, `seasons`, `player_seasons`, `guilds`, `guild_members`
- [ ] `POST /auth/register` → validate username, bcrypt hash (cost=12), create user, return JWT pair
- [ ] `POST /auth/login` → verify bcrypt, return JWT pair
- [ ] `POST /auth/refresh` → verify refresh token (rotate), return new pair
- [ ] Access token (1h): `sub`, `username`, `season_id`, `shard_id`, `guild_id`, `permissions`, `exp`
- [ ] Refresh token (7d): httpOnly cookie, `sub`, `type: "refresh"`, `exp`
- [ ] Gateway validation: RS256 signature verify, extract claims, route WS to shard
- [ ] No email verification, no password recovery
- [ ] Unit tests: registration, login, refresh, token validation, bcrypt