use crate::bot::Bot;
use crate::world::{Tile, World, CANVAS_H, CANVAS_W, H, TILE, W};

// 8-bit palette
const GRASS_A: [u8; 3] = [58, 128, 62];
const GRASS_B: [u8; 3] = [70, 146, 74];
const GRASS_DARK: [u8; 3] = [42, 104, 50];
const FOREST_BG: [u8; 3] = [34, 82, 40];
const FOREST_TRUNK: [u8; 3] = [90, 60, 30];
const FOREST_LEAF: [u8; 3] = [46, 112, 58];
const FOREST_LEAF_HL: [u8; 3] = [70, 146, 80];
const WATER_A: [u8; 3] = [46, 96, 168];
const WATER_B: [u8; 3] = [62, 118, 196];
const WATER_HL: [u8; 3] = [140, 192, 236];
const ROCK_A: [u8; 3] = [108, 112, 120];
const ROCK_B: [u8; 3] = [80, 84, 92];
const ROCK_HL: [u8; 3] = [148, 152, 160];
const SAND_A: [u8; 3] = [214, 182, 124];
const SAND_B: [u8; 3] = [194, 160, 104];
const BERRY: [u8; 3] = [214, 52, 64];
const BERRY_HL: [u8; 3] = [250, 128, 128];
const HOME_WALL: [u8; 3] = [136, 94, 56];
const HOME_ROOF: [u8; 3] = [90, 52, 28];
const HOME_DOOR: [u8; 3] = [58, 36, 22];
const SAPLING_TRUNK: [u8; 3] = [100, 72, 38];
const SAPLING_LEAF: [u8; 3] = [96, 176, 82];
const FLOWER_PINK: [u8; 3] = [234, 140, 190];
const FLOWER_YELLOW: [u8; 3] = [240, 214, 84];
const FLOWER_BLUE: [u8; 3] = [128, 164, 232];
const OUTLINE: [u8; 3] = [250, 250, 250];
const EYE: [u8; 3] = [20, 20, 20];
const WHITE: [u8; 3] = [245, 245, 245];
// Mature tree — denser, richer canopy than Forest.
const TREE_BG: [u8; 3] = [30, 70, 36];
const TREE_TRUNK: [u8; 3] = [82, 52, 26];
const TREE_LEAF_DARK: [u8; 3] = [36, 92, 48];
const TREE_LEAF: [u8; 3] = [54, 126, 64];
const TREE_LEAF_HL: [u8; 3] = [96, 172, 96];
// Mushroom — red cap with white dots on thin stem.
const MUSH_CAP: [u8; 3] = [206, 54, 60];
const MUSH_CAP_HL: [u8; 3] = [240, 120, 116];
const MUSH_DOT: [u8; 3] = [248, 244, 232];
const MUSH_STEM: [u8; 3] = [226, 220, 196];
// Campfire — pulsing warm palette.
const FIRE_LOG: [u8; 3] = [90, 60, 36];
const FIRE_CORE: [u8; 3] = [250, 240, 128];
const FIRE_MID: [u8; 3] = [248, 160, 70];
const FIRE_EDGE: [u8; 3] = [216, 72, 48];
const FIRE_SMOKE: [u8; 3] = [180, 180, 184];
// Log — felled tree drop. Warm brown with dark ring grain.
const LOG_A: [u8; 3] = [130, 88, 48];
const LOG_B: [u8; 3] = [92, 60, 32];
const LOG_HL: [u8; 3] = [170, 122, 72];
// Stone — chipped rock drop. Lighter than rock terrain.
const STONE_A: [u8; 3] = [170, 170, 176];
const STONE_B: [u8; 3] = [130, 132, 140];
const STONE_HL: [u8; 3] = [210, 212, 216];
// Cooked berry — darker, glossier than raw berry.
const COOKED_A: [u8; 3] = [200, 92, 44];
const COOKED_B: [u8; 3] = [152, 60, 28];
const COOKED_HL: [u8; 3] = [240, 180, 96];
// Path — worn grass. Slightly lighter than grass, with dirt streaks.
const PATH_A: [u8; 3] = [118, 108, 72];
const PATH_B: [u8; 3] = [96, 88, 58];
// Puddle — rain remnant. Darker than water, less animated.
const PUDDLE_A: [u8; 3] = [56, 96, 140];
const PUDDLE_B: [u8; 3] = [82, 130, 172];
const PUDDLE_HL: [u8; 3] = [170, 210, 232];
// Ash — burnt fire remains.
const ASH_A: [u8; 3] = [84, 82, 80];
const ASH_B: [u8; 3] = [60, 58, 56];
const ASH_HL: [u8; 3] = [184, 180, 172];
// Shrine — gathering monument. Pale stone + warm inset.
const SHRINE_BASE: [u8; 3] = [188, 184, 168];
const SHRINE_DARK: [u8; 3] = [132, 128, 116];
const SHRINE_GLOW: [u8; 3] = [248, 220, 132];
// Grave — memorial. Dark stone with pale top.
const GRAVE_STONE: [u8; 3] = [104, 100, 96];
const GRAVE_DARK: [u8; 3] = [64, 60, 56];
const GRAVE_CROSS: [u8; 3] = [208, 204, 196];

pub fn render_to_buffer(world: &World, buf: &mut [u8]) {
    // Terrain pass
    for ty in 0..H {
        for tx in 0..W {
            let t = Tile::from_u8(world.tiles[ty * W + tx]);
            draw_tile(buf, tx, ty, t, world.tick);
        }
    }
    // Bot pass
    for bot in &world.bots {
        if !bot.alive {
            continue;
        }
        let selected = world.selected_bot == Some(bot.id as usize);
        draw_bot(buf, bot, world.tick, selected);
    }
}

fn px(buf: &mut [u8], x: usize, y: usize, color: [u8; 3]) {
    if x >= CANVAS_W || y >= CANVAS_H {
        return;
    }
    let i = (y * CANVAS_W + x) * 4;
    buf[i] = color[0];
    buf[i + 1] = color[1];
    buf[i + 2] = color[2];
    buf[i + 3] = 255;
}

fn fill_tile(buf: &mut [u8], tx: usize, ty: usize, color: [u8; 3]) {
    let x0 = tx * TILE;
    let y0 = ty * TILE;
    for dy in 0..TILE {
        for dx in 0..TILE {
            px(buf, x0 + dx, y0 + dy, color);
        }
    }
}

fn draw_tile(buf: &mut [u8], tx: usize, ty: usize, t: Tile, tick: u64) {
    let h = tile_hash(tx as i32, ty as i32);
    match t {
        Tile::Grass => draw_grass(buf, tx, ty, h),
        Tile::Forest => draw_forest(buf, tx, ty, h),
        Tile::Water => draw_water(buf, tx, ty, tick),
        Tile::Rock => draw_rock(buf, tx, ty, h),
        Tile::Sand => draw_sand(buf, tx, ty, h),
        Tile::Berry => draw_berry(buf, tx, ty, h),
        Tile::Home => draw_home(buf, tx, ty),
        Tile::Sapling => draw_sapling(buf, tx, ty),
        Tile::Flower => draw_flower(buf, tx, ty, h),
        Tile::Tree => draw_tree(buf, tx, ty, h),
        Tile::Mushroom => draw_mushroom(buf, tx, ty, h),
        Tile::Fire => draw_fire(buf, tx, ty, tick),
        Tile::Log => draw_log(buf, tx, ty, h),
        Tile::Stone => draw_stone(buf, tx, ty, h),
        Tile::CookedBerry => draw_cooked_berry(buf, tx, ty, h),
        Tile::Path => draw_path(buf, tx, ty, h),
        Tile::Puddle => draw_puddle(buf, tx, ty, tick),
        Tile::Ash => draw_ash(buf, tx, ty, h),
        Tile::Shrine => draw_shrine(buf, tx, ty, tick),
        Tile::Grave => draw_grave(buf, tx, ty),
        Tile::Field => draw_field(buf, tx, ty, h),
        Tile::Fish => draw_fish_tile(buf, tx, ty, h),
        Tile::CookedFish => draw_cooked_fish(buf, tx, ty, h),
    }
}

fn tile_hash(x: i32, y: i32) -> u32 {
    let a = (x as u32).wrapping_mul(374761393);
    let b = (y as u32).wrapping_mul(668265263);
    (a ^ b).wrapping_mul(1274126177)
}

fn draw_grass(buf: &mut [u8], tx: usize, ty: usize, h: u32) {
    let base = if (tx + ty) % 2 == 0 { GRASS_A } else { GRASS_B };
    fill_tile(buf, tx, ty, base);
    // Scatter darker blades
    let x0 = tx * TILE;
    let y0 = ty * TILE;
    for i in 0..3 {
        let hh = h.wrapping_mul(1 + i as u32).wrapping_add(0x9E3779B1);
        let dx = (hh % 8) as usize;
        let dy = ((hh >> 8) % 8) as usize;
        px(buf, x0 + dx, y0 + dy, GRASS_DARK);
    }
}

fn draw_forest(buf: &mut [u8], tx: usize, ty: usize, h: u32) {
    fill_tile(buf, tx, ty, FOREST_BG);
    let x0 = tx * TILE;
    let y0 = ty * TILE;
    // Canopy circle-ish
    let canopy: [(usize, usize); 12] = [
        (1, 2), (2, 1), (3, 1), (4, 1), (5, 2),
        (1, 3), (2, 3), (3, 3), (4, 3), (5, 3),
        (2, 4), (4, 4),
    ];
    for (dx, dy) in canopy {
        px(buf, x0 + dx, y0 + dy, FOREST_LEAF);
    }
    // Highlights — vary by hash
    let hl = [
        ((h >> 2) % 5) as usize + 1,
        ((h >> 4) % 3) as usize + 1,
    ];
    px(buf, x0 + hl[0], y0 + hl[1], FOREST_LEAF_HL);
    // Trunk
    px(buf, x0 + 3, y0 + 5, FOREST_TRUNK);
    px(buf, x0 + 3, y0 + 6, FOREST_TRUNK);
    px(buf, x0 + 4, y0 + 5, FOREST_TRUNK);
    px(buf, x0 + 4, y0 + 6, FOREST_TRUNK);
}

fn draw_water(buf: &mut [u8], tx: usize, ty: usize, tick: u64) {
    let x0 = tx * TILE;
    let y0 = ty * TILE;
    // Per-tile phase offset for organic variety
    let phase = (tx * 7 + ty * 13) as u64;

    for dy in 0..TILE {
        for dx in 0..TILE {
            // Slow undulating blend between two blues using a sine-ish LUT.
            // Different rows shift at different rates for a ripple look.
            let wave_input = (tick.wrapping_add(phase)
                .wrapping_add((dy as u64) * 11)
                .wrapping_add((dx as u64) * 5)) / 12;
            // Simple triangle-wave approximation (no std sin in wasm)
            let t_mod = (wave_input % 40) as i32; // 0..39
            let tri = if t_mod < 20 { t_mod } else { 40 - t_mod }; // 0..20
            let blend = tri as f32 / 20.0; // 0.0..1.0

            let r = (WATER_A[0] as f32 * (1.0 - blend) + WATER_B[0] as f32 * blend) as u8;
            let g = (WATER_A[1] as f32 * (1.0 - blend) + WATER_B[1] as f32 * blend) as u8;
            let b = (WATER_A[2] as f32 * (1.0 - blend) + WATER_B[2] as f32 * blend) as u8;
            px(buf, x0 + dx, y0 + dy, [r, g, b]);
        }
    }

    // Two soft highlight specks that drift at different speeds
    let h1_x = ((tick.wrapping_add(phase) / 18) % 7) as usize + 1;
    let h1_y = ((tick.wrapping_add(phase * 3) / 28) % 5) as usize + 1;
    let h2_x = ((tick.wrapping_add(phase * 2 + 20) / 24) % 6) as usize;
    let h2_y = ((tick.wrapping_add(phase + 35) / 32) % 6) as usize + 1;

    // Softer highlight — blend toward WATER_HL rather than full white
    let soft_hl = [
        ((WATER_B[0] as u16 + WATER_HL[0] as u16) / 2) as u8,
        ((WATER_B[1] as u16 + WATER_HL[1] as u16) / 2) as u8,
        ((WATER_B[2] as u16 + WATER_HL[2] as u16) / 2) as u8,
    ];
    px(buf, x0 + h1_x, y0 + h1_y, soft_hl);
    px(buf, x0 + h2_x, y0 + h2_y, WATER_HL);
}

fn draw_rock(buf: &mut [u8], tx: usize, ty: usize, h: u32) {
    fill_tile(buf, tx, ty, ROCK_A);
    let x0 = tx * TILE;
    let y0 = ty * TILE;
    // Darker lower-right
    for dy in 5..TILE {
        for dx in 5..TILE {
            px(buf, x0 + dx, y0 + dy, ROCK_B);
        }
    }
    // Highlight speck
    let hx = (h % 6) as usize + 1;
    let hy = ((h >> 4) % 4) as usize + 1;
    px(buf, x0 + hx, y0 + hy, ROCK_HL);
    // Crack
    for dy in 2..6 {
        px(buf, x0 + 4, y0 + dy, ROCK_B);
    }
}

fn draw_sand(buf: &mut [u8], tx: usize, ty: usize, h: u32) {
    fill_tile(buf, tx, ty, SAND_A);
    let x0 = tx * TILE;
    let y0 = ty * TILE;
    for i in 0..4 {
        let hh = h.wrapping_mul(3 + i);
        let dx = (hh % 8) as usize;
        let dy = ((hh >> 9) % 8) as usize;
        px(buf, x0 + dx, y0 + dy, SAND_B);
    }
}

fn draw_berry(buf: &mut [u8], tx: usize, ty: usize, h: u32) {
    // Grass underneath
    draw_grass(buf, tx, ty, h.wrapping_add(7));
    let x0 = tx * TILE;
    let y0 = ty * TILE;
    // Bush dark-green base
    for (dx, dy) in [(2, 3), (3, 3), (4, 3), (3, 4), (4, 4), (2, 4), (5, 3)] {
        px(buf, x0 + dx, y0 + dy, FOREST_LEAF);
    }
    // Red berries
    for (dx, dy) in [(3, 3), (4, 4), (2, 4)] {
        px(buf, x0 + dx, y0 + dy, BERRY);
    }
    px(buf, x0 + 3, y0 + 3, BERRY_HL);
}

fn draw_home(buf: &mut [u8], tx: usize, ty: usize) {
    fill_tile(buf, tx, ty, GRASS_A);
    let x0 = tx * TILE;
    let y0 = ty * TILE;
    // Walls
    for dy in 3..7 {
        for dx in 1..7 {
            px(buf, x0 + dx, y0 + dy, HOME_WALL);
        }
    }
    // Roof
    for dx in 0..8 {
        px(buf, x0 + dx, y0 + 2, HOME_ROOF);
    }
    for dx in 1..7 {
        px(buf, x0 + dx, y0 + 1, HOME_ROOF);
    }
    for dx in 2..6 {
        px(buf, x0 + dx, y0, HOME_ROOF);
    }
    // Door
    for dy in 4..7 {
        px(buf, x0 + 3, y0 + dy, HOME_DOOR);
        px(buf, x0 + 4, y0 + dy, HOME_DOOR);
    }
    // Window
    px(buf, x0 + 1, y0 + 3, WATER_HL);
    px(buf, x0 + 6, y0 + 3, WATER_HL);
}

fn draw_sapling(buf: &mut [u8], tx: usize, ty: usize) {
    draw_grass(buf, tx, ty, tile_hash(tx as i32, ty as i32));
    let x0 = tx * TILE;
    let y0 = ty * TILE;
    // Trunk
    px(buf, x0 + 3, y0 + 5, SAPLING_TRUNK);
    px(buf, x0 + 3, y0 + 6, SAPLING_TRUNK);
    // Leaves
    px(buf, x0 + 2, y0 + 4, SAPLING_LEAF);
    px(buf, x0 + 3, y0 + 4, SAPLING_LEAF);
    px(buf, x0 + 4, y0 + 4, SAPLING_LEAF);
    px(buf, x0 + 3, y0 + 3, SAPLING_LEAF);
}

fn draw_flower(buf: &mut [u8], tx: usize, ty: usize, h: u32) {
    draw_grass(buf, tx, ty, h);
    let x0 = tx * TILE;
    let y0 = ty * TILE;
    let color = match h % 3 {
        0 => FLOWER_PINK,
        1 => FLOWER_YELLOW,
        _ => FLOWER_BLUE,
    };
    let dx = (h % 5) as usize + 1;
    let dy = ((h >> 5) % 5) as usize + 1;
    px(buf, x0 + dx, y0 + dy, color);
}

fn draw_tree(buf: &mut [u8], tx: usize, ty: usize, h: u32) {
    fill_tile(buf, tx, ty, TREE_BG);
    let x0 = tx * TILE;
    let y0 = ty * TILE;
    // Full, rounded canopy filling 6x5 — bigger than Forest's sparse cluster.
    let canopy: [(usize, usize); 22] = [
                (2, 0), (3, 0), (4, 0), (5, 0),
        (1, 1), (2, 1), (3, 1), (4, 1), (5, 1), (6, 1),
        (1, 2), (2, 2), (3, 2), (4, 2), (5, 2), (6, 2),
        (1, 3), (2, 3), (3, 3), (4, 3), (5, 3), (6, 3),
    ];
    for (dx, dy) in canopy {
        px(buf, x0 + dx, y0 + dy, TREE_LEAF);
    }
    // Darker speckle for depth
    for i in 0..4 {
        let hh = h.wrapping_mul(7 + i as u32).wrapping_add(0xA5A53931);
        let dx = (hh % 6) as usize + 1;
        let dy = ((hh >> 4) % 4) as usize;
        px(buf, x0 + dx, y0 + dy, TREE_LEAF_DARK);
    }
    // Bright highlight — a single shaft of sun on the crown
    let hlx = ((h >> 2) % 4) as usize + 2;
    px(buf, x0 + hlx, y0 + 1, TREE_LEAF_HL);
    px(buf, x0 + hlx + 1, y0 + 1, TREE_LEAF_HL);
    // Trunk — thicker than Forest's
    for dy in 4..8 {
        px(buf, x0 + 3, y0 + dy, TREE_TRUNK);
        px(buf, x0 + 4, y0 + dy, TREE_TRUNK);
    }
    // Root hint
    px(buf, x0 + 2, y0 + 7, TREE_TRUNK);
    px(buf, x0 + 5, y0 + 7, TREE_TRUNK);
}

fn draw_mushroom(buf: &mut [u8], tx: usize, ty: usize, h: u32) {
    draw_grass(buf, tx, ty, h.wrapping_add(3));
    let x0 = tx * TILE;
    let y0 = ty * TILE;
    // Stem (2 tall)
    px(buf, x0 + 3, y0 + 5, MUSH_STEM);
    px(buf, x0 + 4, y0 + 5, MUSH_STEM);
    px(buf, x0 + 3, y0 + 6, MUSH_STEM);
    px(buf, x0 + 4, y0 + 6, MUSH_STEM);
    // Cap — domed
    for dx in 2..6 {
        px(buf, x0 + dx, y0 + 4, MUSH_CAP);
    }
    for dx in 1..7 {
        px(buf, x0 + dx, y0 + 3, MUSH_CAP);
    }
    for dx in 2..6 {
        px(buf, x0 + dx, y0 + 2, MUSH_CAP);
    }
    // Cap highlight
    px(buf, x0 + 3, y0 + 2, MUSH_CAP_HL);
    px(buf, x0 + 2, y0 + 3, MUSH_CAP_HL);
    // Two white dots — classic
    let d1 = (h % 3) as usize + 1;
    let d2 = ((h >> 4) % 3) as usize + 3;
    px(buf, x0 + d1 + 1, y0 + 3, MUSH_DOT);
    px(buf, x0 + d2 + 1, y0 + 3, MUSH_DOT);
}

fn draw_fire(buf: &mut [u8], tx: usize, ty: usize, tick: u64) {
    // Grass base
    fill_tile(buf, tx, ty, GRASS_A);
    let x0 = tx * TILE;
    let y0 = ty * TILE;
    // Three-phase flicker
    let phase = (tick / 6) as usize % 3;
    // Logs (cross-shape at the bottom)
    for dx in 1..7 {
        px(buf, x0 + dx, y0 + 6, FIRE_LOG);
    }
    px(buf, x0 + 2, y0 + 7, FIRE_LOG);
    px(buf, x0 + 5, y0 + 7, FIRE_LOG);
    // Edge flames
    let base_y = match phase {
        0 => 4,
        1 => 3,
        _ => 4,
    };
    for dx in 2..6 {
        px(buf, x0 + dx, y0 + 5, FIRE_EDGE);
    }
    for dx in 2..6 {
        px(buf, x0 + dx, y0 + base_y, FIRE_MID);
    }
    // Core bright
    px(buf, x0 + 3, y0 + base_y, FIRE_CORE);
    px(buf, x0 + 4, y0 + base_y, FIRE_CORE);
    px(buf, x0 + 3, y0 + (base_y - 1), FIRE_CORE);
    // Rising tip (different each phase)
    let tip_y = if phase == 1 { 1 } else { 2 };
    px(buf, x0 + 3 + (phase & 1), y0 + tip_y, FIRE_MID);
    // Smoke wisp
    if phase == 2 {
        px(buf, x0 + 4, y0, FIRE_SMOKE);
    } else if phase == 0 {
        px(buf, x0 + 3, y0, FIRE_SMOKE);
    }
}

fn draw_log(buf: &mut [u8], tx: usize, ty: usize, h: u32) {
    draw_grass(buf, tx, ty, h.wrapping_add(11));
    let x0 = tx * TILE;
    let y0 = ty * TILE;
    // Horizontal log, 6 wide, 3 tall, centred vertically.
    for dx in 1..7 {
        px(buf, x0 + dx, y0 + 3, LOG_A);
        px(buf, x0 + dx, y0 + 4, LOG_A);
        px(buf, x0 + dx, y0 + 5, LOG_B);
    }
    // End caps — dark ring rings
    px(buf, x0 + 1, y0 + 4, LOG_B);
    px(buf, x0 + 6, y0 + 4, LOG_B);
    // Highlight streak
    for dx in 2..6 {
        px(buf, x0 + dx, y0 + 3, LOG_HL);
    }
    // Bark flecks
    let hx = (h % 4) as usize + 2;
    px(buf, x0 + hx, y0 + 5, LOG_B);
}

fn draw_stone(buf: &mut [u8], tx: usize, ty: usize, h: u32) {
    draw_grass(buf, tx, ty, h.wrapping_add(23));
    let x0 = tx * TILE;
    let y0 = ty * TILE;
    // Small lumpy pile, 4-wide and 3-tall
    let shape: [(usize, usize); 10] = [
        (2, 4), (3, 4), (4, 4), (5, 4),
        (3, 3), (4, 3), (5, 3),
        (2, 5), (3, 5), (4, 5),
    ];
    for (dx, dy) in shape {
        px(buf, x0 + dx, y0 + dy, STONE_A);
    }
    // Dark underside
    px(buf, x0 + 2, y0 + 5, STONE_B);
    px(buf, x0 + 5, y0 + 4, STONE_B);
    // Highlight speck
    let hx = (h % 3) as usize + 3;
    px(buf, x0 + hx, y0 + 3, STONE_HL);
}

fn draw_cooked_berry(buf: &mut [u8], tx: usize, ty: usize, h: u32) {
    draw_grass(buf, tx, ty, h.wrapping_add(31));
    let x0 = tx * TILE;
    let y0 = ty * TILE;
    // Similar shape to berry but darker, glossier
    for (dx, dy) in [(3, 3), (4, 3), (3, 4), (4, 4), (2, 4), (5, 4), (3, 5), (4, 5)] {
        px(buf, x0 + dx, y0 + dy, COOKED_A);
    }
    // Darker underside
    for (dx, dy) in [(3, 5), (4, 5)] {
        px(buf, x0 + dx, y0 + dy, COOKED_B);
    }
    // Gloss spot — indicates cookedness
    px(buf, x0 + 3, y0 + 3, COOKED_HL);
    // Small steam wisps
    if (tx + ty + (h as usize)) & 1 == 0 {
        px(buf, x0 + 3, y0 + 1, [200, 200, 210]);
    }
}

fn draw_path(buf: &mut [u8], tx: usize, ty: usize, h: u32) {
    // Worn patch — dirt tones over a faded grass base.
    let base = if (tx + ty) % 2 == 0 { PATH_A } else { PATH_B };
    fill_tile(buf, tx, ty, base);
    let x0 = tx * TILE;
    let y0 = ty * TILE;
    // A scattering of trodden-bare flecks
    for i in 0..3 {
        let hh = h.wrapping_mul(5 + i as u32).wrapping_add(0xDEADBEEF);
        let dx = (hh % 8) as usize;
        let dy = ((hh >> 8) % 8) as usize;
        px(buf, x0 + dx, y0 + dy, PATH_B);
    }
    // A hint of remembered grass at the edges
    px(buf, x0, y0, GRASS_DARK);
    px(buf, x0 + 7, y0 + 7, GRASS_DARK);
}

fn draw_puddle(buf: &mut [u8], tx: usize, ty: usize, tick: u64) {
    // Grass base showing at the edges
    fill_tile(buf, tx, ty, GRASS_A);
    let x0 = tx * TILE;
    let y0 = ty * TILE;
    // Oval puddle in middle of tile (6x4)
    let shape: [(usize, usize); 18] = [
                (2, 2), (3, 2), (4, 2), (5, 2),
        (1, 3), (2, 3), (3, 3), (4, 3), (5, 3), (6, 3),
        (1, 4), (2, 4), (3, 4), (4, 4), (5, 4), (6, 4),
                (2, 5), (4, 5),
    ];
    for (dx, dy) in shape {
        px(buf, x0 + dx, y0 + dy, PUDDLE_A);
    }
    for dx in 2..6 {
        px(buf, x0 + dx, y0 + 3, PUDDLE_B);
    }
    // Ripple — shifting highlight
    let shift = ((tick / 18) as usize + tx + ty) % 5;
    px(buf, x0 + 2 + shift % 4, y0 + 3, PUDDLE_HL);
}

fn draw_ash(buf: &mut [u8], tx: usize, ty: usize, h: u32) {
    let base = if (tx + ty) % 2 == 0 { ASH_A } else { ASH_B };
    fill_tile(buf, tx, ty, base);
    let x0 = tx * TILE;
    let y0 = ty * TILE;
    // Charred log fragments
    for dx in 2..6 {
        px(buf, x0 + dx, y0 + 5, ASH_B);
    }
    px(buf, x0 + 2, y0 + 6, ASH_B);
    px(buf, x0 + 5, y0 + 6, ASH_B);
    // Grey embers
    let hx = (h % 4) as usize + 2;
    let hy = ((h >> 3) % 3) as usize + 3;
    px(buf, x0 + hx, y0 + hy, ASH_HL);
    // A faint smoke trace
    px(buf, x0 + 3, y0 + 1, [160, 160, 164]);
}

fn draw_shrine(buf: &mut [u8], tx: usize, ty: usize, tick: u64) {
    fill_tile(buf, tx, ty, GRASS_A);
    let x0 = tx * TILE;
    let y0 = ty * TILE;
    // Base: square plinth (4 wide x 3 tall)
    for dy in 4..7 {
        for dx in 2..6 {
            px(buf, x0 + dx, y0 + dy, SHRINE_BASE);
        }
    }
    // Dark band
    for dx in 2..6 {
        px(buf, x0 + dx, y0 + 6, SHRINE_DARK);
    }
    // Upper column (2 wide x 2 tall)
    for dy in 2..4 {
        for dx in 3..5 {
            px(buf, x0 + dx, y0 + dy, SHRINE_BASE);
        }
    }
    // Pulsing glow on top — marks gathering spot
    let phase = (tick / 20) as usize % 3;
    let glow_bright = match phase {
        0 => SHRINE_GLOW,
        1 => [230, 200, 110],
        _ => [210, 180, 98],
    };
    px(buf, x0 + 3, y0 + 1, glow_bright);
    px(buf, x0 + 4, y0 + 1, glow_bright);
}

fn draw_grave(buf: &mut [u8], tx: usize, ty: usize) {
    fill_tile(buf, tx, ty, GRASS_A);
    let x0 = tx * TILE;
    let y0 = ty * TILE;
    // A small patch of disturbed soil
    for dx in 1..7 {
        px(buf, x0 + dx, y0 + 6, [74, 58, 44]);
    }
    // Tombstone — 3 wide x 4 tall
    for dy in 2..6 {
        for dx in 3..5 {
            px(buf, x0 + dx, y0 + dy, GRAVE_STONE);
        }
    }
    // Rounded top
    px(buf, x0 + 3, y0 + 1, GRAVE_STONE);
    px(buf, x0 + 4, y0 + 1, GRAVE_STONE);
    // Dark edge
    px(buf, x0 + 2, y0 + 5, GRAVE_DARK);
    px(buf, x0 + 5, y0 + 5, GRAVE_DARK);
    px(buf, x0 + 2, y0 + 4, GRAVE_DARK);
    // Small cross etched in pale
    px(buf, x0 + 3, y0 + 3, GRAVE_CROSS);
    px(buf, x0 + 4, y0 + 3, GRAVE_CROSS);
    px(buf, x0 + 3, y0 + 2, GRAVE_CROSS);
    px(buf, x0 + 3, y0 + 4, GRAVE_CROSS);
}

// Field — tilled farmland. Warm brown-green base with small dot pattern.
const FIELD_A: [u8; 3] = [140, 120, 60];
const FIELD_B: [u8; 3] = [112, 96, 44];
const FIELD_DOT: [u8; 3] = [96, 80, 36];

fn draw_field(buf: &mut [u8], tx: usize, ty: usize, h: u32) {
    let base = if (tx + ty) % 2 == 0 { FIELD_A } else { FIELD_B };
    fill_tile(buf, tx, ty, base);
    let x0 = tx * TILE;
    let y0 = ty * TILE;
    // Small furrow dots scattered across the tile
    for i in 0..4u32 {
        let hh = h.wrapping_mul(13 + i).wrapping_add(0xCAFEBABE);
        let dx = (hh % 7) as usize + 1;
        let dy = ((hh >> 8) % 7) as usize + 1;
        px(buf, x0 + dx, y0 + dy, FIELD_DOT);
    }
}

// Fish tile — a caught fish on grass. Silvery blue with slight shine.
const FISH_A: [u8; 3] = [100, 160, 210];
const FISH_B: [u8; 3] = [70, 120, 170];
const FISH_HL: [u8; 3] = [180, 220, 240];
// Cooked fish — warm golden tint, same shape as raw fish.
const CFISH_A: [u8; 3] = [210, 160, 70];
const CFISH_B: [u8; 3] = [180, 130, 50];
const CFISH_HL: [u8; 3] = [240, 200, 120];

fn draw_fish_tile(buf: &mut [u8], tx: usize, ty: usize, h: u32) {
    draw_grass(buf, tx, ty, h.wrapping_add(41));
    let x0 = tx * TILE;
    let y0 = ty * TILE;
    // Fish body — horizontal oval shape
    for (dx, dy) in [(2, 4), (3, 4), (4, 4), (5, 4), (2, 3), (3, 3), (4, 3), (5, 3)] {
        px(buf, x0 + dx, y0 + dy, FISH_A);
    }
    // Tail fin
    px(buf, x0 + 1, y0 + 3, FISH_B);
    px(buf, x0 + 1, y0 + 4, FISH_B);
    // Highlight shimmer
    px(buf, x0 + 3, y0 + 3, FISH_HL);
    px(buf, x0 + 4, y0 + 3, FISH_HL);
}

fn draw_cooked_fish(buf: &mut [u8], tx: usize, ty: usize, h: u32) {
    draw_grass(buf, tx, ty, h.wrapping_add(53));
    let x0 = tx * TILE;
    let y0 = ty * TILE;
    // Same shape as raw fish but with warm golden palette
    for (dx, dy) in [(2, 4), (3, 4), (4, 4), (5, 4), (2, 3), (3, 3), (4, 3), (5, 3)] {
        px(buf, x0 + dx, y0 + dy, CFISH_A);
    }
    // Tail fin
    px(buf, x0 + 1, y0 + 3, CFISH_B);
    px(buf, x0 + 1, y0 + 4, CFISH_B);
    // Highlight shimmer
    px(buf, x0 + 3, y0 + 3, CFISH_HL);
    px(buf, x0 + 4, y0 + 3, CFISH_HL);
}

// Skin tones for variety — selected per bot from their id.
const SKIN: [[u8; 3]; 4] = [
    [240, 210, 180], // light
    [210, 170, 130], // medium
    [170, 120, 80],  // tan
    [120, 80, 50],   // dark
];

fn draw_bot(buf: &mut [u8], bot: &Bot, tick: u64, selected: bool) {
    let x0 = (bot.visual_x * TILE as f32).round() as usize;
    let y0 = (bot.visual_y * TILE as f32).round() as usize;
    let shirt = bot.color;
    let shirt_dark = [
        (shirt[0] as u32 * 2 / 3) as u8,
        (shirt[1] as u32 * 2 / 3) as u8,
        (shirt[2] as u32 * 2 / 3) as u8,
    ];
    let skin = SKIN[(bot.id as usize) % SKIN.len()];
    let skin_dark = [
        (skin[0] as u32 * 3 / 4) as u8,
        (skin[1] as u32 * 3 / 4) as u8,
        (skin[2] as u32 * 3 / 4) as u8,
    ];
    let pants = [60, 60, 90]; // dark trousers
    let shoe = [40, 35, 30];

    // Walking animation: alternate leg positions based on age (changes when moving).
    // Two frames: 0 = left-forward, 1 = right-forward
    let walk_frame = (bot.age / 8) % 2 == 0;
    let standing = bot.goal == crate::bot::Goal::Rest
        || bot.chatting_with.is_some()
        || bot.commitment_delay > 0;

    // ── Head (rows 0-1): 4px wide, centered, with hair ──
    //    . . H H H H . .   (H = hair / shirt-color hat)
    //    . . S E S E . .   (S = skin, E = eyes)
    let hair = shirt_dark;
    px(buf, x0 + 2, y0, hair);
    px(buf, x0 + 3, y0, hair);
    px(buf, x0 + 4, y0, hair);
    px(buf, x0 + 5, y0, hair);

    // Face row — skin + eyes
    let blink = (tick + bot.id as u64) % 120 < 4;
    let (fx, _fy) = bot.facing;
    // Shift eyes based on facing direction
    let eye_offset = fx.clamp(-1, 1);
    px(buf, x0 + 2, y0 + 1, skin);
    px(buf, x0 + 3, y0 + 1, skin);
    px(buf, x0 + 4, y0 + 1, skin);
    px(buf, x0 + 5, y0 + 1, skin);
    if !blink {
        let e1 = (3 + eye_offset).clamp(2, 4) as usize;
        let e2 = (4 + eye_offset).clamp(3, 5) as usize;
        px(buf, x0 + e1, y0 + 1, EYE);
        px(buf, x0 + e2, y0 + 1, EYE);
    }

    // ── Torso (rows 2-4): shirt color ──
    //    . . C C C C . .   (shirt, 4 wide)
    //    . A C C C C A .   (A = arm = skin)
    //    . . C C C C . .
    for dy in 2..5 {
        for dx in 2..6 {
            px(buf, x0 + dx, y0 + dy, shirt);
        }
    }
    // Arms (row 3) — skin tone reaching out
    px(buf, x0 + 1, y0 + 3, skin_dark);
    px(buf, x0 + 6, y0 + 3, skin_dark);

    // Shirt collar/highlight
    px(buf, x0 + 3, y0 + 2, [shirt[0].saturating_add(30), shirt[1].saturating_add(30), shirt[2].saturating_add(30)]);
    px(buf, x0 + 4, y0 + 2, [shirt[0].saturating_add(30), shirt[1].saturating_add(30), shirt[2].saturating_add(30)]);

    // ── Legs (rows 5-6): dark trousers ──
    // Walking animation: legs alternate positions
    if standing {
        // Standing: legs together
        px(buf, x0 + 3, y0 + 5, pants);
        px(buf, x0 + 4, y0 + 5, pants);
        px(buf, x0 + 3, y0 + 6, shoe);
        px(buf, x0 + 4, y0 + 6, shoe);
    } else if walk_frame {
        // Frame A: left leg forward, right leg back
        px(buf, x0 + 2, y0 + 5, pants);
        px(buf, x0 + 5, y0 + 5, pants);
        px(buf, x0 + 2, y0 + 6, shoe);
        px(buf, x0 + 5, y0 + 6, shoe);
    } else {
        // Frame B: right leg forward, left leg back
        px(buf, x0 + 3, y0 + 5, pants);
        px(buf, x0 + 4, y0 + 5, pants);
        px(buf, x0 + 3, y0 + 6, shoe);
        px(buf, x0 + 4, y0 + 6, shoe);
    }

    // ── Carry indicator: small colored bundle held above head ──
    if bot.carrying != crate::bot::Carry::None {
        let cc = bot.carrying.color();
        // Small bundle on shoulder (top-right of torso)
        px(buf, x0 + 5, y0 + 2, cc);
        px(buf, x0 + 6, y0 + 2, cc);
    }

    // ── Mood: rosy cheeks when happy ──
    if bot.mood > 20.0 {
        px(buf, x0 + 2, y0 + 1, [220, 160, 160]);
        px(buf, x0 + 5, y0 + 1, [220, 160, 160]);
    }

    // ── Selection outline ──
    if selected {
        for dy in 0..8 {
            px(buf, x0, y0 + dy, OUTLINE);
            px(buf, x0 + 7, y0 + dy, OUTLINE);
        }
        for dx in 0..8 {
            px(buf, x0 + dx, y0, OUTLINE);
            px(buf, x0 + dx, y0 + 7, OUTLINE);
        }
    }

    // ── Sleeping Z above head ──
    if bot.goal == crate::bot::Goal::Rest && bot.energy < 50.0 {
        if bot.visual_y > 0.5 {
            let zx = x0 + 6;
            let zy = ((bot.visual_y - 1.0) * TILE as f32).round() as usize + 5;
            px(buf, zx, zy, WHITE);
            px(buf, zx - 1, zy, WHITE);
            px(buf, zx, zy + 1, WHITE);
        }
    }
}

fn goal_color(g: crate::bot::Goal) -> [u8; 3] {
    use crate::bot::Goal::*;
    match g {
        Eat => [230, 80, 80],
        Rest => [110, 110, 200],
        Socialize => [230, 140, 200],
        Explore => [140, 220, 240],
        Forage => [90, 170, 90],
        Build => [230, 180, 90],
        Flee => [255, 80, 80],
        Visit => [180, 200, 80],
        Craft => [200, 160, 110],
        Chop => [210, 130, 60],
        Drink => [90, 180, 220],
        Cook => [240, 150, 70],
        Gather => [170, 180, 80],
        Deliver => [180, 140, 220],
        Warm => [250, 170, 100],
        Mourn => [140, 140, 160],
        Heal => [180, 220, 160],
        Idle => [180, 180, 180],
        Fish => [80, 180, 200],
        Farm => [160, 200, 80],
    }
}
