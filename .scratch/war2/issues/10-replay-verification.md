# 10: Replay Verification

**What to build:** Binary postcard format, headless verifier (same binary), top-players+reported sampling, Blake3 state hash mismatch detection, Redis+PG storage, ban+rewards revoked.

**Blocked by:** 03-simulation-loop, 09-auth-system

**Status:** ready-for-agent

- [ ] Replay format: header (magic, version, season_id, map_seed, players, initial_hash, final_hash) + tick_index (every 1000 ticks) + zstd-compressed chunks
- [ ] Shard writes replay to Redis `replay:{id}` on match end
- [ ] Verifier binary (same sim crate, feature flag): loads snapshot + inputs → runs headless → computes final Blake3 hash
- [ ] Sampling: top 100 leaderboard 100%, reported 100%, random 5%, new accounts 20%
- [ ] Detection: hash mismatch → alert → 3+ mismatches = auto-ban + season rewards revoked
- [ ] Storage tiers: Redis hot (last 1000, 30d) → Redis warm (last 10k, 1y) → PG cold (forever)
- [ ] Client download: `GET /api/replay/{id}` (zstd compressed)
- [ ] Unit tests: format round-trip, verifier matches live sim, sampling logic