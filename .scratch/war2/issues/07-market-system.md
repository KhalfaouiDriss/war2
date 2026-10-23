# 07: Market System

**What to build:** Gold-denominated P2P market with direct click + Market building discovery, instant atomic swap, distance-based fees (0-8%), configurable duration 1h-7d.

**Blocked by:** 06-tech-tree

**Status:** ready-for-agent

- [ ] `SellOffer`: seller_id, resource, quantity, price_per_unit (gold), duration_hours (1-168), created_tick, market_building_level
- [ ] Discovery: direct click player → see offers; Market L1 (500px radius), L2 (global), L3 (zero fees + history)
- [ ] Execution: instant atomic swap (gold buyer→seller minus fee, resources seller→buyer)
- [ ] Distance fees: Manhattan from seller's nearest Market to buyer's nearest Market
  - 0-500px: 0%, 500-2000: 2%, 2000-5000: 5%, 5000+: 8%, no market = 8%
- [ ] Market building workers reduce local fee (0.5%/worker, min 0%)
- [ ] Offer lifecycle: list → active → filled/expired/cancelled → removed
- [ ] Events: `OfferListed`, `OfferUpdated`, `OfferRemoved`, `TradeExecuted`
- [ ] Unit tests: fee calculation, atomic swap, offer matching, distance tiers