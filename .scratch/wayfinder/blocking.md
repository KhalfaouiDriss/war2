# Blocking Relationships (Frontier Order)

```
01-gateway-shard-protocol
    ↓ blocks
02-pixel-grid-data-structure
    ↓ blocks
03-lockstep-tick-rate
    ↓ blocks
04-army-capture-mechanics
    ↓ blocks
05-building-placement-upgrades
    ↓ blocks
06-tech-tree
    ↓ blocks
07-market-trade-protocol
    ↓ blocks
08-season-transition
    ↓ blocks
09-auth-flow
    ↓ blocks
10-replay-verification
    ↓ blocks
11-client-server-protocol
    ↓ blocks
12-wasm-ts-boundary
    ↓ blocks
13-local-dev-environment
```

**Frontier (start here)**: `01-gateway-shard-protocol` — no blockers

**Parallelizable after 01**: 02, 09, 13 (independent of simulation core)

**Notes**:
- 02 (pixel grid) needs 01 (shard boundaries)
- 03 (lockstep) needs 02 (state representation)
- 04 (combat) needs 03 (tick/input model)
- 05, 06, 07 depend on 04 (core loop defined)
- 08 (seasons) needs 05, 06 (what persists)
- 10 (replays) needs 03 (tick format)
- 11 (protocol) needs 03, 04 (message types)
- 12 (WASM/TS) needs 11 (protocol)
- 13 (dev env) can start anytime