# Wayfinder Map: Multiplayer Pixel War Game (RTS/MMO)

## Destination

A **full implementation spec** for a persistent-world RTS/MMO where players capture territory pixel-by-pixel on a massive grid, with deep production-chain economy, base building, army management, player-driven market, and tech tree. Seasons run 1-2 weeks with partial persistence (tech/cosmetics carry over).

## Notes

- **Core loop**: Gather → Refine → Build → Train → Capture → Expand → Repeat
- **Map**: Pixel grid (each pixel = capturable tile), pixel-art aesthetic
- **Combat**: Click pixel → assign army % → tug-of-war capture; deterministic lockstep + replay verification
- **Economy**: 4 primary → 4 refined → 4 sinks (Wood→Lumber→Buildings, Stone→Bricks→Walls, Iron→Steel→Units, Food→Population)
- **Market**: Direct P2P trade (click player → see sell offers, create own)
- **UI**: World map (main) + base builder + army manager + market + tech tree
- **Stack**: Rust WASM (sim) + TypeScript/React (UI) | Distributed: Gateway + World Shards + State Services | Redis + PostgreSQL | Email/JWT auth
- **Scale**: Start small, architecture supports horizontal scaling
- **Season**: 1-2 weeks, partial persistence

## Decisions so far

- **01-gateway-shard-protocol**: Custom binary TCP (postcard) + dynamic quadtree sharding + event sourcing via Redis Streams → PG snapshots. Cross-shard via multi-shard client connections + dedicated global service shards. Gateway-managed failover.
- **02-pixel-grid-data-structure**: Chunked 64×64 tiles (sparse HashMap), 16-byte Tile struct (owner, army%, building, 4 resources). postcard serialization. Prototype validated: 100k neighbors in 6ms, 20 ticks ~41ms (needs optimization for 20Hz target).
- **03-lockstep-tick-rate**: 20 Hz (50ms), 2-tick buffer, minimal inputs (Capture/Build/Train/Upgrade), gateway auth + shard rules, snapshot+inputs late join, inputs-only replay.
- **04-army-capture-mechanics**: Ratio-based progress (attacker/defender ×10), fatigue (+2/tick attack, -1/tick idle), hard supply lines (BFS to CC, adjacent capture allowed), 30s cooldown, strongest-attacker-wins, pre-set defense %.
- **05-building-placement-upgrades**: 17 building types, varying footprints, placement on owned pixels only, worker allocation production, branch-choice upgrades (L2 split), destroyed on capture (50% loot).
- **06-tech-tree**: 4 parallel branches × 10 tiers, Lab-generated science, exponential costs, hard prerequisites, all tech persists across seasons, unlocks buildings/units/upgrades/abilities.
- **07-market-trade-protocol**: Gold-only currency, direct click + Market building discovery, instant atomic swap, distance-based fees (0–8%), configurable duration 1h–7d, Market L1=local L2=global L3=zero-fee.
- **08-season-transition**: 14-day fixed seasons, ~5min hot-swap downtime, fully random map each season, persists techs/cosmetics/gold/guild, resets territory/buildings/armies/resources/MMR, per-season leaderboards, free-placement spawn.
- **09-auth-flow**: Username/password + bcrypt, JWT (1h access / 7d refresh), stateless gateway validation, RS256, season_id/shard_id in claims, no email/recovery, PG schema for users/seasons/guilds.
- **10-replay-verification**: Binary postcard format, header+tick-index+chunks, top-players+reported sampling, headless verifier (same binary), Blake3 state hash mismatch detection, Redis+PG storage, ban+rewards revoked punishment.
- **11-client-server-protocol**: JSON over WebSocket, 7 categories (Auth/Input/State/Event/Chat/Market/System), full snapshot on join + delta per tick, interest management = owned + border, lockstep inputs batched, reconnection via snapshot+delta replay.
- **12-wasm-ts-boundary**: wasm-bindgen+tsify codegen, WASM linear memory + JS views, WASM computes/JS renders, JS rAF+accumulator tick loop, wasm-pack+Vite build, shared types auto-generated.
- **13-local-dev-environment**: 6 services (gateway, 2 shards, redis, postgres, client-dev), docker-compose + Makefile, cargo watch + Vite HMR, SQL migrations on startup, 2 shards for partition testing.

## Not yet specified

<!-- All tickets resolved -->

## Out of scope

- Mobile app (web-first)
- Blockchain/Web3 integration
- AI opponents (PvP only)
- Custom map editor (procedural only)

## Out of scope

- Mobile app (web-first)
- Blockchain/Web3 integration
- AI opponents (PvP only)
- Custom map editor (procedural only)