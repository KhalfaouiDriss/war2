# Ticket: Client-Server Protocol (WebSocket Messages) — RESOLVED

## Decision

**Encoding**: **JSON** (WebSocket frames)
- Human-readable, debuggable
- Schema defined in shared TypeScript/Rust types
- Acceptable bandwidth (compression enabled)

**Transport**: **WebSocket over TLS (wss://)**
- Gateway terminates TLS, forwards to shard via TCP+postcard
- Single persistent connection per shard (multi-shard = multiple WS)
- Heartbeat: ping/pong every 30s

---

## Message Categories

| Category | Direction | Purpose |
|----------|-----------|---------|
| **Auth** | Both | Login, register, refresh, logout |
| **Input** | C→S | Lockstep actions (capture, build, train, upgrade) |
| **State** | S→C | Full snapshot, delta updates |
| **Event** | S→C | Gameplay events (capture, build complete, etc.) |
| **Chat** | Both | Global, guild, whisper, system |
| **Market** | Both | List offer, buy, cancel, offer updates |
| **System** | S→C | Season transition, maintenance, errors |

---

## Message Schemas

### Auth
```json
// C→S
{ "type": "auth.login", "payload": { "username": "str", "password": "str" } }
{ "type": "auth.register", "payload": { "username": "str", "password": "str" } }
{ "type": "auth.refresh", "payload": {} }

// S→C
{ "type": "auth.success", "payload": { "access_token": "jwt", "user": {...} } }
{ "type": "auth.error", "payload": { "code": "INVALID_CREDENTIALS", "message": "str" } }
```

### Input (Lockstep)
```json
// C→S (batched per tick by gateway)
{ "type": "input.capture", "payload": { "tick": 12345, "pixel_x": 5000, "pixel_y": 5000, "army_pct": 60 } }
{ "type": "input.build", "payload": { "tick": 12345, "pixel_x": 5000, "pixel_y": 5000, "building": "lumber_mill" } }
{ "type": "input.train", "payload": { "tick": 12345, "pixel_x": 5000, "pixel_y": 5000, "unit": "infantry", "count": 10 } }
{ "type": "input.upgrade", "payload": { "tick": 12345, "pixel_x": 5000, "pixel_y": 5000 } }
```

### State Sync
```json
// S→C: Full snapshot on join/reconnect
{ "type": "state.snapshot", "payload": {
    "tick": 12345,
    "season_id": 42,
    "player_id": 12345,
    "chunks": [  // only visible chunks (owned + border)
      { "cx": 78, "cy": 78, "tiles": [...] }
    ],
    "resources": { "gold": 5000, "wood": 1000, ... },
    "techs": [1, 5, 12, ...],
    "buildings": [...]
}}

// S→C: Delta per tick (interest-managed)
{ "type": "state.delta", "payload": {
    "tick": 12346,
    "chunk_updates": [
      { "cx": 78, "cy": 78, "tile_changes": [
        { "x": 5, "y": 3, "owner": 12345, "army_pct": 60, "building": 0, "resources": [...] }
      ]}
    ],
    "resource_changes": { "gold": -500, "wood": +50 }
}}
```

### Events
```json
// S→C
{ "type": "event.capture", "payload": { "tick": 12345, "pixel_x": 5000, "pixel_y": 5000, "attacker": 12345, "defender": 67890 } }
{ "type": "event.build_complete", "payload": { "tick": 12345, "pixel_x": 5000, "pixel_y": 5000, "building": "lumber_mill", "level": 1 } }
{ "type": "event.unit_ready", "payload": { "tick": 12345, "pixel_x": 5000, "pixel_y": 5000, "unit": "infantry", "count": 10 } }
{ "type": "event.trade_executed", "payload": { "buyer": 12345, "seller": 67890, "resource": "wood", "quantity": 100, "gold": 5000 } }
{ "type": "event.combat", "payload": { "tick": 12345, "attacker": 12345, "defender": 67890, "damage": 50, "captured": false } }
```

### Chat
```json
// C→S
{ "type": "chat.send", "payload": { "channel": "global|guild|whisper", "target": "username?", "message": "str" } }

// S→C
{ "type": "chat.message", "payload": { "channel": "global", "sender": "player1", "message": "str", "timestamp": 1699999999 } }
```

### Market
```json
// C→S
{ "type": "market.list", "payload": { "resource": "wood", "quantity": 1000, "price_per_unit": 5, "duration_hours": 24 } }
{ "type": "market.buy", "payload": { "offer_id": "uuid", "quantity": 500 } }
{ "type": "market.cancel", "payload": { "offer_id": "uuid" } }

// S→C
{ "type": "market.offer_listed", "payload": { "offer": {...} } }
{ "type": "market.offer_updated", "payload": { "offer_id": "uuid", "remaining_quantity": 500 } }
{ "type": "market.offer_removed", "payload": { "offer_id": "uuid", "reason": "filled|cancelled|expired" } }
{ "type": "market.trade_executed", "payload": { "offer_id": "uuid", "buyer": 12345, "quantity": 500, "gold_paid": 2500, "fee": 0 } }
```

### System
```json
{ "type": "system.season_ending", "payload": { "seconds_remaining": 300 } }
{ "type": "system.season_transition", "payload": { "new_season_id": 43, "map_seed": 123456789 } }
{ "type": "system.maintenance", "payload": { "message": "Server restart in 5 min", "timestamp": 1699999999 } }
{ "type": "system.error", "payload": { "code": "RATE_LIMITED", "message": "Too many requests" } }
```

---

## Interest Management

**Visible Chunks**: Owned territory + 1-chunk border
- Player sees: all chunks where they own ≥1 pixel
- Plus: all 8 neighbors of those chunks (border visibility)
- Fog of war: client-side only (server sends all visible chunk data)

**Chunk Updates**: Only changed tiles within visible chunks
- `state.delta` sent every tick (20 Hz)
- Batch size: ~50-200 tiles/tick typical
- Bandwidth: ~10-50 KB/s per player

---

## Reconnection / Late Join

1. Client reconnects → sends `auth.refresh` or stored access token
2. Gateway validates → routes to shard
3. Shard sends `state.snapshot` (full visible chunks)
4. Shard sends missed `state.delta` from snapshot tick → current
5. Client fast-forwards locally (deterministic)

---

## Rate Limits

| Message Type | Limit |
|--------------|-------|
| Input (lockstep) | 1 per tick (20/s) |
| Chat | 5/s |
| Market | 10/s |
| Auth | 5/min |

Excess → `system.error { code: "RATE_LIMITED" }`

---

## Shared Types (Ticket 12)

Generated from Rust → TypeScript via `tsify` + `wasm-bindgen`:
- All payload structs
- Enum discriminants
- Version field for forward compatibility

## Blocking → Unblocked

- ✅ Unblocks: **12-wasm-ts-boundary** (shared message types)
- ✅ Unblocks: **13-local-dev-environment** (gateway + shard in docker)