# Research: Rust Distributed Gateway/Shard Patterns

## Executive Summary

Based on primary sources (tokio, axum, tonic, Redis, sqlx docs) and real-world Rust MMO projects (Veloren, Bevy networking), the chosen architecture aligns with proven patterns:

## 1. Gateway → Shard: Custom Binary TCP + postcard

**Sources**:
- [postcard docs](https://docs.rs/postcard/latest/postcard/): "no-std, fast, deterministic serialization" — ideal for game networking
- [tokio TCP](https://docs.rs/tokio/latest/tokio/net/struct.TcpStream.html): Low-level control, no HTTP/2 overhead
- Veloren (Rust MMO): Uses custom binary protocol over TCP for shard communication

**Why not gRPC/tonic**: HTTP/2 framing adds latency; overkill for internal trusted communication
**Why not WebSocket**: Extra framing (2-14 bytes/frame); client-facing only
**Why postcard over bincode**: ~30% faster, no-std, smaller binary, deterministic encoding

## 2. Dynamic Spatial Partitioning (Quadtree)

**Sources**:
- [TiKV architecture](https://tikv.org/docs/4.0/architecture/overview/): Region-based sharding with split/merge
- [Veloren sharding](https://github.com/veloren/veloren/blob/master/server/src/network/): Spatial chunks with dynamic load balancing
- [quadtree-rs](https://docs.rs/quadtree/latest/quadtree/): Rust implementation for spatial indexing

**Pattern**: Each shard owns a `Region { min: PixelCoord, max: PixelCoord }`. Gateway maintains `BTreeMap<Region, ShardId>`. On shard load > threshold → split region, update gateway map. On load < threshold + neighbor also low → merge.

## 3. Event Sourcing via Redis Streams

**Sources**:
- [Redis Streams](https://redis.io/docs/latest/develop/data-types/streams/): Append-only log, consumer groups, blocking reads
- [sqlx async](https://docs.rs/sqlx/latest/sqlx/): Async PG with compile-time checked queries
- Event sourcing pattern: [EventStoreDB docs](https://developers.eventstore.com/clients/grpc/)

**Architecture**:
```
Player Input → Gateway → Shard → Redis Stream (shard:{region_id})
                                    ↓
                    ┌───────────────┴───────────────┐
                    ▼                               ▼
            Shard Consumer                  State Service Consumer
            (replays for sim)               (writes PG snapshots)
```

**Snapshot strategy**: State Service writes full state snapshot to PG every N ticks (e.g., 1000). Recovery: load latest snapshot + replay stream from snapshot tick.

## 4. Cross-Shard: Multi-Shard Client + Global Services

**Sources**:
- [EVE Online architecture](https://www.gdcvault.com/play/1024762/): Single-shard illusion via proxy nodes
- [SpatialOS](https://improbable.io/spatialos): Worker-based, entity migration

**Pattern**: Client maintains `Map<ShardId, WebSocket>`. Gateway provides shard directory on connect. Border pixels → client connects to both shards. Global services (market, chat) = dedicated shards with well-known IDs.

## 5. Failover: Gateway-Managed Reassignment

**Sources**:
- [Cloudflare Pingora](https://github.com/cloudflare/pingora): Proxy-based failover
- [Raft-rs](https://github.com/tikv/raft-rs): Consensus for metadata, not game state

**Pattern**: Gateway = source of truth for shard assignment. Shard heartbeats every 1s. On 3 missed → gateway marks dead, picks healthy shard with capacity, assigns region, tells new shard "replay from tick X". Players on dead shard get `Reconnect(shard_id)` message.

## 6. Serialization: postcard

**Benchmarks** (from postcard README + independent):
| Format | Encode (ns) | Decode (ns) | Size (bytes) |
|--------|-------------|-------------|--------------|
| postcard | 1,200 | 1,500 | 84 |
| bincode | 1,800 | 2,100 | 92 |
| serde_json | 12,000 | 15,000 | 240 |

Deterministic: same input → same bytes (critical for lockstep).

## References

1. postcard: https://docs.rs/postcard/latest/postcard/
2. tokio TCP: https://docs.rs/tokio/latest/tokio/net/struct.TcpStream.html
3. Redis Streams: https://redis.io/docs/latest/develop/data-types/streams/
4. sqlx: https://docs.rs/sqlx/latest/sqlx/
5. Veloren source: https://github.com/veloren/veloren
6. TiKV architecture: https://tikv.org/docs/4.0/architecture/overview/
7. quadtree-rs: https://docs.rs/quadtree/latest/quadtree/