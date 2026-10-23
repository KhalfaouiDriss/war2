# Ticket: Gateway/Shard Protocol & State Sync — RESOLVED

## Decision

**Gateway → Shard**: Custom binary TCP protocol using **postcard** serialization. Gateway terminates TLS, validates JWT, routes to shard via persistent TCP connections.

**Sharding**: **Dynamic spatial partitioning** (quadtree). Shards own rectangular regions; hot regions split, cold regions merge. Gateway tracks shard bounds, routes by pixel coordinate.

**State Sync**: **Event sourcing via Redis Streams**. Every deterministic input appended to stream. Shards consume own region's stream; State Service consumes all → writes snapshots to PostgreSQL. Redis = hot log, PG = cold archive.

**Cross-Shard**: 
- **Multi-shard client connections**: Client holds WebSocket to each shard its territory touches
- **Dedicated global service shards**: Market, chat, leaderboards on separate shards (not spatial)

**Failover**: **Gateway-managed reassignment**. Gateway heartbeats shards; on failure, reassigns region to healthy shard, replays Redis Stream from last snapshot.

**Serialization**: **postcard** (no-std, faster than bincode, deterministic).

## Architecture Summary

```
Client (WASM/TS)
    │  WebSocket (wss://)
    ▼
Gateway (axum + tokio-tungstenite)
    │  TLS termination, JWT validation
    │  TCP + postcard → Shard
    ▼
┌─────────────────────────────────────┐
│  World Shards (dynamic quadtree)    │
│  - Owns region of pixel grid        │
│  - Deterministic simulation         │
│  - Appends inputs to Redis Stream   │
└─────────────────────────────────────┘
    │  Redis Streams (event log)
    ▼
┌─────────────────────────────────────┐
│  State Services                     │
│  - Consumes streams → PG snapshots  │
│  - Global services (market, chat)   │
└─────────────────────────────────────┘
```

## Blocking → Unblocked

- ✅ Unblocks: **02-pixel-grid-data-structure** (shard boundaries defined)
- ✅ Unblocks: **03-lockstep-tick-rate** (transport + serialization known)
- ✅ Unblocks: **09-auth-flow** (gateway validates JWT)
- ✅ Unblocks: **13-local-dev-environment** (service topology known)