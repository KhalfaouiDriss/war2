# Ticket: Tech Tree Structure & Unlock Conditions — RESOLVED

## Decision

**Tree Shape**: **4 Parallel Branches** × 10 Tiers each
```
┌─────────────┬─────────────┬─────────────┬─────────────┐
│  MILITARY   │  ECONOMY    │  DEFENSE    │  UTILITY    │
├─────────────┼─────────────┼─────────────┼─────────────┤
│ T1: Infantry│ T1: Logging │ T1: Walls   │ T1: Radar   │
│ T2: Vehicles│ T2: Mining  │ T2: Towers  │ T2: Trade   │
│ T3: Aircraft│ T3: Refining│ T3: Bunkers │ T3: Spy     │
│ T4: Siege   │ T4: Farming │ T4: Shields │ T4: Diplomacy│
│ ...         │ ...         │ ...         │ ...         │
│ T10: Orbital│ T10: Auto   │ T10: Planetary│ T10: AI   │
└─────────────┴─────────────┴─────────────┴─────────────┘
```

**Research Currency**: **Laboratory Buildings**
- Each Lab generates Science/tick = workers × base_rate (10/tick/worker)
- Multiple Labs additive
- Science pools globally per player (not per Lab)
- No passive territory science (avoids snowball)

**Unlock Categories**:
| Category | Examples |
|----------|----------|
| **Buildings** | New building types, higher-level upgrades |
| **Units** | New unit classes (Infantry→Vehicle→Aircraft→Siege) |
| **Upgrades** | Branch choices at L2/L3 (e.g., Siege vs Mobile Barracks) |
| **Abilities** | Orbital strike, forced trade, spy satellite, shield overcharge |

**Season Carryover**: **All unlocked techs persist**
- Full tree retained between seasons
- No re-grinding; new seasons = fresh map, same knowledge
- Balances by: fresh territory, resource reset, new opponents

**Cost Scaling**: **Exponential per Tier**
| Tier | Science Cost | Cumulative |
|------|--------------|------------|
| 1 | 100 | 100 |
| 2 | 300 | 400 |
| 3 | 900 | 1,300 |
| 4 | 2,700 | 4,000 |
| 5 | 8,100 | 12,100 |
| 6 | 24,300 | 36,400 |
| 7 | 72,900 | 109,300 |
| 8 | 218,700 | 328,000 |
| 9 | 656,100 | 984,100 |
| 10 | 1,968,300 | 2,952,400 |

~2.9M science for full branch. At 1 Lab (10 workers) = 100 science/tick = 50 min/tier early, ~11 days for T10. Multiple Labs + workers accelerate.

**Prerequisites**: **Hard Requires (A→B)**
- Tier N requires Tier N-1 in same branch
- Cross-branch: some T3+ require T1 in another branch (e.g., Aircraft T3 requires Radar T1)
- No soft unlocks; clear progression gates

**Laboratory Building** (from Ticket 05):
- Footprint: 2×2
- Workers: up to 20
- Output: 10 science/tick/worker
- Upgrade branches: L2a Quantum (2× output) / L2b Network (shared science with allies)

## Blocking → Unblocked

- ✅ Unblocks: **07-market-trade-protocol** (Trade branch unlocks market features)
- ✅ Unblocks: **08-season-transition** (tech persistence defined)
- ✅ Unblocks: **11-client-server-protocol** (tech tree sync messages)