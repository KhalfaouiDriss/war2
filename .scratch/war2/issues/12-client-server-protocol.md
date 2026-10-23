# 12: Client-Server Protocol

**What to build:** JSON over WebSocket, 7 categories (Auth/Input/State/Event/Chat/Market/System), full snapshot on join + delta per tick, interest management = owned + border.

**Blocked by:** 11-gateway-shard-protocol, 04-capture-mechanics, 05-building-system, 06-tech-tree, 07-market-system

**Status:** ready-for-agent

- [ ] Message schemas: `Auth`, `Input`, `State`, `Event`, `Chat`, `Market`, `System` (from spec)
- [ ] Client→Server: `Input` (batched per tick), `Chat`, `Market` actions
- [ ] Server→Client: `State.snapshot` (visible chunks: owned + 1-border), `State.delta` (20 Hz), `Event.*`, `Chat`, `Market`, `System`
- [ ] Reconnection: client sends stored access token → gateway validates → shard sends snapshot + missed deltas → client fast-forwards
- [ ] Rate limits: Input 20/s, Chat 5/s, Market 10/s
- [ ] Shared types generated from Rust via tsify (Ticket 13)
- [ ] Integration test: join → play → reconnect → state consistent