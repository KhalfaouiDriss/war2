//! Benchmarks for pixel grid core

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use war2_sim::grid::{WorldGrid, PixelCoord, Tile};
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

fn benchmark_neighbor_lookup(c: &mut Criterion) {
    let mut rng = StdRng::seed_from_u64(42);
    let mut world = WorldGrid::new();
    
    // Populate 10k active tiles in a concentrated area
    for _ in 0..10_000 {
        let x = rng.gen_range(1000..5000);
        let y = rng.gen_range(1000..5000);
        let pos = PixelCoord { x, y };
        let mut tile = world.get_tile(pos);
        tile.owner = rng.gen_range(1..1000);
        tile.army_pct = rng.gen_range(1..100);
        world.set_tile(pos, tile);
    }
    
    c.bench_function("neighbor_lookup_100k", |b| {
        b.iter(|| {
            for _ in 0..100_000 {
                let x = rng.gen_range(1001..4999);
                let y = rng.gen_range(1001..4999);
                let _ = black_box(world.neighbors(PixelCoord { x, y }));
            }
        })
    });
}

fn benchmark_radius_query(c: &mut Criterion) {
    let mut rng = StdRng::seed_from_u64(42);
    let mut world = WorldGrid::new();
    
    for _ in 0..10_000 {
        let x = rng.gen_range(1000..5000);
        let y = rng.gen_range(1000..5000);
        let pos = PixelCoord { x, y };
        let mut tile = world.get_tile(pos);
        tile.owner = rng.gen_range(1..1000);
        tile.army_pct = rng.gen_range(1..100);
        world.set_tile(pos, tile);
    }
    
    c.bench_function("radius_32px_1k", |b| {
        b.iter(|| {
            for _ in 0..1_000 {
                let x = rng.gen_range(1032..4968);
                let y = rng.gen_range(1032..4968);
                let _ = black_box(world.radius_tiles(PixelCoord { x, y }, 32));
            }
        })
    });
}

fn benchmark_rect_query(c: &mut Criterion) {
    let mut rng = StdRng::seed_from_u64(42);
    let mut world = WorldGrid::new();
    
    for _ in 0..10_000 {
        let x = rng.gen_range(1000..5000);
        let y = rng.gen_range(1000..5000);
        let pos = PixelCoord { x, y };
        let mut tile = world.get_tile(pos);
        tile.owner = rng.gen_range(1..1000);
        tile.army_pct = rng.gen_range(1..100);
        world.set_tile(pos, tile);
    }
    
    c.bench_function("rect_256x256_100", |b| {
        b.iter(|| {
            for _ in 0..100 {
                let x = rng.gen_range(1000..4800);
                let y = rng.gen_range(1000..4800);
                let _ = black_box(world.rect_tiles(
                    PixelCoord { x, y },
                    PixelCoord { x: x + 256, y: y + 256 }
                ));
            }
        })
    });
}

fn benchmark_tick(c: &mut Criterion) {
    let mut rng = StdRng::seed_from_u64(42);
    let mut world = WorldGrid::new();
    
    for _ in 0..10_000 {
        let x = rng.gen_range(1000..5000);
        let y = rng.gen_range(1000..5000);
        let pos = PixelCoord { x, y };
        let mut tile = world.get_tile(pos);
        tile.owner = rng.gen_range(1..1000);
        tile.army_pct = rng.gen_range(1..100);
        world.set_tile(pos, tile);
    }
    
    c.bench_function("20_ticks", |b| {
        b.iter(|| {
            let mut w = world.clone();
            for _ in 0..20 {
                w.tick = w.tick + 1;
                for chunk in w.chunks.values_mut() {
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
            black_box(w);
        })
    });
}

fn benchmark_snapshot_delta(c: &mut Criterion) {
    let mut rng = StdRng::seed_from_u64(42);
    let mut world = WorldGrid::new();
    
    for _ in 0..10_000 {
        let x = rng.gen_range(1000..5000);
        let y = rng.gen_range(1000..5000);
        let pos = PixelCoord { x, y };
        let mut tile = world.get_tile(pos);
        tile.owner = rng.gen_range(1..1000);
        tile.army_pct = rng.gen_range(1..100);
        world.set_tile(pos, tile);
    }
    
    c.bench_function("delta_snapshot", |b| {
        b.iter(|| {
            let _ = black_box(world.snapshot_delta());
        })
    });
}

criterion_group!(
    benches,
    benchmark_neighbor_lookup,
    benchmark_radius_query,
    benchmark_rect_query,
    benchmark_tick,
    benchmark_snapshot_delta
);
criterion_main!(benches);