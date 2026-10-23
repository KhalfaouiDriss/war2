# 08: Season System

**What to build:** 14-day fixed seasons, ~5min hot-swap downtime, fully random map each season, persists techs/cosmetics/gold/guild, resets territory/buildings/armies/resources/MMR.

**Blocked by:** 07-market-system

**Status:** ready-for-agent

- [ ] Season timer: 14 days from start tick; triggers transition at tick boundary
- [ ] Migration pipeline: flush Redis Streams → PG snapshots → new empty shards → new map seed
- [ ] Persisted: techs (all), cosmetics, gold, guild membership, season stats (archived)
- [ ] Reset: territory, buildings, armies, resources, MMR (to 1000), market offers (cancelled, resources returned)
- [ ] Leaderboards: per-season snapshots (territory, kills, economy, tech); historical archive
- [ ] Spawn: free placement first 30 minutes; late joiners → random corner
- [ ] Events: `SeasonEnding` (5min warning), `SeasonTransition` (new season_id, map_seed)
- [ ] Integration test: full cycle play → transition → persist → new season