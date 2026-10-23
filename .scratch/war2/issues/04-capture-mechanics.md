# 04: Capture Mechanics

**What to build:** Ratio-based capture progress with fatigue, hard supply lines (BFS to CC), 30s cooldown, strongest-attacker-wins, pre-set defense%.

**Blocked by:** 03-simulation-loop

**Status:** ready-for-agent

- [ ] Capture progress formula: `progress += (attacker_eff / defender_eff.max(1)) * 10` where `eff = pct * (100 - fatigue) / 100`
- [ ] Progress range -100..100; capture at +100 (instant flip), secure at -100 (reset to 0)
- [ ] Fatigue: +2/tick attacking, +5/tick repelled, -1/tick idle (progress == 0)
- [ ] Supply check: BFS from Command Center through owned+supply-connected pixels; adjacent capture allowed
- [ ] Cooldown: 600 ticks after capture; blocks further capture attempts
- [ ] Multi-attacker: strongest effective army wins
- [ ] Defense: pre-set `defense_pct` per pixel (default 50)
- [ ] Unit tests: capture progression, fatigue bounds, supply invariants, cooldown enforcement
- [ ] Integration test: 90% vs 80% captures in ~10 ticks (from prototype)