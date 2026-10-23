# Ticket: Army Capture Mechanics (Formulas, Morale, Supply Lines) — RESOLVED

## Decision

**Capture Formula**: Ratio-based per tick.
```
attacker_eff = army_pct * (100 - fatigue) / 100
defender_eff = army_pct * (100 - fatigue) / 100
progress += (attacker_eff / defender_eff.max(1)) * 10
```
- Progress range: -100..100 (negative = defender advantage)
- Capture at +100 → instant flip, 30s cooldown
- Defender secures at -100 → progress resets to 0

**Fatigue System**:
- Attacking: +2 fatigue/tick
- Repelled attack: +5 fatigue
- Recovery: -1 fatigue/tick when not in combat (progress == 0)
- Fatigue reduces effective army linearly: `eff = pct * (100 - fatigue) / 100`

**Supply Lines** (Hard requirement):
- Every captured pixel must connect to player's Command Center through owned, supply-connected pixels
- BFS from CC through owned pixels with `supply_connected=true`
- **Adjacent capture allowed**: Can target pixel adjacent to supply-connected territory
- Cut-off pixels: `supply_connected=false` → cannot reinforce, cannot capture further

**Multi-Attacker**: Strongest attacker wins (others wait/reinforce)

**Defense**: Pre-set `defense_pct` per border pixel (default 50). Defender doesn't manually assign.

**Re-capture**: 30s cooldown (600 ticks at 20Hz) after capture. During cooldown: `Cooldown` result.

**Prototype Results**:
- 90% attacker vs 80% defender → capture in ~10 ticks (0.5s at 20Hz)
- Fatigue limits sustained pushes → natural combat pauses
- Supply cutoff correctly blocks deep strikes
- Cooldown prevents ping-pong

## Blocking → Unblocked

- ✅ Unblocks: **05-building-placement-upgrades** (capture defines territory expansion)
- ✅ Unblocks: **06-tech-tree** (tech can unlock capture bonuses)
- ✅ Unblocks: **07-market-trade-protocol** (territory control affects market access)