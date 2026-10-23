# Ticket: Season Transition & Data Migration — RESOLVED

## Decision

**Trigger**: **Time-based (14 days)**
- Fixed 2-week seasons
- No early end condition (simpler, predictable)

**Downtime**: **Minutes (hot swap)**
- Season tick N ends → maintenance window begins
- State Service computes migrations
- New shards spawn with fresh map
- Players reconnect to new season (gateway directs)

**Map Generation**: **Fully Random Each Season**
- New terrain, resource deposits, layout every season
- No fixed seed (prevents map memorization advantage)
- Resource distribution balanced by algorithm (not symmetric)

**Persistence Rules**:

| Category | Persists | Resets |
|----------|----------|--------|
| **Techs** | ✅ All unlocked | — |
| **Cosmetics** | ✅ Skins, titles, emotes | — |
| **Gold (Money)** | ✅ Carryover | — |
| **MMR/Rating** | ❌ | Reset (non-helpful statistic) |
| **Season Stats** | ✅ Archived per-season | Current season reset |
| **Guild/Alliance** | ✅ Membership | — |
| **Territory** | ❌ | Full reset |
| **Buildings** | ❌ | Destroyed |
| **Armies** | ❌ | Disbanded |
| **Resources** | ❌ | Zeroed |
| **Market Offers** | ❌ | Cancelled (resources returned) |

**Leaderboards**: **Per-Season Snapshots**
- Each season: separate leaderboard (territory, kills, economy, tech)
- Historical archive: previous seasons viewable
- No all-time cumulative (avoids snowball)

**New Season Spawn**: **Free Placement (First Come)**
- Map opens → players click to place initial Command Center
- First-come advantage for prime spots
- Spawn window: first 30 minutes of season
- Late joiners: random available corner

**Migration Pipeline** (runs during downtime):
```
1. Gateway stops accepting new connections
2. Shards flush dirty chunks → Redis Streams
3. State Service reads streams → writes final snapshots to PG
4. Migration job:
   FOR each player:
     - Copy: techs, cosmetics, gold, guild_id, season_stats
     - Archive: current season stats → season_leaderboards
     - Reset: MMR = 1000 (base)
   FOR each guild:
     - Copy: members, name, tag, bank
5. Map Generator creates new world (random seed)
6. New shards start with empty pixel grid
7. Gateway updates shard map → new shards
8. Gateway accepts connections → players place CC
```

**Timing**: ~5 minutes typical downtime
- Parallelizable across shards
- PG writes batched (COPY)
- Map generation: procedural, <30s

**Edge Cases**:
- Player mid-trade: offer cancelled, resources returned to seller
- Player mid-capture: progress lost, cooldown applies in new season
- Guild bank: persists (guild is cross-season entity)

## Blocking → Unblocked

- ✅ Unblocks: **09-auth-flow** (season_id in JWT)
- ✅ Unblocks: **11-client-server-protocol** (season transition messages)
- ✅ Unblocks: **13-local-dev-environment** (season cycle in dev)