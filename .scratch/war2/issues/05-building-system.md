# 05: Building System

**What to build:** 17 building types with varying footprints, placement on owned pixels only, worker allocation production, branch-choice upgrades (L2 split), destroyed on capture (50% loot).

**Blocked by:** 04-capture-mechanics

**Status:** ready-for-agent

- [ ] `BuildingType` enum with 17 variants (CC, 4 Extractors, 4 Refineries, Barracks, Factory, Airport, Wall, Tower, Market, Lab, Radar)
- [ ] `Building` struct: `type`, `level`, `branch`, `workers_assigned`, `hp`, `construction_tick`
- [ ] Footprints: CC=3×3, Extractors=1×1, Refineries/Barracks/Factory/Market/Lab/Radar=2×2, Airport=3×3, Wall/Tower=1×1
- [ ] Placement: all footprint pixels must be owned by player, no existing building, no enemy army, no capture in progress
- [ ] Worker allocation: global population pool; assign/recall instantly; output = workers × base_rate × level_multiplier
- [ ] Upgrade branches at L2: each building has 2+ branch choices (e.g., Barracks→Siege/Mobile, Extractor→Deep/Wide)
- [ ] Construction time: L1 instant, L2 60s, L3 180s; builders not required
- [ ] Capture destruction: building removed, 50% resource cost refunded as loot to capturer
- [ ] Unit tests: placement validation, worker allocation, upgrade branches, loot calculation