# 03: Deterministic Simulation Loop

**What to build:** 20 Hz lockstep simulation loop with 2-tick input buffer, deterministic tick function, and input validation.

**Blocked by:** 02-pixel-grid-core

**Status:** ready-for-agent

- [ ] `Simulation` struct holding `WorldGrid`, `tick: u64`, `input_buffer: VecDeque<InputBatch>`
- [ ] `step(inputs: &[Input]) -> TickResult` — pure deterministic function
- [ ] Input types: `Capture`, `Build`, `Train`, `Upgrade` (postcard-serializable)
- [ ] Input validation: ownership, adjacency, resources, cooldown, supply
- [ ] Per-tick processing: apply inputs → update capture progress → fatigue recovery → cooldown decay → resource production → supply re-check
- [ ] `TickResult` with `dirty_chunks`, `events`, `resource_deltas`
- [ ] Property test: same seed + inputs = identical state hash (Blake3)
- [ ] Benchmark: 20 Hz target with dirty-chunk optimization + Rayon parallelization