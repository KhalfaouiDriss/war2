# 11: Gateway & Shard Protocol

**What to build:** Custom binary TCP (postcard) between gateway and shards, dynamic quadtree sharding, Redis Streams event sourcing, gateway-managed failover.

**Blocked by:** 01-project-foundation, 09-auth-system

**Status:** ready-for-agent

- [ ] Gateway: axum + tokio-tungstenite, TLS termination, JWT validation, routes WS→shard via TCP
- [ ] Shard: tokio TCP listener, postcard codec, owns dynamic quadtree region
- [ ] Protocol: `Authenticated { user_id, season_id }`, `InputBatch { tick, inputs }`, `StateDelta`, `Events`
- [ ] Dynamic sharding: gateway tracks `Region → ShardId`; splits hot regions, merges cold; shard heartbeats 1s
- [ ] Event sourcing: shard appends inputs to Redis Stream `shard:{region}`; State Service consumes → PG snapshots
- [ ] Cross-shard: client holds multiple WS (one per touched shard); global services on dedicated shards
- [ ] Failover: gateway detects dead shard (3 missed heartbeats) → reassigns region → new shard replays from snapshot
- [ ] Integration test: gateway routes to correct shard, shard split/merge, failover recovery