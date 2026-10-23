# Ticket: Market Order Book & Trade Protocol — RESOLVED

## Decision

**Discovery**: **Both Direct + Market Building**
- **Direct**: Click player avatar/base on map → see their personal sell offers
- **Market Building**: 
  - L1: Reveals offers in 500px radius
  - L2: Global market board (all player offers)
  - L3: Zero fees + price history charts

**Currency**: **Gold Only**
- All prices denominated in Gold
- No direct resource-for-resource barter
- Gold earned from: selling resources, capturing territory, combat rewards
- Creates single price discovery mechanism

**Offer Structure**:
```
SellOffer {
    seller_id: u32,
    resource: ResourceType,     // Wood, Stone, Iron, Food, Lumber, Bricks, Steel, Population, Gold
    quantity: u32,
    price_per_unit: u32,        // in Gold
    total_gold: u32,            // quantity × price
    duration_hours: u8,         // 1–168 (1h–7d), seller chooses
    created_tick: u64,
    market_building_level: u8,  // 0=direct only, 1=local, 2=global, 3=premium
}
```

**Execution**: **Instant Atomic Swap**
- Buyer clicks "Buy" → immediate transfer
- Gold: buyer → seller (minus fee)
- Resources: seller → buyer
- No escrow, no confirmation window (speed > safety for RTS)
- Transaction logged for history/replay

**Distance-Based Fees**:
| Distance (pixels) | Fee % | Paid By |
|-------------------|-------|---------|
| 0–500 (local) | 0% | — |
| 500–2000 | 2% | Seller |
| 2000–5000 | 5% | Seller |
| 5000+ | 8% | Seller |

- Distance = Manhattan from seller's nearest Market building to buyer's nearest Market building
- No Market building = max distance tier (8%)
- Encourages local trade networks, Market building placement strategy

**Offer Duration**: **Configurable 1h–7d**
- Seller chooses duration when listing
- Auto-expires at duration end; resources returned to seller
- Can cancel early (resources returned instantly)
- No relisting cooldown

**Market Building** (from Ticket 05):
- Footprint: 2×2
- L1: Local board (500px radius)
- L2: Global board (all offers)
- L3: Zero fees + price history (30-day charts)
- Workers: up to 10 (reduces local fee by 0.5%/worker, min 0%)

**Anti-Fraud**:
- No reputation system (speed priority)
- Transaction log visible to both parties
- Dispute resolution: manual admin only (rare)
- Cheat detection via replay verification (Ticket 10) catches market manipulation

**Protocol Messages** (for Ticket 11):
```
Client → Server: ListOffer(offer)
Server → Client: OfferListed(offer_id)
Client → Server: BuyOffer(offer_id, quantity)
Server → Client: TradeExecuted { resources, gold, fee }
Server → All (in radius): OfferUpdated(offer_id, remaining_qty)
Server → All (global board): OfferRemoved(offer_id)
```

## Blocking → Unblocked

- ✅ Unblocks: **08-season-transition** (market state reset rules)
- ✅ Unblocks: **11-client-server-protocol** (market message types)
- ✅ Unblocks: **12-wasm-ts-boundary** (market UI data structures)