# 15: Client UI/UX

**What to build:** React UI with World Map (Canvas/WebGL), Base Builder, Army Manager, Market, Tech Tree panels, responsive pixel-art rendering.

**Blocked by:** 13-wasm-ts-boundary, 12-client-server-protocol

**Status:** ready-for-agent

- [ ] World Map: Canvas/WebGL renderer, zoom/pan, fog of war overlay, territory colors, building sprites, army indicators
- [ ] Base Builder: click owned pixel → build menu (filtered by tech) → place building (footprint preview)
- [ ] Army Manager: list armies, assign percentages to border pixels, reinforce, move
- [ ] Market: direct player offers (click avatar), Market building board (L1/L2/L3), list/buy/cancel offers
- [ ] Tech Tree: 4-branch visualization, tier nodes, prerequisites lines, research queue, progress
- [ ] HUD: resources (gold, wood, stone, iron, food, lumber, bricks, steel, population), season timer, notifications
- [ ] Responsive: desktop primary, mobile viewport (touch-friendly)
- [ ] Accessibility: keyboard nav, color-blind palette, reduced motion
- [ ] E2E test: Playwright scenario join → build → capture → trade → research