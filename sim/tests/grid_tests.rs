//! Integration tests for pixel grid core

use postcard::{to_stdvec, from_bytes};
use war2_sim::grid::{WorldGrid, PixelCoord, ChunkCoord, Tile};
use war2_sim::config::CHUNK_SIZE;

#[test]
fn test_pixel_coord_chunk_and_local() {
    let p = PixelCoord { x: 130, y: 200 };
    let chunk = p.chunk();
    assert_eq!(chunk, ChunkCoord { x: 2, y: 3 }); // 130/64=2, 200/64=3
    
    let (lx, ly) = p.local();
    assert_eq!((lx, ly), (2, 8)); // 130%64=2, 200%64=8
}

#[test]
fn test_pixel_coord_neighbors() {
    let p = PixelCoord { x: 100, y: 100 };
    let neighbors = p.neighbors();
    
    // Should have 8 neighbors
    assert_eq!(neighbors.len(), 8);
    
    // Check specific neighbors
    assert!(neighbors.contains(&PixelCoord { x: 99, y: 99 }));
    assert!(neighbors.contains(&PixelCoord { x: 100, y: 99 }));
    assert!(neighbors.contains(&PixelCoord { x: 101, y: 99 }));
    assert!(neighbors.contains(&PixelCoord { x: 99, y: 100 }));
    assert!(neighbors.contains(&PixelCoord { x: 101, y: 100 }));
    assert!(neighbors.contains(&PixelCoord { x: 99, y: 101 }));
    assert!(neighbors.contains(&PixelCoord { x: 100, y: 101 }));
    assert!(neighbors.contains(&PixelCoord { x: 101, y: 101 }));
}

#[test]
fn test_pixel_coord_neighbors_at_edge() {
    // At world origin
    let p = PixelCoord { x: 0, y: 0 };
    let neighbors = p.neighbors();
    
    // Should only have 3 valid neighbors (others clamped to origin)
    let valid: Vec<_> = neighbors.iter()
        .filter(|n| n.x > 0 || n.y > 0)
        .collect();
    assert_eq!(valid.len(), 3);
}

#[test]
fn test_world_grid_basic_operations() {
    let mut world = WorldGrid::new();
    let pos = PixelCoord { x: 100, y: 200 };
    
    // Default tile
    let tile = world.get_tile(pos);
    assert_eq!(tile.owner, 0);
    assert_eq!(tile.army_pct, 0);
    
    // Set tile
    let new_tile = Tile {
        owner: 1,
        army_pct: 50,
        building: 0,
        resources: [100, 50, 25, 75],
        ..Default::default()
    };
    world.set_tile(pos, new_tile);
    
    // Get back
    let tile = world.get_tile(pos);
    assert_eq!(tile.owner, 1);
    assert_eq!(tile.army_pct, 50);
    assert_eq!(tile.resources, [100, 50, 25, 75]);
}

#[test]
fn test_world_grid_neighbors() {
    let mut world = WorldGrid::new();
    
    // Set up a 3x3 grid of owned tiles
    for dx in -1..=1 {
        for dy in -1..=1 {
            let pos = PixelCoord { x: (100 + dx) as u32, y: (100 + dy) as u32 };
            let mut tile = Tile::default();
            tile.owner = 1;
            tile.army_pct = 50;
            world.set_tile(pos, tile);
        }
    }
    
    // Check center pixel's neighbors
    let center = PixelCoord { x: 100, y: 100 };
    let neighbors = world.neighbors(center);
    
    // All 8 neighbors should be owned by player 1
    for neighbor in neighbors {
        assert_eq!(neighbor.owner, 1);
        assert_eq!(neighbor.army_pct, 50);
    }
}

#[test]
fn test_world_grid_radius_tiles() {
    let mut world = WorldGrid::new();
    
    // Fill a 10x10 area
    for x in 100..110 {
        for y in 100..110 {
            let pos = PixelCoord { x, y };
            let mut tile = Tile::default();
            tile.owner = 1;
            tile.army_pct = 10;
            world.set_tile(pos, tile);
        }
    }
    
    // Radius 2 from center (should get ~13 tiles for radius 2 circle)
    let center = PixelCoord { x: 105, y: 105 };
    let tiles = world.radius_tiles(center, 2);
    
    assert!(!tiles.is_empty());
    for tile in tiles {
        assert_eq!(tile.owner, 1);
    }
}

#[test]
fn test_world_grid_rect_tiles() {
    let mut world = WorldGrid::new();
    
    // Fill a rectangle
    for x in 100..110 {
        for y in 200..210 {
            let pos = PixelCoord { x, y };
            let mut tile = Tile::default();
            tile.owner = 1;
            world.set_tile(pos, tile);
        }
    }
    
    let min = PixelCoord { x: 100, y: 200 };
    let max = PixelCoord { x: 109, y: 209 };
    let tiles = world.rect_tiles(min, max);
    
    assert_eq!(tiles.len(), 100); // 10x10
    for tile in tiles {
        assert_eq!(tile.owner, 1);
    }
}

#[test]
fn test_world_grid_visible_chunks() {
    let world = WorldGrid::new();
    let center = PixelCoord { x: 1000, y: 2000 };
    
    // Visible chunks within radius 1
    let chunks = world.visible_chunks(center, 1);
    
    // Should include center chunk + 8 neighbors = 9 chunks
    assert_eq!(chunks.len(), 9);
    
    let center_chunk = center.chunk();
    assert!(chunks.contains(&center_chunk));
}

#[test]
fn test_world_grid_dirty_tracking() {
    let mut world = WorldGrid::new();
    let pos = PixelCoord { x: 100, y: 100 };
    
    // Initially no dirty chunks
    assert!(world.dirty_chunks().is_empty());
    
    // Modify a tile
    let mut tile = Tile::default();
    tile.owner = 1;
    world.set_tile(pos, tile);
    
    // Should have 1 dirty chunk
    let dirty = world.dirty_chunks();
    assert_eq!(dirty.len(), 1);
    assert!(dirty[0].1.dirty);
}

#[test]
fn test_postcard_serialization_roundtrip() {
    
    // Tile
    let tile = Tile {
        owner: 42,
        army_pct: 75,
        building: 3,
        resources: [100, 200, 300, 400],
        fatigue: 25,
        capture_progress: 50,
        supply_connected: true,
        ..Default::default()
    };
    let bytes = to_stdvec(&tile).unwrap();
    let decoded: Tile = from_bytes(&bytes).unwrap();
    assert_eq!(tile, decoded);
}

#[test]
fn test_sparse_storage() {
    let mut world = WorldGrid::new();
    
    // Place tiles far apart
    world.set_tile(PixelCoord { x: 100, y: 100 }, { let mut t = Tile::default(); t.owner = 1; t });
    world.set_tile(PixelCoord { x: 10000, y: 20000 }, { let mut t = Tile::default(); t.owner = 2; t });
    world.set_tile(PixelCoord { x: 40000, y: 30000 }, { let mut t = Tile::default(); t.owner = 3; t });
    
    // Should only have 3 chunks allocated
    assert_eq!(world.chunks.len(), 3);
    assert_eq!(world.active_tiles(), 3);
}

#[test]
fn test_chunk_coord_hash() {
    use std::collections::HashSet;
    
    let mut set = HashSet::new();
    set.insert(ChunkCoord { x: 1, y: 2 });
    set.insert(ChunkCoord { x: 2, y: 1 });
    set.insert(ChunkCoord { x: 1, y: 2 }); // duplicate
    
    assert_eq!(set.len(), 2);
}

#[test]
fn test_tile_default() {
    let tile = Tile::default();
    assert_eq!(tile.owner, 0);
    assert_eq!(tile.army_pct, 0);
    assert_eq!(tile.building, 0);
    assert_eq!(tile.resources, [0, 0, 0, 0]);
    assert_eq!(tile.fatigue, 0);
    assert_eq!(tile.capture_progress, 0);
    assert!(!tile.supply_connected);
}