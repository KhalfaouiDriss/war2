# 06: Tech Tree

**What to build:** 4 parallel branches × 10 tiers, Lab-generated science, exponential costs (100→300→900...), hard prerequisites, all techs persist across seasons.

**Blocked by:** 05-building-system

**Status:** ready-for-agent

- [ ] `TechBranch`: Military, Economy, Defense, Utility (each 10 tiers)
- [ ] `TechNode`: `id`, `branch`, `tier`, `cost`, `prereqs`, `unlocks` (buildings, units, upgrades, abilities)
- [ ] Science generation: Laboratory building with workers → 10 science/tick/worker
- [ ] Cost scaling: tier N = 100 × 3^(N-1) (100, 300, 900, 2700, 8100...)
- [ ] Hard prerequisites: tier N requires tier N-1 same branch; cross-branch reqs for T3+ (e.g., Aircraft T3 requires Radar T1)
- [ ] Unlocks: building types, unit types, upgrade branches, abilities (orbital strike, spy sat, etc.)
- [ ] Persistence: all unlocked techs saved per player, carried across seasons
- [ ] Research UI data: `PlayerTechs { unlocked: BitSet, available: Vec<TechId>, researching: Option<TechId>, progress: u64 }`
- [ ] Unit tests: prerequisite validation, cost calculation, unlock logic, persistence