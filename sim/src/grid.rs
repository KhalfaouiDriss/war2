//! Pixel grid implementation with chunked storage

use ahash::AHasher;
use postcard::to_extend;
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;
use std::collections::HashMap;
use std::hash::BuildHasherDefault;

use crate::config::{CHUNK_SIZE, CHUNK_TILES, WORLD_SIZE};

type FastHashMap<K, V> = HashMap<K, V, BuildHasherDefault<AHasher>>;

/// 2D coordinate in chunk space
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ChunkCoord {
    pub x: u32,
    pub y: u32,
}

/// 2D coordinate in pixel space
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PixelCoord {
    pub x: u32,
    pub y: u32,
}

impl PixelCoord {
    pub fn chunk(&self) -> ChunkCoord {
        ChunkCoord {
            x: self.x / CHUNK_SIZE,
            y: self.y / CHUNK_SIZE,
        }
    }

    pub fn local(&self) -> (u32, u32) {
        (self.x % CHUNK_SIZE, self.y % CHUNK_SIZE)
    }

    pub fn neighbors(&self) -> [PixelCoord; 8] {
        let mut out = [PixelCoord { x: 0, y: 0 }; 8];
        let dirs = [(-1, -1), (0, -1), (1, -1), (-1, 0), (1, 0), (-1, 1), (0, 1), (1, 1)];
        for (i, (dx, dy)) in dirs.iter().enumerate() {
            let nx = self.x as i32 + dx;
            let ny = self.y as i32 + dy;
            if nx >= 0 && ny >= 0 && nx < WORLD_SIZE as i32 && ny < WORLD_SIZE as i32 {
                out[i] = PixelCoord { x: nx as u32, y: ny as u32 };
            }
        }
        out
    }
}

/// 20-byte tile data (16-byte + fatigue + capture_progress + supply_connected)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[repr(C)]
pub struct Tile {
    pub owner: u32,
    pub army_pct: u8,
    pub building: u16,
    pub resources: [u16; 4], // wood, stone, iron, food
    pub fatigue: u8,
    pub capture_progress: i16,
    pub supply_connected: bool,
    _pad: u8, // padding for alignment
}

impl Default for Tile {
    fn default() -> Self {
        Tile {
            owner: 0,
            army_pct: 0,
            building: 0,
            resources: [0, 0, 0, 0],
            fatigue: 0,
            capture_progress: 0,
            supply_connected: false,
            _pad: 0,
        }
    }
}

/// Chunk of 64x64 = 4096 tiles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    #[serde(with = "BigArray")]
    pub tiles: [Tile; CHUNK_TILES],
    pub dirty: bool,
    pub version: u64,
}

impl Chunk {
    pub fn new() -> Self {
        Chunk {
            tiles: [Tile::default(); CHUNK_TILES],
            dirty: false,
            version: 0,
        }
    }

    fn tile_idx(x: u32, y: u32) -> usize {
        (y * CHUNK_SIZE + x) as usize
    }

    pub fn get(&self, x: u32, y: u32) -> Tile {
        self.tiles[Self::tile_idx(x, y)]
    }

    pub fn set(&mut self, x: u32, y: u32, tile: Tile) {
        self.tiles[Self::tile_idx(x, y)] = tile;
        self.dirty = true;
    }
}

/// World grid with chunked sparse storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldGrid {
    pub chunks: FastHashMap<ChunkCoord, Chunk>,
    pub tick: u64,
}

impl WorldGrid {
    pub fn new() -> Self {
        WorldGrid {
            chunks: FastHashMap::default(),
            tick: 0,
        }
    }

    pub fn get_chunk(&mut self, coord: ChunkCoord) -> &mut Chunk {
        self.chunks.entry(coord).or_insert_with(Chunk::new)
    }

    pub fn get_tile(&mut self, pos: PixelCoord) -> Tile {
        let chunk = self.get_chunk(pos.chunk());
        let (lx, ly) = pos.local();
        chunk.get(lx, ly)
    }

    pub fn set_tile(&mut self, pos: PixelCoord, tile: Tile) {
        let chunk = self.get_chunk(pos.chunk());
        let (lx, ly) = pos.local();
        chunk.set(lx, ly, tile);
    }

    pub fn neighbors(&mut self, pos: PixelCoord) -> [Tile; 8] {
        let mut out = [Tile::default(); 8];
        for (i, npos) in pos.neighbors().iter().enumerate() {
            out[i] = self.get_tile(*npos);
        }
        out
    }

    pub fn radius_tiles(&mut self, center: PixelCoord, radius: u32) -> Vec<Tile> {
        let mut tiles = Vec::new();
        let r2 = radius * radius;
        for dy in -(radius as i32)..=(radius as i32) {
            for dx in -(radius as i32)..=(radius as i32) {
                if dx * dx + dy * dy <= r2 as i32 {
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

    pub fn rect_tiles(&mut self, min: PixelCoord, max: PixelCoord) -> Vec<Tile> {
        let mut tiles = Vec::new();
        for y in min.y..=max.y {
            for x in min.x..=max.x {
                tiles.push(self.get_tile(PixelCoord { x, y }));
            }
        }
        tiles
    }

    pub fn visible_chunks(&self, center: PixelCoord, view_radius_chunks: u32) -> Vec<ChunkCoord> {
        let cc = center.chunk();
        let mut chunks = Vec::new();
        for dy in -(view_radius_chunks as i32)..=(view_radius_chunks as i32) {
            for dx in -(view_radius_chunks as i32)..=(view_radius_chunks as i32) {
                let cx = cc.x as i32 + dx;
                let cy = cc.y as i32 + dy;
                if cx >= 0 && cy >= 0 {
                    chunks.push(ChunkCoord { x: cx as u32, y: cy as u32 });
                }
            }
        }
        chunks
    }

    pub fn dirty_chunks(&self) -> Vec<(&ChunkCoord, &Chunk)> {
        self.chunks.iter().filter(|(_, c)| c.dirty).collect()
    }

    pub fn snapshot_delta(&self) -> Vec<u8> {
        let dirty: Vec<_> = self.dirty_chunks().into_iter().collect();
        let buf = Vec::new();
        let buf = postcard::to_extend(&dirty, buf).unwrap();
        buf
    }

    pub fn active_tiles(&self) -> usize {
        self.chunks
            .values()
            .map(|c| c.tiles.iter().filter(|t| t.owner != 0).count())
            .sum()
    }
}

impl Default for WorldGrid {
    fn default() -> Self {
        Self::new()
    }
}