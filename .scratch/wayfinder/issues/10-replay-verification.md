# Ticket: Replay Format & Verification Pipeline — RESOLVED

## Decision

**Format**: **Binary (postcard)**
- Same serialization as game state (deterministic)
- Compact: ~50 KB per 14-day season replay (20 Hz × 1.2M ticks × ~40 bytes/tick)
- Fast to serialize/deserialize
- Header + tick-indexed chunks for seeking

**Replay Structure**:
```
ReplayFile {
    magic: u32 = 0x57415232,  // "WAR2"
    version: u16 = 1,
    header: ReplayHeader,
    tick_index: Vec<TickIndexEntry>,  // every 1000 ticks: (tick, byte_offset)
    chunks: Vec<ReplayChunk>,         // compressed tick batches
}

ReplayHeader {
    season_id: u32,
    map_seed: u64,
    started_tick: u64,
    ended_tick: u64,
    players: Vec<PlayerInfo>,         // id, username, initial_state_hash
    initial_snapshot_hash: [u8; 32],  // Blake3 of initial world state
    final_state_hash: [u8; 32],       // Blake3 of final world state
}

TickIndexEntry { tick: u32, offset: u32 }

ReplayChunk {
    start_tick: u32,
    end_tick: u32,
    data: Vec<u8>,  // postcard-encoded TickBatch[], zstd compressed
}

TickBatch {
    tick: u32,
    inputs: Vec<PlayerInput>,  // (player_id, Input)
}
```

**Verification Sampling**: **Top Players + Reported**
- Top 100 leaderboard (each category): 100% verified
- Player-reported: 100% verified (priority queue)
- Random 5% of remaining: background audit
- New accounts (< 3 seasons): 20% verified

**Headless Verification**: **Same Binary, No Rendering**
- `cargo build --release --bin verifier` (same codebase, feature flag)
- Loads initial snapshot + replay inputs
- Runs deterministic simulation tick-by-tick
- Computes final state hash (Blake3)
- Compares with `header.final_state_hash`
- Mismatch = cheat detected

**Detection**: **State Hash Mismatch**
- Primary: Blake3(final_state) ≠ header.final_state_hash
- Secondary (future): statistical anomaly (APM, perfect micro, etc.)
- False positives: log for review; version mismatch = skip

**Storage**: **Redis (Recent) + PG (Archive)**
| Tier | Location | Retention | Access |
|------|----------|-----------|--------|
| Hot (last 1000) | Redis Hash `replay:{id}` | 30 days | Instant |
| Warm (last 10k) | Redis Stream `replays:archive` | 1 year | <1s |
| Cold (all) | PG `replay_blobs` (BYTEA) | Forever | Minutes |

- Redis: `HSET replay:{id} data <blob> meta <json>`
- PG: `INSERT INTO replay_blobs (id, season_id, data, created_at) VALUES (...)`
- Background job moves Hot→Warm→Cold

**Verification Pipeline**:
```
1. Match ends → Shard writes replay to Redis (hot)
2. Verifier picks from queue (priority: top players → reported → random)
3. Verifier loads: initial snapshot (from State Service) + replay inputs
4. Verifier runs headless sim → computes final hash
5. If mismatch:
   a. Log alert with player_id, replay_id, expected/actual hash
   b. Queue for manual review (auto-ban if 3+ mismatches)
   c. Revoke season rewards (leaderboard position, cosmetics)
   d. Ban account (permanent)
6. If match: mark verified in Redis/PG
```

**Cheater Punishment**: **Ban + Season Rewards Revoked**
- Account: permanent ban (hardware ID + IP tracked)
- Season: all rewards stripped (cosmetics, titles, leaderboard position)
- Guild: member removed, bank contribution forfeited
- Appeal: manual review only (14-day cooldown)

**Client Replay Access**:
- Download via `GET /api/replay/{id}` (zstd compressed)
- Spectator mode: streams ticks at 20Hz for live viewing
- Scrubber: uses tick_index for seeking

## Blocking → Unblocked

- ✅ Unblocks: **11-client-server-protocol** (replay download messages)
- ✅ Unblocks: **12-wasm-ts-boundary** (replay types in shared schema)
- ✅ Unblocks: **13-local-dev-environment** (verifier binary in docker)