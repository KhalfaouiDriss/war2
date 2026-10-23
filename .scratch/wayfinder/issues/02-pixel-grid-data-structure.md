# Ticket: Pixel Grid Data Structure & Spatial Indexing

## Question

How is the massive pixel grid stored, queried, and updated? Each pixel is a capturable tile with owner, army %, building (if any), resources.

**Type**: `wayfinder:research` + `wayfinder:prototype`

## Context

- Map = pixel grid (each pixel = tile)
- Potentially millions of pixels per shard
- Need fast: owner lookup, army % update, neighbor queries, region capture detection
- Must support deterministic simulation (same input → same state)

## Sub-questions

1. **Storage**: Flat array? Quadtree? Sparse hash map? Chunked tiles?
2. **Indexing**: Spatial queries (neighbors, radius, rectangle) — how?
3. **Serialization**: Tick snapshot format for lockstep? Delta compression?
4. **Memory**: Fit in Redis? Shard-local memory + periodic flush?
5. **Prototype**: Build a Rust struct + benchmark 1M pixels @ 20 tick/s