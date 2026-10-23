# SPEC: War2 — Persistent-World RTS/MMO

## Problem Statement

Players want a browser-based, persistent-world RTS game in the style of frontwar.io where they capture territory pixel-by-pixel on a massive shared map, build bases, manage a deep production-chain economy, research technologies, trade with other players via a P2P market, and compete in 14-day seasons with partial persistence. The game must support thousands of concurrent players, run deterministically for cheat resistance, and be playable in a browser with pixel-art aesthetics.

## Solution

A distributed, deterministic lockstep multiplayer game with:
- **Rust WebAssembly simulation** (client + server) for performance and determinism
- **TypeScript/React client** for UI, rendering, and input
- **Gateway + World Shards + State Services** backend for horizontal scaling
- **Event sourcing via Redis Streams** with PostgreSQL snapshots for persistence
- **Dynamic quadtree sharding** of the 50,000×50,000 pixel grid
- **20 Hz deterministic lockstep** with 2-tick input buffer
- **Ratio-based capture mechanics** with fatigue, supply lines, and cooldowns
- **17 building types** with worker allocation and branch-choice upgrades
- **4-branch × 10-tier tech tree** with Lab-generated science
- **Gold-denominated P2P market** with distance-based fees
- **14-day seasons** with hot-swap transition, persisting techs/cosmetics/gold/guild

## User Stories

### Core Gameplay
1. As a player, I want to click an adjacent enemy pixel and assign a percentage of my army to capture it, so that I can expand my territory.
2. As a player, I want capture progress to be determined by the ratio of attacker vs defender effective army (modified by fatigue), so that combat feels fair and strategic.
3. As a player, I want my army to accumulate fatigue during combat and recover when idle, so that sustained pushes have diminishing returns.
4. As a player, I want captured pixels to have a 30-second cooldown before they can be recaptured, so that frontlines stabilize.
5. As a player, I want supply lines to require a continuous path of owned pixels back to my Command Center, so that deep strikes are risky.
6. As a player, I want to capture pixels adjacent to my supply-connected territory, so that I can expand naturally.

### Economy & Buildings
7. As a player, I want to place buildings on my owned pixels (replacing terrain), so that I can develop my territory.
8. As a player, I want to assign workers (population from Food) to buildings to produce resources, so that I control my economy.
9. As a player, I want extractors to produce primary resources (Wood, Stone, Iron, Food) and refineries to convert them (Lumber, Bricks, Steel, Population), so that there are meaningful production chains.
10. As a player, I want buildings to have upgrade branches at Level 2 (e.g., Barracks → Siege or Mobile), so that I can specialize my base.
11. As a player, I want captured buildings to be destroyed (50% resource loot to capturer), so that territory loss is meaningful.

### Technology
12. As a player, I want to build Laboratories that generate Science based on assigned workers, so that I can research technologies.
13. As a player, I want a tech tree with 4 parallel branches (Military, Economy, Defense, Utility) × 10 tiers, so that I have long-term progression goals.
14. As a player, I want tech costs to scale exponentially (100 → 300 → 900...), so that higher tiers require significant investment.
15. As a player, I want all unlocked techs to persist across seasons, so that my knowledge is never lost.

### Market & Trade
16. As a player, I want to click another player's base to see their sell offers, so that I can trade directly.
17. As a player, I want to build a Market building to see global offers (L1: local radius, L2: global, L3: zero fees), so that I can access wider trade.
18. As a player, I want all trades to use Gold as currency with instant atomic swap, so that trading is fast and unambiguous.
19. As a player, I want distance-based fees (0% local, up to 8% cross-map), so that local trade networks are encouraged.
20. As a player, I want to set offer duration (1h–7d) and cancel anytime, so that I control my listings.

### Multiplayer & Seasons
21. As a player, I want to register with username/password and receive a JWT for authentication, so that my account is secure.
22. As a player, I want the game to run at 20 Hz with deterministic lockstep, so that gameplay is fair and cheat-resistant.
23. As a player, I want to play in 14-day seasons with a fresh random map each season, so that the world resets but my progress remains.
24. As a player, I want my techs, cosmetics, gold, and guild membership to persist across seasons, so that I keep meaningful progress.
25. As a player, I want per-season leaderboards (territory, kills, economy, tech) archived historically, so that I can compare past performance.
26. As a player, I want to place my initial Command Center freely during the first 30 minutes of a season, so that I can claim prime territory.

### Technical
27. As a developer, I want the simulation to be deterministic (same inputs → same state), so that replays can verify gameplay integrity.
28. As a developer, I want headless verification of replays using the same binary, so that cheaters are detected via state hash mismatch.
29. As a developer, I want the client to render via Canvas/WebGL while WASM computes simulation, so that performance is optimal.
30. As a developer, I want a local dev environment with Docker Compose + Makefile, so that I can develop with hot reload.

## Implementation Decisions

### Architecture
- **Gateway** (Rust, axum + tokio-tungstenite): TLS termination, JWT validation, routes WS to shards via TCP+postcard
- **World Shards** (Rust, tokio): Own dynamic quadtree regions of pixel grid, run deterministic simulation, append inputs to Redis Streams
- **State Services** (Rust): Consume Redis Streams → write PostgreSQL snapshots; run global services (market, chat, leaderboards)
- **Client** (TypeScript/React + Vite): WASM simulation via wasm-bindgen, Canvas/WebGL rendering, rAF + accumulator tick loop

### Data Structures
- **Pixel Grid**: Chunked 64×64 tiles, sparse `HashMap<ChunkCoord, Chunk>`, 16-byte `Tile { owner: u32, army_pct: u8, building: u16, resources: [u16;4] }`
- **Chunk**: 4096 tiles, dirty flag, version; only dirty chunks serialized per tick
- **Input**: `Capture { pixel, army_pct }`, `Build { pixel, building }`, `Train { pixel, unit, count }`, `Upgrade { pixel }`
- **Replay**: Binary postcard, header + tick-index + zstd-compressed chunks, Blake3 state hashes

### Protocols
- **Gateway↔Shard**: Custom binary TCP, postcard serialization, persistent connections
- **Client↔Gateway**: JSON over WebSocket (wss://), 7 message categories (Auth, Input, State, Event, Chat, Market, System)
- **State Sync**: Full snapshot on join (visible chunks: owned + 1-border), delta per tick (20 Hz)
- **Shard Boundaries**: Dynamic quadtree; gateway tracks region→shard map, routes by pixel coordinate

### Simulation
- **Tick Rate**: 20 Hz (50ms), 2-tick input buffer at gateway
- **Capture Formula**: `progress += (attacker_eff / defender_eff.max(1)) * 10` where `eff = pct * (100 - fatigue) / 100`
- **Fatigue**: +2/tick attacking, +5/tick repelled, -1/tick idle
- **Supply Check**: BFS from CC through owned+supply-connected pixels; adjacent capture allowed
- **Cooldown**: 600 ticks (30s) after capture

### Persistence
- **Event Sourcing**: Every input → Redis Stream `shard:{region}`; State Service consumes → PG snapshots every 1000 ticks
- **Season Migration**: Flush streams → PG → new shards with empty grid → players place CC
- **Persisted**: techs, cosmetics, gold, guild, season stats (archived)
- **Reset**: territory, buildings, armies, resources, MMR, market offers

### Build & Dev
- **WASM**: `wasm-pack + Vite`, `wasm-bindgen + tsify` for shared types
- **Hot Reload**: `cargo watch` (Rust) + Vite HMR (TS)
- **Dev Stack**: `make up` → 6 containers (gateway, shard-1, shard-2, redis, postgres, client-dev)

## Testing Decisions

### Unit Tests (Rust)
- Capture mechanics: ratio formula, fatigue accumulation, supply BFS, cooldown enforcement
- Pixel grid: chunk allocation, neighbor queries, dirty tracking, serialization
- Tech tree: prerequisite validation, cost calculation, unlock logic
- Market: fee calculation, offer matching, atomic swap

### Integration Tests
- Gateway→Shard routing with dynamic shard map
- Client reconnection: snapshot + delta replay fast-forward
- Season transition: data migration correctness
- Replay verification: headless sim matches recorded hash

### Property Tests
- Determinism: same seed + inputs → identical state hash across runs
- Capture: progress monotonicity, fatigue bounds, supply invariants
- Economy: resource conservation (no creation/destruction except extractors)

### End-to-End (Manual/Staging)
- Multi-player capture battle across shard boundary
- Market trade with distance fee tiers
- Full season cycle: play → transition → persist → new season

## Out of Scope

- Mobile app (web-first)
- Blockchain/Web3 integration
- AI opponents (PvP only)
- Custom map editor (procedural only)
- Email verification / account recovery
- Resource-for-resource barter (Gold only)
- Soft tech prerequisites (hard requires only)
- Spectator mode (post-launch)
- Guild wars / alliance mechanics (post-launch)

## Further Notes

### Prototype-Validated Decisions
- **Pixel Grid** (`.scratch/wayfinder/prototypes/pixel-grid/`): 100k neighbors in ~6ms, 20 ticks ~41ms naive (target <10ms with dirty-chunk + Rayon)
- **Capture Mechanics** (`.scratch/wayfinder/prototypes/capture-mechanics/`): 90% vs 80% captures in ~10 ticks, fatigue creates natural pauses, supply cutoff works

### Research-Backed Decisions
- **Gateway/Shard** (`.scratch/wayfinder/research/01-gateway-shard-protocol.md`): postcard over bincode, dynamic quadtree over static, Redis Streams event sourcing, gateway-managed failover

### Glossary Terms (for `/domain-modeling`)
- **Pixel**: Atomic capturable tile on the 50k×50k grid
- **Chunk**: 64×64 tile region, unit of allocation/serialization
- **Shard**: Simulation process owning a dynamic quadtree region
- **Supply Line**: Continuous path of owned pixels to Command Center
- **Fatigue**: Combat exhaustion reducing effective army strength
- **Season**: 14-day competition cycle with partial persistence
- **Tech Tier**: One of 10 levels in a branch, unlocks buildings/units/upgrades/abilities
- **Market Building**: Unlocks trade radius (L1 local, L2 global, L3 zero-fee)

### ADRs to Create
- ADR-001: Deterministic lockstep over WebSocket
- ADR-002: Event sourcing with Redis Streams + PG snapshots
- ADR-003: Dynamic quadtree sharding for pixel grid
- ADR-004: WASM simulation + TypeScript client architecture
- ADR-005: Ratio-based capture with fatigue and supply lines