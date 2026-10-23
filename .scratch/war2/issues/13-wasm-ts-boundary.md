# 13: WASM/TypeScript Boundary

**What to build:** wasm-bindgen+tsify codegen, WASM linear memory + JS views, WASM computes/JS renders, JS rAF+accumulator tick loop, wasm-pack+Vite build, shared types auto-generated.

**Blocked by:** 02-pixel-grid-core, 03-simulation-loop, 12-client-server-protocol

**Status:** ready-for-agent

- [ ] `sim` crate: `cdylib` target, `wasm-bindgen` exports: `step(inputs)`, `get_dirty_rects()`, `get_pixel_data(rect)`, `get_state_ptr()`, `get_tick()`, `get_perf_counters()`
- [ ] Shared types: `JsInput`, `DirtyRect`, `GameState`, all messages — annotated with `#[derive(Tsify)]`
- [ ] `wasm-pack build --target web --out-dir client/pkg` integrated in Vite
- [ ] JS tick loop: `requestAnimationFrame` + accumulator (50ms), calls `wasm.step(inputs)` N times
- [ ] Rendering: JS reads dirty rects from WASM memory views → uploads to WebGL texture / Canvas
- [ ] Input: JS encodes click → postcard → `wasm.step()` → reads result
- [ ] Dev: `cargo watch` rebuilds WASM, Vite HMR reloads client
- [ ] Benchmark: 60 FPS render, 20 Hz sim, <16ms frame budget