# 02: Pixel Grid Core

**What to build:** Chunked 64×64 pixel grid with sparse HashMap storage, 16-byte Tile struct, postcard serialization, and core spatial queries (neighbors, radius, rect).

**Blocked by:** 01-project-foundation

**Status:** ready-for-agent

- [ ] `ChunkCoord`, `PixelCoord`, `Tile`, `Chunk` types in `sim/src/grid/`
- [ ] `WorldGrid` with `get_tile`, `set_tile`, `get_chunk_mut` methods
- [ ] Spatial queries: `neighbors(p)`, `radius_tiles(p, r)`, `rect_tiles(min, max)`, `visible_chunks(center, radius)`
- [ ] `dirty_chunks()` and `snapshot_delta()` for tick serialization
- [ ] postcard (de)serialization for Tile, Chunk, WorldGrid
- [ ] Unit tests: neighbor lookup, radius, rect, serialization round-trip
- [ ] Benchmarks matching prototype: 100k neighbors < 10ms, 20 ticks < 10ms