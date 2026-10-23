use std::collections::HashMap;
use std::hash::BuildHasherDefault;
use ahash::AHasher;
use postcard::to_slice;
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

type FastHashMap<K, V> = HashMap<K, V, BuildHasherDefault<AHasher>>;

const CHUNK_SIZE: u32 = 64;
const CHUNK_TILES: usize = (CHUNK_SIZE * CHUNK_SIZE) as usize;
const WORLD_SIZE: u32 = 50_000;
const CHUNKS_PER_DIM: u32 = WORLD_SIZE / CHUNK_SIZE;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
#[repr(C)]
struct Tile {
    owner: u32,
    army_pct: u8,
    building: u16,
    resources: [u16; 4],
}

impl Default for Tile {
    fn default() -> Self {
        Tile { owner: 0, army_pct: 0, building: 0, resources: [0, 0, 0, 0] }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Chunk {
    #[serde(with = "BigArray")]
    tiles: [Tile; CHUNK_TILES],
    dirty: bool,
    version: u64,
}

impl Chunk {
    fn new() -> Self {
        Chunk {
            tiles: [Tile::default(); CHUNK_TILES],
            dirty: false,
            version: 0,
        }
    }
    
    fn tile_idx(x: u32, y: u32) -> usize {
        (y * CHUNK_SIZE + x) as usize
    }
    
    fn get(&self, x: u32, y: u32) -> Tile {
        self.tiles[Self::tile_idx(x, y)]
    }
    
    fn set(&mut self, x: u32, y: u32, tile: Tile) {
        self.tiles[Self::tile_idx(x, y)] = tile;
        self.dirty = true;
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct ChunkCoord {
    x: u32,
    y: u32,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct PixelCoord {
    x: u32,
    y: u32,
}

impl PixelCoord {
    fn chunk(&self) -> ChunkCoord {
        ChunkCoord { x: self.x / CHUNK_SIZE, y: self.y / CHUNK_SIZE }
    }
    
    fn local(&self) -> (u32, u32) {
        (self.x % CHUNK_SIZE, self.y % CHUNK_SIZE)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct World {
    chunks: FastHashMap<ChunkCoord, Chunk>,
    tick: u64,
}

impl World {
    fn new() -> Self {
        World { chunks: FastHashMap::default(), tick: 0 }
    }
    
    fn get_chunk(&mut self, coord: ChunkCoord) -> &mut Chunk {
        self.chunks.entry(coord).or_insert_with(Chunk::new)
    }
    
    fn get_tile(&mut self, pos: PixelCoord) -> Tile {
        let chunk_coord = pos.chunk();
        if let Some(chunk) = self.chunks.get(&chunk_coord) {
            let (lx, ly) = pos.local();
            chunk.get(lx, ly)
        } else {
            Tile::default()
        }
    }
    
    fn set_tile(&mut self, pos: PixelCoord, tile: Tile) {
        let chunk = self.get_chunk(pos.chunk());
        let (lx, ly) = pos.local();
        chunk.set(lx, ly, tile);
    }
    
    fn neighbors(&mut self, pos: PixelCoord) -> [Tile; 8] {
        let mut out = [Tile::default(); 8];
        let dirs = [(-1,-1), (0,-1), (1,-1), (-1,0), (1,0), (-1,1), (0,1), (1,1)];
        for (i, (dx, dy)) in dirs.iter().enumerate() {
            let nx = pos.x as i32 + dx;
            let ny = pos.y as i32 + dy;
            if nx >= 0 && ny >= 0 && nx < WORLD_SIZE as i32 && ny < WORLD_SIZE as i32 {
                out[i] = self.get_tile(PixelCoord { x: nx as u32, y: ny as u32 });
            }
        }
        out
    }
    
    fn radius_tiles(&mut self, center: PixelCoord, radius: u32) -> Vec<Tile> {
        let mut tiles = Vec::new();
        for dy in -(radius as i32)..=(radius as i32) {
            for dx in -(radius as i32)..=(radius as i32) {
                if dx*dx + dy*dy <= (radius as i32)*(radius as i32) {
                    let nx = center.x as i32 + dx;
                    let ny = center.y as i32 + dy;
                    if nx >= 0 && ny >= 0 && nx < WORLD_SIZE as i32 && ny < WORLD_SIZE as i32 {
                        tiles.push(self.get_tile(PixelCoord { x: nx as u32, y: ny as u32 }));
                    }
                }
            }
        }
        tiles
    }
    
    fn rect_tiles(&mut self, min: PixelCoord, max: PixelCoord) -> Vec<Tile> {
        let mut tiles = Vec::new();
        for y in min.y..=max.y {
            for x in min.x..=max.x {
                tiles.push(self.get_tile(PixelCoord { x, y }));
            }
        }
        tiles
    }
    
    fn visible_chunks(&self, center: PixelCoord, view_radius_chunks: u32) -> Vec<ChunkCoord> {
        let cc = center.chunk();
        let mut chunks = Vec::new();
        for dy in -(view_radius_chunks as i32)..=(view_radius_chunks as i32) {
            for dx in -(view_radius_chunks as i32)..=(view_radius_chunks as i32) {
                let cx = cc.x as i32 + dx;
                let cy = cc.y as i32 + dy;
                if cx >= 0 && cy >= 0 && cx < CHUNKS_PER_DIM as i32 && cy < CHUNKS_PER_DIM as i32 {
                    chunks.push(ChunkCoord { x: cx as u32, y: cy as u32 });
                }
            }
        }
        chunks
    }
    
    fn tick(&mut self) {
        self.tick += 1;
        for chunk in self.chunks.values_mut() {
            if !chunk.dirty { continue; }
            chunk.version = self.tick;
            for tile in chunk.tiles.iter_mut() {
                if tile.army_pct > 0 {
                    tile.army_pct = tile.army_pct.saturating_sub(1);
                }
                for r in tile.resources.iter_mut() {
                    *r = (*r + 1).min(1000);
                }
            }
        }
    }
    
    fn dirty_chunks(&self) -> Vec<(&ChunkCoord, &Chunk)> {
        self.chunks.iter().filter(|(_, c)| c.dirty).map(|(k, v)| (k, v)).collect()
    }
    
    fn snapshot_delta(&self) -> Vec<u8> {
        let dirty: Vec<_> = self.dirty_chunks().into_iter().collect();
        let mut buf = vec![0u8; 64*1024*1024];
        let slice = postcard::to_slice(&dirty, &mut buf).unwrap();
        slice.to_vec()
    }
    
    fn active_tiles(&self) -> usize {
        self.chunks.values().map(|c| c.tiles.iter().filter(|t| t.owner != 0).count()).sum()
    }
}

fn benchmark() {
    let mut rng = StdRng::seed_from_u64(42);
    let mut world = World::new();
    
    // Populate 10k active tiles in a concentrated area (simulates player bases)
    for _ in 0..10_000 {
        let x = rng.gen_range(1000..5000);
        let y = rng.gen_range(1000..5000);
        let pos = PixelCoord { x, y };
        let mut tile = world.get_tile(pos);
        tile.owner = rng.gen_range(1..1000);
        tile.army_pct = rng.gen_range(1..100);
        world.set_tile(pos, tile);
    }
    
    println!("Active tiles: {}", world.active_tiles());
    println!("Chunks allocated: {}", world.chunks.len());
    println!("Memory estimate: ~{} MB", world.chunks.len() * 64 / 1024);
    
    // Benchmark neighbor lookup
    let start = std::time::Instant::now();
    for _ in 0..100_000 {
        let x = rng.gen_range(1..WORLD_SIZE-1);
        let y = rng.gen_range(1..WORLD_SIZE-1);
        let _ = world.neighbors(PixelCoord { x, y });
    }
    println!("Neighbor lookup (100k): {:?}", start.elapsed());
    
    // Benchmark radius query (fog of war - small radius)
    let start = std::time::Instant::now();
    for _ in 0..1_000 {
        let x = rng.gen_range(1000..5000);
        let y = rng.gen_range(1000..5000);
        let _ = world.radius_tiles(PixelCoord { x, y }, 32);
    }
    println!("Radius 32px (1k): {:?}", start.elapsed());
    
    // Benchmark rect query (viewport 256x256)
    let start = std::time::Instant::now();
    for _ in 0..100 {
        let x = rng.gen_range(1000..4800);
        let y = rng.gen_range(1000..4800);
        let _ = world.rect_tiles(PixelCoord { x, y }, PixelCoord { x: x+256, y: y+256 });
    }
    println!("Rect 256x256 (100): {:?}", start.elapsed());
    
    // Benchmark tick update
    let start = std::time::Instant::now();
    for _ in 0..20 {
        world.tick();
    }
    println!("20 ticks: {:?}", start.elapsed());
    
    // Benchmark delta snapshot (skip - all chunks dirty after tick)
    // let start = std::time::Instant::now();
    // for _ in 0..100 {
    //     let _ = world.snapshot_delta();
    // }
    // println!("Delta snapshot (100): {:?}", start.elapsed());
    
    // Full snapshot size
    let full = {
        let mut buf = vec![0u8; 256*1024*1024];
        let slice = postcard::to_slice(&world.chunks, &mut buf).unwrap();
        slice.to_vec()
    };
    println!("Full snapshot size: {} KB", full.len() / 1024);
}

fn main() {
    benchmark();
}