# Ticket: Lockstep Tick Rate & Input Encoding — RESOLVED

## Decision

**Tick Rate**: **20 Hz (50ms/tick)** — balances bandwidth, server load, and game feel. Prototype showed ~41ms/tick naive; with dirty-chunk optimization + Rayon parallelization, target <10ms/tick achievable.

**Input Encoding**: Minimal action set for lockstep:
```rust
enum Input {
    Capture { pixel: PixelCoord, army_pct: u8 },  // click pixel → assign %
    Build { pixel: PixelCoord, building: BuildingType },
    Train { pixel: PixelCoord, unit: UnitType, count: u16 },
    Upgrade { pixel: PixelCoord },
    // Market, tech, chat handled via RPC (not lockstep)
}
```
Encoded as postcard bytes. ~20-40 bytes per input.

**Input Validation**: 
- **Gateway**: JWT auth, rate limiting, basic schema check
- **Shard**: Game rules (ownership, adjacent, resources, cooldowns)

**Frame Window**: **2-tick buffer**. Gateway collects inputs for ticks N and N+1, forwards batch to shard. Absorbs network jitter, keeps latency ~100ms.

**Late Join / Reconnect**: 
1. Gateway sends latest snapshot (from State Service)
2. Shard sends inputs from snapshot tick → current
3. Client fast-forwards deterministically to present

**Replay Format**: **Inputs only** (deterministic). 
```
Replay {
    version: u32,
    seed: u64,
    initial_snapshot_tick: u64,
    initial_snapshot_hash: [u8; 32],
    inputs: Vec<(tick: u32, player: u32, input_bytes: Vec<u8>)>,
}
```
Verification: Headless server loads snapshot, replays inputs, compares final state hash.

## Blocking → Unblocked

- ✅ Unblocks: **04-army-capture-mechanics** (tick rate + input type known)
- ✅ Unblocks: **10-replay-verification** (format defined)
- ✅ Unblocks: **11-client-server-protocol** (message types known)