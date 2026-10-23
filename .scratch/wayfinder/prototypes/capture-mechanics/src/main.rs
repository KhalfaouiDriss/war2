use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
struct PixelCoord {
    x: u32,
    y: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
enum Player {
    None = 0,
    P1 = 1,
    P2 = 2,
    P3 = 3,
}

impl From<u32> for Player {
    fn from(v: u32) -> Self {
        match v { 0 => Player::None, 1 => Player::P1, 2 => Player::P2, 3 => Player::P3, _ => Player::None }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PixelState {
    owner: Player,
    army_pct: u8,        // 0-100
    defense_pct: u8,     // pre-set defense %
    capture_progress: i16, // -100..100 (negative = defender advantage)
    fatigue: u8,         // 0-100, reduces effective army
    supply_connected: bool,
    cooldown_ticks: u16, // re-capture cooldown
}

impl Default for PixelState {
    fn default() -> Self {
        PixelState {
            owner: Player::None,
            army_pct: 0,
            defense_pct: 50,
            capture_progress: 0,
            fatigue: 0,
            supply_connected: false,
            cooldown_ticks: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CaptureInput {
    pixel: PixelCoord,
    army_pct: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct World {
    pixels: HashMap<PixelCoord, PixelState>,
    command_centers: HashMap<Player, PixelCoord>,
    tick: u64,
}

impl World {
    fn new() -> Self {
        let mut cc = HashMap::new();
        cc.insert(Player::P1, PixelCoord { x: 100, y: 5000 });
        cc.insert(Player::P2, PixelCoord { x: 9900, y: 5000 });
        World { pixels: HashMap::new(), command_centers: cc, tick: 0 }
    }
    
    fn get(&mut self, p: PixelCoord) -> &mut PixelState {
        self.pixels.entry(p).or_default()
    }
    
    fn supply_check(&self, player: Player, target: PixelCoord) -> bool {
        // Fast path: if target already has supply_connected and is owned by player
        if let Some(state) = self.pixels.get(&target) {
            if state.owner == player && state.supply_connected {
                return true;
            }
        }
        
        // Allow capture adjacent to supply-connected territory
        // Check if any neighbor of target is supply-connected and owned by player
        for (dx, dy) in [(-1,0),(1,0),(0,-1),(0,1)] {
            let nx = target.x as i32 + dx;
            let ny = target.y as i32 + dy;
            if nx < 0 || ny < 0 { continue; }
            let np = PixelCoord { x: nx as u32, y: ny as u32 };
            if let Some(state) = self.pixels.get(&np) {
                if state.owner == player && state.supply_connected {
                    return true;
                }
            }
        }
        
        // Simple BFS from CC to target through owned pixels (limited depth)
        let Some(&cc) = self.command_centers.get(&player) else { return false; };
        if cc == target { return true; }
        
        let mut visited = std::collections::HashSet::new();
        let mut queue = vec![cc];
        visited.insert(cc);
        let mut depth = 0;
        
        while let Some(curr) = queue.pop() {
            if depth > 10000 { return false; } // limit BFS
            depth += 1;
            
            for (dx, dy) in [(-1,0),(1,0),(0,-1),(0,1)] {
                let nx = curr.x as i32 + dx;
                let ny = curr.y as i32 + dy;
                if nx < 0 || ny < 0 { continue; }
                let np = PixelCoord { x: nx as u32, y: ny as u32 };
                if visited.insert(np) {
                    if np == target { return true; }
                    if let Some(state) = self.pixels.get(&np) {
                        if state.owner == player && state.supply_connected {
                            queue.push(np);
                        }
                    }
                }
            }
        }
        false
    }
    
    fn process_capture(&mut self, player: Player, input: CaptureInput) -> CaptureResult {
        // Supply check (hard requirement) - do this first, before mutable borrow
        let has_supply = self.supply_check(player, input.pixel);
        
        let state = self.get(input.pixel);
        
        // Cooldown check
        if state.cooldown_ticks > 0 {
            return CaptureResult::Cooldown;
        }
        
        // Supply check
        if !has_supply {
            return CaptureResult::NoSupply;
        }
        
        // Owner check
        if state.owner == player {
            // Reinforce own pixel
            state.army_pct = (state.army_pct + input.army_pct).min(100);
            return CaptureResult::Reinforced;
        }
        
        // Find strongest attacker (for multi-attacker: strongest wins)
        let attacker_strength = effective_army(input.army_pct, state.fatigue);
        let defender_strength = effective_army(state.army_pct, state.fatigue);
        
        if attacker_strength <= defender_strength {
            // Failed attack - defender holds, attacker loses some army
            state.fatigue = (state.fatigue + 5).min(100);
            return CaptureResult::Repelled { defender_remaining: defender_strength };
        }
        
        // Ratio-based capture progress: progress += attacker/defender ratio
        let ratio = attacker_strength as f32 / defender_strength.max(1) as f32;
        let progress_delta = (ratio * 10.0) as i16; // ~10 progress per tick at 1:1
        
        state.capture_progress = (state.capture_progress + progress_delta).clamp(-100, 100);
        state.fatigue = (state.fatigue + 2).min(100); // fatigue accumulates
        
        if state.capture_progress >= 100 {
            // Capture complete - instant flip
            let old_owner = state.owner;
            state.owner = player;
            state.army_pct = input.army_pct;
            state.capture_progress = 0;
            state.fatigue = 0;
            state.cooldown_ticks = 600; // 30s at 20Hz
            state.supply_connected = has_supply;
            CaptureResult::Captured { from: old_owner }
        } else if state.capture_progress <= -100 {
            // Defender pushes back to secure
            state.capture_progress = 0;
            CaptureResult::Held
        } else {
            CaptureResult::InProgress { progress: state.capture_progress }
        }
    }
    
    fn tick(&mut self) {
        self.tick += 1;
        
        // Collect supply check results first
        let supply_results: Vec<(PixelCoord, bool)> = self.pixels.iter()
            .filter(|(_, s)| s.owner != Player::None)
            .map(|(p, s)| (*p, self.supply_check(s.owner, *p)))
            .collect();
        
        for state in self.pixels.values_mut() {
            // Fatigue recovery when not in combat
            if state.capture_progress == 0 && state.fatigue > 0 {
                state.fatigue = state.fatigue.saturating_sub(1);
            }
            // Cooldown decay
            if state.cooldown_ticks > 0 {
                state.cooldown_ticks -= 1;
            }
        }
        
        // Apply supply results
        for (p, connected) in supply_results {
            if let Some(state) = self.pixels.get_mut(&p) {
                state.supply_connected = connected;
            }
        }
    }
}

fn effective_army(pct: u8, fatigue: u8) -> u16 {
    // Fatigue reduces effective strength linearly
    (pct as u16 * (100 - fatigue as u16) / 100).max(1)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum CaptureResult {
    Captured { from: Player },
    InProgress { progress: i16 },
    Held,
    Repelled { defender_remaining: u16 },
    Reinforced,
    Cooldown,
    NoSupply,
}

fn simulate_battle() {
    let mut rng = StdRng::seed_from_u64(42);
    let mut world = World::new();
    
    // Set up initial border - connect to CCs
    for x in 100..6000 {
        let p = PixelCoord { x, y: 5000 };
        world.get(p).owner = Player::P1;
        world.get(p).army_pct = 80;
        world.get(p).defense_pct = 60;
        world.get(p).supply_connected = true;
    }
    for x in 6000..10000 {
        let p = PixelCoord { x, y: 5000 };
        world.get(p).owner = Player::P2;
        world.get(p).army_pct = 80;
        world.get(p).defense_pct = 60;
        world.get(p).supply_connected = true;
    }
    
    println!("Initial border: P1 4000-6000, P2 6000-8000 at y=5000");
    
    // P1 attacks P2's border at x=6000 with stronger army
    let attack_pixel = PixelCoord { x: 6000, y: 5000 };
    
    for tick in 0..200 {
        let input = CaptureInput { pixel: attack_pixel, army_pct: 90 };
        let result = world.process_capture(Player::P1, input);
        
        if tick % 20 == 0 || matches!(result, CaptureResult::Captured{..}) {
            println!("Tick {}: {:?}", tick, result);
        }
        
        if matches!(result, CaptureResult::Captured{..}) {
            println!("  -> Pixel captured! Cooldown: 30s (600 ticks)");
            break;
        }
        
        world.tick();
    }
    
    // Test counter-attack after cooldown (simplified)
    println!("\n--- Counter-attack test (with cooldown) ---");
    let input = CaptureInput { pixel: attack_pixel, army_pct: 70 };
    let result = world.process_capture(Player::P2, input);
    println!("Counter-attack during cooldown: {:?}", result);
    
    // Test supply cutoff
    println!("\n--- Supply cutoff test ---");
    let mut world2 = World::new();
    world2.command_centers.insert(Player::P1, PixelCoord { x: 100, y: 100 });
    
    // Build connected territory from CC (including CC pixel)
    for x in 100..500 {
        let p = PixelCoord { x, y: 100 };
        world2.get(p).owner = Player::P1;
        world2.get(p).army_pct = 50;
        world2.get(p).supply_connected = true;
    }
    // Ensure CC itself is in pixels map
    world2.get(PixelCoord { x: 100, y: 100 }).owner = Player::P1;
    world2.get(PixelCoord { x: 100, y: 100 }).army_pct = 100;
    world2.get(PixelCoord { x: 100, y: 100 }).supply_connected = true;
    // Run ticks to update supply
    for _ in 0..5 { world2.tick(); }
    
    // Debug: check supply for various pixels
    println!("CC (100,100) supply: {}", world2.supply_check(Player::P1, PixelCoord { x: 100, y: 100 }));
    println!("Pixel (200,100) supply: {}", world2.supply_check(Player::P1, PixelCoord { x: 200, y: 100 }));
    println!("Pixel (499,100) supply: {}", world2.supply_check(Player::P1, PixelCoord { x: 499, y: 100 }));
    println!("Pixel (500,100) supply: {}", world2.supply_check(Player::P1, PixelCoord { x: 500, y: 100 }));
    println!("Pixel (501,100) supply: {}", world2.supply_check(Player::P1, PixelCoord { x: 501, y: 100 }));
    
    // Try to capture disconnected pixel
    let far_pixel = PixelCoord { x: 1000, y: 1000 };
    let result = world2.process_capture(Player::P1, CaptureInput { pixel: far_pixel, army_pct: 50 });
    println!("Capture far pixel (no supply): {:?}", result);
    
    // Capture (500,100) - adjacent to supply at (499,100)
    let adj_pixel = PixelCoord { x: 500, y: 100 };
    let result = world2.process_capture(Player::P1, CaptureInput { pixel: adj_pixel, army_pct: 50 });
    println!("Capture (500,100) adjacent to supply: {:?}", result);
    
    // Now (501,100) should have supply via (500,100)
    let next_pixel = PixelCoord { x: 501, y: 100 };
    let result = world2.process_capture(Player::P1, CaptureInput { pixel: next_pixel, army_pct: 50 });
    println!("Capture (501,100) now has supply via (500,100): {:?}", result);
}

fn benchmark() {
    let mut rng = StdRng::seed_from_u64(42);
    let mut world = World::new();
    
    // Populate 10k border pixels
    for x in 0..10_000 {
        let p = PixelCoord { x, y: 5000 };
        if x < 5000 {
            world.get(p).owner = Player::P1;
            world.get(p).army_pct = 50;
        } else {
            world.get(p).owner = Player::P2;
            world.get(p).army_pct = 50;
        }
        world.get(p).supply_connected = true;
    }
    
    let start = std::time::Instant::now();
    for _tick in 0..1000 {
        // Simulate 100 attacks per tick
        for _ in 0..100 {
            let x = rng.gen_range(4000..6000);
            let player = if rng.gen_bool(0.5) { Player::P1 } else { Player::P2 };
            let input = CaptureInput { pixel: PixelCoord { x, y: 5000 }, army_pct: rng.gen_range(30..80) };
            let _ = world.process_capture(player, input);
        }
        world.tick();
    }
    println!("\n1000 ticks × 100 attacks: {:?}", start.elapsed());
    println!("Pixels in world: {}", world.pixels.len());
}

fn main() {
    simulate_battle();
    // benchmark(); // skip for now
}