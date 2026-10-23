//! Capture mechanics system

use crate::grid::{WorldGrid, PixelCoord};
use crate::config::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Player identifier
pub type PlayerId = u32;

/// Result of a capture attempt
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CaptureResult {
    Captured { from: PlayerId },
    InProgress { progress: i16 },
    Held,
    Repelled { defender_remaining: u16 },
    Reinforced,
    Cooldown,
    NoSupply,
}

/// Capture system with fatigue and supply lines
pub struct CaptureSystem;

impl CaptureSystem {
    /// Effective army after fatigue
    pub fn effective_army(pct: u8, fatigue: u8) -> u16 {
        ((pct as u16) * (100 - fatigue as u16) / 100).max(1)
    }

    /// Process a capture input
    pub fn process_capture(
        world: &mut WorldGrid,
        player: PlayerId,
        pixel: PixelCoord,
        army_pct: u8,
        command_centers: &HashMap<PlayerId, PixelCoord>,
        cooldowns: &mut HashMap<PixelCoord, u16>,
    ) -> CaptureResult {
        // Check cooldown
        if let Some(&cd) = cooldowns.get(&pixel) {
            if cd > 0 {
                return CaptureResult::Cooldown;
            }
        }

        // Check supply (hard requirement)
        if !Self::has_supply(world, player, pixel, command_centers) {
            return CaptureResult::NoSupply;
        }

        let tile = world.get_tile(pixel);

        // Reinforce own territory
        if tile.owner == player {
            let new_army = (tile.army_pct + army_pct).min(100);
            let mut new_tile = tile;
            new_tile.army_pct = new_army;
            world.set_tile(pixel, new_tile);
            return CaptureResult::Reinforced;
        }

        // Calculate effective strengths
        let attacker_eff = Self::effective_army(army_pct, tile.fatigue);
        let defender_eff = Self::effective_army(tile.army_pct, tile.fatigue);

        // Failed attack
        if attacker_eff <= defender_eff {
            let mut new_tile = tile;
            new_tile.fatigue = new_tile.fatigue.saturating_add(FATIGUE_REPELLED);
            world.set_tile(pixel, new_tile);
            return CaptureResult::Repelled {
                defender_remaining: defender_eff,
            };
        }

        // Ratio-based progress
        let ratio = attacker_eff as f32 / defender_eff.max(1) as f32;
        let progress_delta = (ratio * 10.0) as i16;

        let mut new_tile = tile;
        new_tile.capture_progress = (new_tile.capture_progress + progress_delta).clamp(-100, 100);
        new_tile.fatigue = new_tile.fatigue.saturating_add(FATIGUE_ATTACK);

        // Check capture completion
        if new_tile.capture_progress >= 100 {
            new_tile.owner = player;
            new_tile.army_pct = army_pct;
            new_tile.capture_progress = 0;
            new_tile.fatigue = 0;
            cooldowns.insert(pixel, CAPTURE_COOLDOWN_TICKS);
            world.set_tile(pixel, new_tile);
            CaptureResult::Captured { from: tile.owner }
        } else if new_tile.capture_progress <= -100 {
            new_tile.capture_progress = 0;
            world.set_tile(pixel, new_tile);
            CaptureResult::Held
        } else {
            world.set_tile(pixel, new_tile);
            CaptureResult::InProgress {
                progress: new_tile.capture_progress,
            }
        }
    }

    /// Check if player has supply line to pixel
    fn has_supply(
        world: &WorldGrid,
        player: PlayerId,
        target: PixelCoord,
        command_centers: &HashMap<PlayerId, PixelCoord>,
    ) -> bool {
        // Fast path: target already supply-connected
        if let Some(tile) = world.chunks.get(&target.chunk()) {
            if let Some(t) = tile.tiles.get(Self::tile_idx(target.local().0, target.local().1)) {
                if t.owner == player && t.supply_connected {
                    return true;
                }
            }
        }

        // Check adjacent to supply
        for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
            let nx = target.x as i32 + dx;
            let ny = target.y as i32 + dy;
            if nx < 0 || ny < 0 {
                continue;
            }
            let np = PixelCoord {
                x: nx as u32,
                y: ny as u32,
            };
            if let Some(tile) = world.chunks.get(&np.chunk()) {
                if let Some(t) = tile.tiles.get(Self::tile_idx(np.local().0, np.local().1)) {
                    if t.owner == player && t.supply_connected {
                        return true;
                    }
                }
            }
        }

        // BFS from Command Center
        let Some(&cc) = command_centers.get(&player) else {
            return false;
        };
        if cc == target {
            return true;
        }

        let mut visited = std::collections::HashSet::new();
        let mut queue = vec![cc];
        visited.insert(cc);
        let mut depth = 0;

        while let Some(curr) = queue.pop() {
            if depth > 10000 {
                return false;
            }
            depth += 1;

            for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                let nx = curr.x as i32 + dx;
                let ny = curr.y as i32 + dy;
                if nx < 0 || ny < 0 {
                    continue;
                }
                let np = PixelCoord {
                    x: nx as u32,
                    y: ny as u32,
                };
                if visited.insert(np) {
                    if np == target {
                        return true;
                    }
                    if let Some(chunk) = world.chunks.get(&np.chunk()) {
                        if let Some(t) = chunk.tiles.get(Self::tile_idx(np.local().0, np.local().1)) {
                            if t.owner == player && t.supply_connected {
                                queue.push(np);
                            }
                        }
                    }
                }
            }
        }
        false
    }

    fn tile_idx(x: u32, y: u32) -> usize {
        (y * CHUNK_SIZE + x) as usize
    }

    /// Tick update: fatigue recovery, cooldown decay
    pub fn tick(
        world: &mut WorldGrid,
        cooldowns: &mut HashMap<PixelCoord, u16>,
    ) {
        // Fatigue recovery
        for chunk in world.chunks.values_mut() {
            for tile in chunk.tiles.iter_mut() {
                if tile.capture_progress == 0 && tile.fatigue > 0 {
                    tile.fatigue = tile.fatigue.saturating_sub(FATIGUE_RECOVERY);
                }
            }
        }

        // Cooldown decay
        cooldowns.retain(|_, cd| {
            *cd = cd.saturating_sub(1);
            *cd > 0
        });
    }
}