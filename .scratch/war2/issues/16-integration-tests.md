# 16: Integration & Smoke Tests

**What to build:** End-to-end integration tests covering full gameplay loop, season transition, replay verification, and multi-shard scenarios.

**Blocked by:** 08-season-system, 10-replay-verification, 11-gateway-shard-protocol, 15-client-ui-ux

**Status:** ready-for-agent

- [ ] Full gameplay loop: register → login → place CC → build extractor → assign workers → capture pixel → train units → trade → research tech
- [ ] Multi-player capture battle across shard boundary (2 shards)
- [ ] Season transition: play 1 tick before end → transition → verify persistence (techs, gold, guild) and reset (territory, MMR)
- [ ] Replay verification: record match → headless verifier confirms hash → simulate cheat → verifier detects mismatch
- [ ] Gateway failover: kill shard → gateway reassigns → new shard replays from snapshot → players reconnect
- [ ] Load test: 100 concurrent bots issuing inputs at 20 Hz for 5 minutes
- [ ] CI integration: run integration tests on PR merge to main