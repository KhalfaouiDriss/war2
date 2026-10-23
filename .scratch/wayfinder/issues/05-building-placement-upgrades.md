# Ticket: Building Placement Rules & Upgrade Paths — RESOLVED

## Decision

**Building Types (Full RTS Set)**:
| Building | Footprint | Primary Function |
|----------|-----------|------------------|
| Command Center | 3×3 | Spawn point, supply root, 1 per player |
| Wood Extractor | 1×1 | Produces Wood (worker allocation) |
| Stone Extractor | 1×1 | Produces Stone |
| Iron Extractor | 1×1 | Produces Iron |
| Food Extractor | 1×1 | Produces Food |
| Lumber Mill | 2×2 | Wood → Lumber (worker allocation) |
| Stonecutter | 2×2 | Stone → Bricks |
| Foundry | 2×2 | Iron → Steel |
| Farm | 2×2 | Food → Population |
| Barracks | 2×2 | Trains Infantry (worker allocation) |
| Factory | 2×2 | Trains Vehicles |
| Airport | 3×3 | Trains Aircraft |
| Wall | 1×1 | Defense bonus, blocks capture |
| Tower | 1×1 | Auto-defends adjacent pixels |
| Market | 2×2 | Enables P2P trade in radius |
| Laboratory | 2×2 | Generates Science (tech tree) |
| Radar | 2×2 | Reveals fog of war in radius |

**Placement Rules**: **On owned pixels only**
- Building replaces the pixel's terrain (territory → building)
- Must be placed on pixel owned by player
- Footprint pixels must all be owned (no partial placement)
- Command Center: first one free at spawn; additional CCs require tech + resources
- No placement on pixels with: existing building, enemy army >0, capture in progress

**Worker Allocation Production**:
- Population (from Food) = global worker pool
- Player assigns workers to buildings via UI
- Output = workers × base_rate × building_level_multiplier
- Example: Lumber Mill base 5 Lumber/tick/worker; 10 workers = 50 Lumber/tick
- Workers can be reassigned instantly (no cooldown)

**Upgrade System: Branch Choices** (at Level 2):
```
Level 1 → Level 2 (branch) → Level 3 (specialized)

Extractor:   L1 Basic → L2a Deep (2× output, 3× workers) 
                        L2b Wide (1.5× output, 1.5× workers, larger radius)
                        
Refinery:    L1 Basic → L2a Efficient (90% conversion vs 75%)
                        L2b Fast (2× speed, 1.2× workers)
                        
Barracks:    L1 Basic → L2a Siege (heavy units, slower)
                        L2b Mobile (fast units, weaker)
                        
Wall:        L1 Wood → L2a Stone (HP×3, fire resist)
                        L2b Reinforced (HP×2, repair faster)
                        
Tower:       L1 Basic → L2a Cannon (range+1, damage×2)
                        L2b Missile (anti-air, homing)
```

**Capture/Destruction**: **Destroyed on capture**
- When pixel captured, building is removed
- Pixel becomes empty owned territory (no building)
- Resources invested: 50% refunded to capturer as loot
- Command Center: cannot be captured (player eliminated when last CC lost)

**Upgrade Costs** (scaling):
- Level 1→2: 500 primary + 200 refined + 60s
- Level 2→3: 1500 primary + 800 refined + 180s
- Branch choice at Level 2 is permanent (cannot switch)

## Blocking → Unblocked

- ✅ Unblocks: **06-tech-tree** (buildings unlock via tech)
- ✅ Unblocks: **07-market-trade-protocol** (Market building enables trade radius)
- ✅ Unblocks: **08-season-transition** (building persistence rules)