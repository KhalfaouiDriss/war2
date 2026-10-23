# Ticket: WASM/TS Boundary & Shared Types — RESOLVED

## Decision

**Codegen**: **wasm-bindgen + tsify**
- Rust structs/enums annotated with `#[wasm_bindgen]` + `#[derive(Serialize, Deserialize, Tsify)]`
- `cargo build --target wasm32-unknown-unknown` → `wasm-bindgen` → `.d.ts` + `.js` glue
- Single source of truth in Rust; TypeScript types always in sync
- Version field in all messages for forward compatibility

**Memory Model**: **WASM Linear Memory + JS Views**
- WASM exports `memory` (WebAssembly.Memory)
- JS creates `Uint8Array`, `Int32Array`, `Float32Array` views into WASM memory
- State transfer: WASM writes dirty rects to shared buffer → JS reads via views → renders
- Zero-copy for pixel data (large); postcard for small messages (inputs, events)

**Rendering**: **WASM Computes, JS Renders (Canvas/WebGL)**
- WASM: simulation, dirty rect detection, pixel data preparation
- JS: `requestAnimationFrame` loop, `canvas.drawImage()` or WebGL texture upload
- WASM exports: `get_dirty_rects() → *const DirtyRect`, `get_pixel_data(rect) → *const u8`
- JS reads dirty rects → uploads only changed regions to GPU

**Input Flow**: **JS Encodes → WASM `step(inputs)`**
```rust
// WASM exports
#[wasm_bindgen]
pub fn step(inputs: &[JsInput]) -> SimResult { ... }

#[wasm_bindgen]
pub fn get_dirty_rects() -> *const DirtyRect { ... }

#[wasm_bindgen]
pub fn get_pixel_data(rect: DirtyRect) -> *const u8 { ... }

#[wasm_bindgen]
pub fn get_state_ptr() -> *const GameState { ... }  // for debugging
```

```typescript
// JS side
const inputs = encodeInputs(pendingInputs);  // postcard encode
const result = wasm.step(inputs);
const dirtyRects = readDirtyRects();
for (const rect of dirtyRects) {
  const pixels = readPixels(rect);
  renderer.uploadTexture(rect, pixels);
}
```

**Tick Loop**: **JS `requestAnimationFrame` + Accumulator**
```typescript
let accumulator = 0;
const TICK_MS = 50;  // 20 Hz

function frame(timestamp) {
  accumulator += timestamp - lastFrame;
  lastFrame = timestamp;

  while (accumulator >= TICK_MS) {
    const inputs = collectInputs();
    wasm.step(inputs);
    accumulator -= TICK_MS;
  }

  const dirtyRects = wasm.getDirtyRects();
  renderDirtyRects(dirtyRects);
  
  requestAnimationFrame(frame);
}
```
- Decouples render rate (60/144 Hz) from sim rate (20 Hz)
- Interpolation: JS can lerp between last two states for smooth rendering

**Build Toolchain**: **wasm-pack + Vite**
```
war2/
├── Cargo.toml (workspace)
├── sim/           # Rust simulation crate (cdylib target)
│   ├── src/lib.rs
│   └── Cargo.toml
├── client/        # TypeScript/React client
│   ├── src/
│   ├── vite.config.ts
│   └── package.json
└── shared/        # Shared types (optional separate crate)
```

```toml
# sim/Cargo.toml
[package]
name = "war2-sim"
crate-type = ["cdylib", "rlib"]

[dependencies]
wasm-bindgen = "0.2"
serde = { version = "1.0", features = ["derive"] }
tsify = "0.5"
postcard = { version = "1.0", features = ["use-std"] }
```

```typescript
// vite.config.ts
import { defineConfig } from 'vite';
import wasmPack from 'vite-plugin-wasm-pack';

export default defineConfig({
  plugins: [wasmPack()],
  worker: { format: 'es' },  // for SharedWorker if needed
});
```

**Shared Types** (auto-generated):
```rust
// sim/src/types.rs
#[derive(Serialize, Deserialize, Tsify, Clone)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct JsInput {
    pub kind: InputKind,
    pub tick: u32,
    pub pixel_x: u32,
    pub pixel_y: u32,
    pub army_pct: u8,
    // ...
}

#[derive(Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct DirtyRect {
    pub x: u32, pub y: u32, pub w: u32, pub h: u32,
    pub pixel_offset: u32,  // offset in linear memory
}
```

**Debug/Profiling Exports**:
- `get_state_ptr()` → raw GameState pointer (for dev tools)
- `get_tick()` → current simulation tick
- `get_perf_counters()` → sim time, entity counts, memory usage

## Blocking → Unblocked

- ✅ Unblocks: **13-local-dev-environment** (wasm-pack + Vite in docker)
- ✅ Completes all technical tickets