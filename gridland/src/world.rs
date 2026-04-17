use crate::bot::Bot;
use crate::rng::Rng;
use std::collections::HashMap;

pub const W: usize = 64;
pub const H: usize = 64;
pub const TILE: usize = 8; // pixels per tile
pub const CANVAS_W: usize = W * TILE;
pub const CANVAS_H: usize = H * TILE;

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Tile {
    Grass = 0,
    Forest = 1,
    Water = 2,
    Rock = 3,
    Sand = 4,
    Berry = 5,        // edible fruit on grass
    Home = 6,         // bot-built shelter
    Sapling = 7,
    Flower = 8,       // decorative, on grass
    Tree = 9,         // mature tree — grown from Forest; drops berries nearby
    Mushroom = 10,    // cool-damp spot food — gives an energy jolt, not hunger cure
    Fire = 11,        // campfire; social attractor; now consumes Log fuel
    // --- new tiles (the crazy expansion) ---
    Log = 12,         // felled-tree drop; burnable fuel; haulable material
    Stone = 13,       // chipped-rock drop; haulable crafting material
    CookedBerry = 14, // upgraded berry (berry + cook near fire); keeps longer, tastier
    Path = 15,        // worn grass trail; cheaper movement, subtle visual aging
    Puddle = 16,      // rain-made water; drinkable; evaporates
    Ash = 17,         // burnt-out fire; fertilizes neighbours, decays to grass
    Shrine = 18,      // socialite waypoint; gathering bonus when bots cluster
    Grave = 19,       // dead bot remnant; neighbours mourn; decays to grass
    Field = 20,       // farmer-tilled plot; periodically spawns berries; trees won't grow
    Fish = 21,        // raw caught fish; must be cooked at a fire before eating
    CookedFish = 22,  // cooked fish; the richest food — big hunger+mood+energy
}

/// Nutritional properties of a food tile. Returned by `Tile::food_props()`.
/// Adding a new food type means adding one entry here — everything else
/// in the simulation queries these properties instead of matching tile variants.
pub struct FoodProps {
    pub hunger_relief: f32,
    pub energy_gain: f32,
    pub mood_boost: f32,
    pub stress_relief: f32,
    /// If true, the tile is consumed (replaced with Grass) when eaten.
    pub consumed: bool,
    /// Minimum hunger level before the bot will eat this food.
    /// 0 means always eat when found.
    pub hunger_threshold: f32,
}

impl Tile {
    pub fn from_u8(v: u8) -> Tile {
        match v {
            1 => Tile::Forest,
            2 => Tile::Water,
            3 => Tile::Rock,
            4 => Tile::Sand,
            5 => Tile::Berry,
            6 => Tile::Home,
            7 => Tile::Sapling,
            8 => Tile::Flower,
            9 => Tile::Tree,
            10 => Tile::Mushroom,
            11 => Tile::Fire,
            12 => Tile::Log,
            13 => Tile::Stone,
            14 => Tile::CookedBerry,
            15 => Tile::Path,
            16 => Tile::Puddle,
            17 => Tile::Ash,
            18 => Tile::Shrine,
            19 => Tile::Grave,
            20 => Tile::Field,
            21 => Tile::Fish,
            22 => Tile::CookedFish,
            _ => Tile::Grass,
        }
    }

    /// Can a bot stand on this tile?
    pub fn walkable(self) -> bool {
        !matches!(self, Tile::Water | Tile::Rock | Tile::Tree)
    }

    /// Is it something a bot would eat off the ground?
    pub fn is_food(self) -> bool {
        // Raw fish is NOT food — it must be cooked at a fire first.
        matches!(self, Tile::Berry | Tile::Mushroom | Tile::CookedBerry | Tile::CookedFish)
    }

    /// Drinkable — satisfies thirst. Water tiles aren't walkable but bots
    /// can drink from the edge. Puddles are both walkable and drinkable.
    pub fn is_drinkable(self) -> bool {
        matches!(self, Tile::Water | Tile::Puddle)
    }

    /// A resource that can be picked up and carried.
    pub fn is_haulable(self) -> bool {
        matches!(self, Tile::Log | Tile::Stone | Tile::Berry | Tile::CookedBerry | Tile::Mushroom | Tile::Fish | Tile::CookedFish)
    }

    /// Reducing terrain friction: paths are faster, sand slower.
    pub fn move_cost_bonus(self) -> i32 {
        match self {
            Tile::Path => 2,
            Tile::Home => 1,
            Tile::Sand => -1,
            Tile::Shrine => 1,
            _ => 0,
        }
    }

    /// Nutritional properties for edible tiles. Returns `None` for tiles that
    /// aren't food (including raw Fish, which must be cooked first, and Fire,
    /// whose ambient warmth effects are handled separately).
    pub fn food_props(self) -> Option<FoodProps> {
        match self {
            Tile::Berry => Some(FoodProps {
                hunger_relief: 55.0,
                energy_gain: 0.0,
                mood_boost: 10.0,
                stress_relief: 0.0,
                consumed: true,
                hunger_threshold: 15.0,
            }),
            Tile::CookedBerry => Some(FoodProps {
                hunger_relief: 70.0,
                energy_gain: 0.0,
                mood_boost: 18.0,
                stress_relief: 8.0,
                consumed: true,
                hunger_threshold: 10.0,
            }),
            Tile::Mushroom => Some(FoodProps {
                hunger_relief: 15.0,
                energy_gain: 30.0,
                mood_boost: 3.0,
                stress_relief: 0.0,
                consumed: true,
                hunger_threshold: 0.0,
            }),
            Tile::CookedFish => Some(FoodProps {
                hunger_relief: 85.0,
                energy_gain: 20.0,
                mood_boost: 22.0,
                stress_relief: 12.0,
                consumed: true,
                hunger_threshold: 5.0,
            }),
            // Fish is NOT food — must be cooked first.
            // Fire's ambient effects are handled separately (not eaten).
            _ => None,
        }
    }

    /// Convert a haulable tile to its carried form. Inverse of Carry::to_tile().
    /// Returns None for non-haulable tiles.
    pub fn to_carry(self) -> Option<crate::bot::Carry> {
        use crate::bot::Carry;
        match self {
            Tile::Berry => Some(Carry::Berry),
            Tile::Log => Some(Carry::Log),
            Tile::Stone => Some(Carry::Stone),
            Tile::CookedBerry => Some(Carry::CookedBerry),
            Tile::Mushroom => Some(Carry::Mushroom),
            Tile::Fish => Some(Carry::Fish),
            Tile::CookedFish => Some(Carry::CookedFish),
            _ => None,
        }
    }

    /// Can this tile be cooked at a fire? (Raw ingredient → cooked version)
    pub fn is_cookable(self) -> bool {
        matches!(self, Tile::Berry | Tile::Fish)
    }

    /// What does this tile become after cooking? Returns None if not cookable.
    pub fn cooked_form(self) -> Option<Tile> {
        match self {
            Tile::Berry => Some(Tile::CookedBerry),
            Tile::Fish => Some(Tile::CookedFish),
            _ => None,
        }
    }

    /// Is this an obstacle that can be cleared by a frustrated bot?
    pub fn is_clearable(self) -> bool {
        matches!(self, Tile::Tree | Tile::Rock)
    }

    /// Clearing effort threshold — how much work to break through.
    /// Higher = harder. Returns None for non-clearable tiles.
    pub fn clear_effort(self) -> Option<u16> {
        match self {
            Tile::Tree => Some(60),
            Tile::Rock => Some(200),
            _ => None,
        }
    }

    /// What does this tile become when cleared?
    pub fn cleared_into(self) -> Tile {
        match self {
            Tile::Tree => Tile::Log,
            Tile::Rock => Tile::Stone,
            _ => Tile::Grass,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Weather {
    Clear,
    Raining,
    Clearing, // brief post-rain state where puddles stay but no new rain
}

pub struct World {
    pub tiles: Vec<u8>,
    pub bots: Vec<Bot>,
    pub rng: Rng,
    pub tick: u64,
    pub selected_bot: Option<usize>,
    pub event_log: Vec<String>,
    pub last_bubble_tick: u64,
    pub tree_complaints: Vec<(i32, i32)>,

    /// Per-tile walk counter. When grass accrues ≥ PATH_WEAR_THRESHOLD
    /// footsteps it converts to a Path. Non-grass tiles ignore this.
    pub path_wear: Vec<u8>,
    /// Fire-tile fuel clock. Each fire starts with a finite number of ticks;
    /// when it runs out the fire becomes Ash. Adding a Log resets/adds fuel.
    pub fire_fuel: HashMap<(i32, i32), u16>,
    /// Per-tile age for decaying props (Ash, Grave, Puddle, Log on ground).
    /// Stored sparsely by position; absence = untracked / no decay.
    pub tile_age: HashMap<(i32, i32), u16>,
    /// For cook jobs — per-tile "cooking progress". A berry sitting adjacent
    /// to a fire with a Cook standing by accumulates; at threshold it upgrades
    /// to CookedBerry.
    pub cook_progress: HashMap<(i32, i32), u16>,

    pub weather: Weather,
    pub weather_ticks: u32,
    /// Ticks per simulated day; used to colour-tint the world and shift
    /// warmth drives. 1200 ≈ 10s at 2× speed.
    pub day_length: u32,

    // --- Aggregate counters used by stats/telemetry ---
    pub graves_placed: u32,
    pub logs_chopped_total: u32,
    pub berries_cooked_total: u32,
    pub rains_total: u32,
}

pub const PATH_WEAR_THRESHOLD: u8 = 8;
// Fire fuel measured in ticks. At 2× speed ≈ 120 ticks/sec so 9000 is ~75s.
// Toolmakers chop slowly; fires need to outlive the chop→haul→refuel delay
// or the whole cooking chain collapses on itself.
pub const FIRE_INITIAL_FUEL: u16 = 9000;
pub const FIRE_LOG_FUEL: u16 = 4500;
pub const PUDDLE_LIFETIME: u16 = 1400;
pub const LOG_GROUND_LIFETIME: u16 = 3000;
pub const ASH_LIFETIME: u16 = 600;
pub const GRAVE_LIFETIME: u16 = 4000;

impl World {
    pub fn new(seed: u64) -> Self {
        let mut rng = Rng::new(seed);
        let tiles = generate_terrain(&mut rng);
        let mut world = Self {
            tiles,
            bots: Vec::new(),
            rng,
            tick: 0,
            selected_bot: None,
            event_log: Vec::new(),
            last_bubble_tick: 0,
            tree_complaints: Vec::new(),
            path_wear: vec![0u8; W * H],
            fire_fuel: HashMap::new(),
            tile_age: HashMap::new(),
            cook_progress: HashMap::new(),
            weather: Weather::Clear,
            weather_ticks: 0,
            day_length: 1200,
            graves_placed: 0,
            logs_chopped_total: 0,
            berries_cooked_total: 0,
            rains_total: 0,
        };
        world.spawn_initial_bots(24);
        world.seed_initial_fires(6);
        world
    }

    /// Place a handful of campfires in safe spots so bots have warmth anchors
    /// from tick 0. Without these, a population without a generous user will
    /// slowly freeze by nightfall.
    fn seed_initial_fires(&mut self, count: usize) {
        let mut placed = 0;
        let mut attempts = 0;
        while placed < count && attempts < 500 {
            attempts += 1;
            let x = self.rng.range_i32(4, W as i32 - 4);
            let y = self.rng.range_i32(4, H as i32 - 4);
            if matches!(self.tile(x, y), Tile::Grass | Tile::Sand)
                && self.bot_at(x, y).is_none()
            {
                // Don't stack fires — keep ~6 tiles between them.
                let mut too_close = false;
                for k in self.fire_fuel.keys() {
                    let d = (k.0 - x).abs() + (k.1 - y).abs();
                    if d < 6 {
                        too_close = true;
                        break;
                    }
                }
                if !too_close {
                    self.set_tile(x, y, Tile::Fire);
                    placed += 1;
                }
            }
        }
    }

    pub fn spawn_initial_bots(&mut self, count: usize) {
        let names = [
            "Blip", "Moss", "Pip", "Cog", "Zen", "Rune", "Tock", "Mox",
            "Fern", "Dot", "Nim", "Bop", "Wix", "Gus", "Kip", "Loz",
            "Ash", "Rho", "Vex", "Oro", "Ini", "Jet", "Qua", "Lum",
            "Pax", "Sip", "Tol", "Umi",
        ];
        let mut placed = 0;
        let mut attempts = 0;
        while placed < count && attempts < 2000 {
            attempts += 1;
            let x = self.rng.range_i32(2, W as i32 - 2);
            let y = self.rng.range_i32(2, H as i32 - 2);
            if self.tile(x, y).walkable() && !self.bot_at(x, y).is_some() {
                let name = names[placed % names.len()].to_string();
                let b = Bot::new(placed as u32, name, x, y, &mut self.rng);
                self.bots.push(b);
                placed += 1;
            }
        }
    }

    pub fn tile(&self, x: i32, y: i32) -> Tile {
        if x < 0 || y < 0 || x >= W as i32 || y >= H as i32 {
            return Tile::Rock;
        }
        Tile::from_u8(self.tiles[(y as usize) * W + (x as usize)])
    }

    pub fn set_tile(&mut self, x: i32, y: i32, t: Tile) {
        if x < 0 || y < 0 || x >= W as i32 || y >= H as i32 {
            return;
        }
        self.tiles[(y as usize) * W + (x as usize)] = t as u8;
        // Any tile change wipes its wear & cook progress so a new tile type
        // doesn't inherit stale state.
        if matches!(t, Tile::Grass | Tile::Path) {
            // preserve path wear across grass↔path transitions
        } else {
            self.path_wear[(y as usize) * W + (x as usize)] = 0;
        }
        self.cook_progress.remove(&(x, y));
        // Bookkeeping for decaying tiles.
        match t {
            Tile::Puddle | Tile::Ash | Tile::Grave | Tile::Log | Tile::Stone => {
                self.tile_age.insert((x, y), 0);
            }
            Tile::Fire => {
                self.fire_fuel.insert((x, y), FIRE_INITIAL_FUEL);
            }
            _ => {
                self.tile_age.remove(&(x, y));
                self.fire_fuel.remove(&(x, y));
            }
        }
    }

    pub fn bot_at(&self, x: i32, y: i32) -> Option<usize> {
        self.bots.iter().position(|b| b.x == x && b.y == y && b.alive)
    }

    /// Place a fire with a specific fuel amount. Use this instead of
    /// `set_tile(Fire)` when the fire should burn longer or shorter than
    /// FIRE_INITIAL_FUEL — e.g. a novice's campfire vs a Cook's well-built one.
    pub fn set_fire(&mut self, x: i32, y: i32, fuel: u16) {
        self.set_tile(x, y, Tile::Fire);
        // Override the default fuel that set_tile inserted.
        self.fire_fuel.insert((x, y), fuel);
    }

    /// Record a footstep on this tile — used to wear grass into a Path.
    pub fn mark_step(&mut self, x: i32, y: i32) {
        if x < 0 || y < 0 || x >= W as i32 || y >= H as i32 {
            return;
        }
        let idx = (y as usize) * W + (x as usize);
        if self.tiles[idx] == Tile::Grass as u8 {
            self.path_wear[idx] = self.path_wear[idx].saturating_add(1);
            if self.path_wear[idx] >= PATH_WEAR_THRESHOLD {
                self.tiles[idx] = Tile::Path as u8;
            }
        } else if self.tiles[idx] == Tile::Path as u8 {
            // Refresh path usage — prevents a path from being abandoned immediately.
            self.path_wear[idx] = PATH_WEAR_THRESHOLD;
        }
    }

    pub fn step(&mut self) {
        self.tick += 1;
        self.step_weather();
        self.step_decay();
        self.step_environment();

        // Bot updates.
        let snap: Vec<(i32, i32, bool, u32)> = self
            .bots
            .iter()
            .map(|b| (b.x, b.y, b.alive, b.id))
            .collect();
        for idx in 0..self.bots.len() {
            if !self.bots[idx].alive {
                continue;
            }
            crate::ai::think_and_act(self, idx, &snap);
        }

        // Death & graves.
        self.step_mortality();

        // Trim event log
        if self.event_log.len() > 80 {
            let drop = self.event_log.len() - 80;
            self.event_log.drain(0..drop);
        }
        // Prune complaints whose tree has already been felled (or morphed).
        if !self.tree_complaints.is_empty() {
            let tiles = &self.tiles;
            self.tree_complaints.retain(|(x, y)| {
                if *x < 0 || *y < 0 || *x >= W as i32 || *y >= H as i32 {
                    return false;
                }
                tiles[(*y as usize) * W + (*x as usize)] == Tile::Tree as u8
            });
        }
    }

    fn step_weather(&mut self) {
        self.weather_ticks = self.weather_ticks.saturating_add(1);
        match self.weather {
            Weather::Clear => {
                // Rain chance rises after at least ~20s of clear.
                if self.weather_ticks > 2400 && self.rng.chance(0.0015) {
                    self.weather = Weather::Raining;
                    self.weather_ticks = 0;
                    self.rains_total += 1;
                    self.log("Rain clouds gather.".to_string());
                }
            }
            Weather::Raining => {
                // Sprinkle puddles on grass/path while raining.
                if self.tick % 18 == 0 {
                    for _ in 0..5 {
                        let x = self.rng.range_i32(0, W as i32);
                        let y = self.rng.range_i32(0, H as i32);
                        if matches!(self.tile(x, y), Tile::Grass | Tile::Path)
                            && self.rng.chance(0.35)
                        {
                            self.set_tile(x, y, Tile::Puddle);
                        }
                    }
                }
                // Rain extinguishes unattended fires a little faster. Keep
                // this mild — an aggressive rain sweep made the whole first
                // storm wipe out every fire in the world.
                if self.tick % 30 == 0 {
                    let keys: Vec<(i32, i32)> = self.fire_fuel.keys().cloned().collect();
                    for k in keys {
                        if let Some(f) = self.fire_fuel.get_mut(&k) {
                            *f = f.saturating_sub(3);
                        }
                    }
                }
                // Lightning strike — moderate rate means one or two bolts per
                // storm on average. Each spark a fire on an unattended tile.
                if self.rng.chance(0.003) {
                    for _ in 0..20 {
                        let x = self.rng.range_i32(2, W as i32 - 2);
                        let y = self.rng.range_i32(2, H as i32 - 2);
                        if matches!(self.tile(x, y), Tile::Grass | Tile::Sand)
                            && self.bot_at(x, y).is_none()
                        {
                            self.set_tile(x, y, Tile::Fire);
                            self.log(format!("Lightning sparked a fire at ({},{})", x, y));
                            break;
                        }
                    }
                }
                if self.weather_ticks > 800 && self.rng.chance(0.01) {
                    self.weather = Weather::Clearing;
                    self.weather_ticks = 0;
                    self.log("The rain eases off.".to_string());
                }
            }
            Weather::Clearing => {
                if self.weather_ticks > 400 {
                    self.weather = Weather::Clear;
                    self.weather_ticks = 0;
                }
            }
        }
    }

    fn step_decay(&mut self) {
        // Age counters for sparse decaying tiles.
        let keys: Vec<(i32, i32)> = self.tile_age.keys().cloned().collect();
        for (x, y) in keys {
            let t = self.tile(x, y);
            let age = self.tile_age.get(&(x, y)).copied().unwrap_or(0).saturating_add(1);
            self.tile_age.insert((x, y), age);
            match t {
                Tile::Puddle if age >= PUDDLE_LIFETIME => {
                    self.set_tile(x, y, Tile::Grass);
                }
                Tile::Ash if age >= ASH_LIFETIME => {
                    self.set_tile(x, y, Tile::Grass);
                    // Fertile: boost chance of a sapling nearby.
                    if self.rng.chance(0.35) {
                        let dx = self.rng.range_i32(-1, 2);
                        let dy = self.rng.range_i32(-1, 2);
                        if matches!(self.tile(x + dx, y + dy), Tile::Grass) {
                            self.set_tile(x + dx, y + dy, Tile::Sapling);
                        }
                    }
                }
                Tile::Grave if age >= GRAVE_LIFETIME => {
                    // Graves fade to flowers (memorial) or grass.
                    if self.rng.chance(0.6) {
                        self.set_tile(x, y, Tile::Flower);
                    } else {
                        self.set_tile(x, y, Tile::Grass);
                    }
                }
                Tile::Log if age >= LOG_GROUND_LIFETIME => {
                    // Un-gathered log rots back into the soil.
                    self.set_tile(x, y, Tile::Grass);
                    if self.rng.chance(0.25) {
                        self.set_tile(x, y, Tile::Mushroom);
                    }
                }
                Tile::Stone if age >= LOG_GROUND_LIFETIME => {
                    // Stones just linger; fade slowly.
                    self.set_tile(x, y, Tile::Grass);
                }
                _ => {}
            }
        }

        // Fire fuel — each fire burns down a tick, extinguishes to Ash at 0.
        let fire_keys: Vec<(i32, i32)> = self.fire_fuel.keys().cloned().collect();
        for (x, y) in fire_keys {
            if self.tile(x, y) != Tile::Fire {
                self.fire_fuel.remove(&(x, y));
                continue;
            }
            let f = self.fire_fuel.get(&(x, y)).copied().unwrap_or(0);
            if f == 0 {
                self.set_tile(x, y, Tile::Ash);
                self.log(format!("A fire burned out at ({},{})", x, y));
            } else {
                self.fire_fuel.insert((x, y), f - 1);
            }
        }

        // Path erosion — paths not used fade back to grass.
        if self.tick % 300 == 0 {
            for y in 0..H {
                for x in 0..W {
                    let i = y * W + x;
                    if self.tiles[i] == Tile::Path as u8 {
                        self.path_wear[i] = self.path_wear[i].saturating_sub(1);
                        if self.path_wear[i] < 2 {
                            self.tiles[i] = Tile::Grass as u8;
                        }
                    }
                }
            }
        }
    }

    fn step_environment(&mut self) {
        // Environment tick — occasionally regrow berries near forest/tree tiles
        if self.tick % 20 == 0 {
            for _ in 0..3 {
                let x = self.rng.range_i32(0, W as i32);
                let y = self.rng.range_i32(0, H as i32);
                if self.tile(x, y) == Tile::Grass {
                    let mut nearby_wood = 0;
                    for dy in -1..=1 {
                        for dx in -1..=1 {
                            if matches!(self.tile(x + dx, y + dy), Tile::Forest | Tile::Tree) {
                                nearby_wood += 1;
                            }
                        }
                    }
                    if nearby_wood >= 2 && self.rng.chance(0.35) {
                        self.set_tile(x, y, Tile::Berry);
                    }
                }
            }
        }
        if self.tick % 200 == 0 {
            for i in 0..self.tiles.len() {
                if self.tiles[i] == Tile::Sapling as u8 && self.rng.chance(0.12) {
                    self.tiles[i] = Tile::Forest as u8;
                }
            }
        }
        if self.tick % 400 == 0 {
            for i in 0..self.tiles.len() {
                if self.tiles[i] == Tile::Forest as u8 && self.rng.chance(0.05) {
                    // Forest → Tree only if NOT adjacent to a path. Roads
                    // suppress tree maturation — this prevents trees from
                    // re-choking cleared thoroughfares.
                    let x = (i % W) as i32;
                    let y = (i / W) as i32;
                    let mut path_adj = false;
                    for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                        if matches!(self.tile(x + dx, y + dy), Tile::Path) {
                            path_adj = true;
                            break;
                        }
                    }
                    if !path_adj {
                        self.tiles[i] = Tile::Tree as u8;
                    }
                }
            }
        }
        // Forest self-thinning. Without bot intervention trees accumulate
        // until the map chokes; this simulates old-growth toppling into a
        // haulable log. Multi-tier:
        //   - Dense groves (4+ woody neighbours): 18% per pass
        //   - Very dense (6+ neighbours): 35% → rapid opening
        //   - Any tree when overpop: 10% baseline
        //   - Lone trees past severe overpop: 15%
        // Cap total tree coverage at ~12% of the map (~490 tiles on 64×64).
        // Runs every 200 ticks for responsiveness.
        if self.tick % 200 == 0 {
            let mut victims: Vec<(usize, bool)> = Vec::new();
            let mut tree_count = 0usize;
            let mut forest_count = 0usize;
            for i in 0..self.tiles.len() {
                if self.tiles[i] == Tile::Tree as u8 { tree_count += 1; }
                if self.tiles[i] == Tile::Forest as u8 { forest_count += 1; }
            }
            let total_woody = tree_count + forest_count;
            let overpop = total_woody > (W * H * 12 / 100);
            let severe = total_woody > (W * H * 20 / 100);
            for y in 1..H - 1 {
                for x in 1..W - 1 {
                    let i = y * W + x;
                    if self.tiles[i] != Tile::Tree as u8 {
                        continue;
                    }
                    let mut woody = 0;
                    for dy in -1i32..=1 {
                        for dx in -1i32..=1 {
                            if dx == 0 && dy == 0 {
                                continue;
                            }
                            let nx = x as i32 + dx;
                            let ny = y as i32 + dy;
                            if matches!(self.tile(nx, ny), Tile::Tree | Tile::Forest) {
                                woody += 1;
                            }
                        }
                    }
                    let chance = if woody >= 6 {
                        0.35
                    } else if woody >= 4 {
                        0.18
                    } else if severe {
                        0.15
                    } else if overpop {
                        0.10
                    } else if woody >= 2 {
                        0.03
                    } else {
                        0.0
                    };
                    if chance > 0.0 && self.rng.chance(chance) {
                        victims.push((i, woody >= 3));
                    }
                }
            }
            // Also thin some Forest tiles during severe overpop
            if severe {
                for y in 1..H - 1 {
                    for x in 1..W - 1 {
                        let i = y * W + x;
                        if self.tiles[i] == Tile::Forest as u8 && self.rng.chance(0.08) {
                            victims.push((i, false));
                        }
                    }
                }
            }
            for (i, drop_log) in victims {
                if drop_log {
                    self.tiles[i] = Tile::Log as u8;
                    let x = (i % W) as i32;
                    let y = (i / W) as i32;
                    self.tile_age.insert((x, y), 0);
                } else {
                    self.tiles[i] = Tile::Grass as u8;
                }
            }
        }
        // Mature trees periodically drop berries on nearby grass
        if self.tick % 60 == 0 {
            for _ in 0..4 {
                let x = self.rng.range_i32(0, W as i32);
                let y = self.rng.range_i32(0, H as i32);
                if self.tile(x, y) != Tile::Tree {
                    continue;
                }
                let dx = self.rng.range_i32(-1, 2);
                let dy = self.rng.range_i32(-1, 2);
                let (nx, ny) = (x + dx, y + dy);
                if (dx != 0 || dy != 0) && self.tile(nx, ny) == Tile::Grass && self.rng.chance(0.55) {
                    self.set_tile(nx, ny, Tile::Berry);
                }
            }
        }
        // Field harvest — tilled plots spawn berries on adjacent grass every
        // 80 ticks. Fields near water are more productive (0.45 chance vs 0.25).
        if self.tick % 80 == 0 {
            for y in 1..H - 1 {
                for x in 1..W - 1 {
                    let i = y * W + x;
                    if self.tiles[i] != Tile::Field as u8 {
                        continue;
                    }
                    let xi = x as i32;
                    let yi = y as i32;
                    let mut near_water = false;
                    for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                        if matches!(self.tile(xi + dx, yi + dy), Tile::Water | Tile::Puddle) {
                            near_water = true;
                            break;
                        }
                    }
                    let prob = if near_water { 0.45 } else { 0.25 };
                    if self.rng.chance(prob) {
                        // Pick a random adjacent tile for the berry.
                        let dx = self.rng.range_i32(-1, 2);
                        let dy = self.rng.range_i32(-1, 2);
                        if (dx != 0 || dy != 0)
                            && matches!(self.tile(xi + dx, yi + dy), Tile::Grass | Tile::Path)
                        {
                            self.set_tile(xi + dx, yi + dy, Tile::Berry);
                        }
                    }
                }
            }
        }
        // Mushrooms in cool-damp spots — forests + rock/water neighbours.
        if self.tick % 90 == 0 {
            for _ in 0..4 {
                let x = self.rng.range_i32(1, W as i32 - 1);
                let y = self.rng.range_i32(1, H as i32 - 1);
                if self.tile(x, y) != Tile::Grass {
                    continue;
                }
                let mut woody = 0;
                let mut damp = 0;
                for dy in -1..=1 {
                    for dx in -1..=1 {
                        match self.tile(x + dx, y + dy) {
                            Tile::Forest | Tile::Tree => woody += 1,
                            Tile::Rock | Tile::Water | Tile::Puddle => damp += 1,
                            _ => {}
                        }
                    }
                }
                if woody >= 1 && damp >= 1 && self.rng.chance(0.5) {
                    self.set_tile(x, y, Tile::Mushroom);
                }
            }
        }
    }

    fn step_mortality(&mut self) {
        for idx in 0..self.bots.len() {
            let b = &self.bots[idx];
            if !b.alive {
                continue;
            }
            // Death conditions:
            //   - hunger maxed + energy crashed for sustained period
            //   - or very old with low mood
            let starving = b.hunger >= 100.0 && b.energy <= 0.0;
            let ancient = b.age > 30_000 && b.mood < -60.0;
            let died = starving && b.stress > 80.0 || ancient;
            if died {
                let (x, y, name) = (b.x, b.y, b.name.clone());
                self.bots[idx].alive = false;
                self.set_tile(x, y, Tile::Grave);
                self.graves_placed += 1;
                self.log(format!("{} passed on at ({},{})", name, x, y));
            }
        }
    }

    pub fn log(&mut self, s: String) {
        self.event_log.push(format!("[{}] {}", self.tick, s));
    }

    /// Register a blocking tree so the Toolmakers know to service it.
    pub fn push_complaint(&mut self, x: i32, y: i32) {
        if !matches!(self.tile(x, y), Tile::Tree) {
            return;
        }
        if self.tree_complaints.iter().any(|(cx, cy)| *cx == x && *cy == y) {
            return;
        }
        self.tree_complaints.push((x, y));
        if self.tree_complaints.len() > 24 {
            self.tree_complaints.remove(0);
        }
    }

    /// Crude day-night phase in [0,1). 0 = midnight, 0.5 = noon.
    pub fn day_phase(&self) -> f32 {
        let t = (self.tick % self.day_length as u64) as f32 / self.day_length as f32;
        t
    }

    pub fn is_night(&self) -> bool {
        let p = self.day_phase();
        p < 0.15 || p > 0.85
    }

    /// Check if any adjacent tile (4-directional) matches a predicate.
    pub fn has_adjacent(&self, x: i32, y: i32, pred: impl Fn(Tile) -> bool) -> bool {
        for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            if pred(self.tile(x + dx, y + dy)) {
                return true;
            }
        }
        false
    }

    /// Find nearest tile within radius matching a predicate (Manhattan distance).
    pub fn find_nearest_where(&self, bx: i32, by: i32, radius: i32, pred: impl Fn(Tile) -> bool) -> Option<(i32, i32)> {
        let mut best: Option<(i32, (i32, i32))> = None;
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                let x = bx + dx;
                let y = by + dy;
                if x < 0 || y < 0 || x >= W as i32 || y >= H as i32 {
                    continue;
                }
                if pred(self.tile(x, y)) {
                    let d = dx.abs() + dy.abs();
                    if best.as_ref().map_or(true, |(bd, _)| d < *bd) {
                        best = Some((d, (x, y)));
                    }
                }
            }
        }
        best.map(|(_, p)| p)
    }
}

// -- Terrain generation ----------------------------------------------------

fn generate_terrain(rng: &mut Rng) -> Vec<u8> {
    let elev = noise_grid(rng, 16);
    let moist = noise_grid(rng, 16);
    let mut tiles = vec![Tile::Grass as u8; W * H];
    for y in 0..H {
        for x in 0..W {
            let e = sample(&elev, x, y);
            let m = sample(&moist, x, y);
            let t = if e < 0.30 {
                Tile::Water
            } else if e < 0.36 {
                Tile::Sand
            } else if e > 0.82 {
                Tile::Rock
            } else if m > 0.65 && e < 0.70 {
                Tile::Forest
            } else if m > 0.55 && e < 0.70 {
                if rng.chance(0.04) {
                    Tile::Flower
                } else {
                    Tile::Grass
                }
            } else {
                Tile::Grass
            };
            tiles[y * W + x] = t as u8;
        }
    }
    for y in 1..H - 1 {
        for x in 1..W - 1 {
            let idx = y * W + x;
            if tiles[idx] == Tile::Grass as u8 {
                let mut forest_neighbors = 0;
                let mut shaded = 0;
                for dy in -1i32..=1 {
                    for dx in -1i32..=1 {
                        let nx = (x as i32 + dx) as usize;
                        let ny = (y as i32 + dy) as usize;
                        if tiles[ny * W + nx] == Tile::Forest as u8 {
                            forest_neighbors += 1;
                        }
                        if matches!(
                            Tile::from_u8(tiles[ny * W + nx]),
                            Tile::Rock | Tile::Water
                        ) {
                            shaded += 1;
                        }
                    }
                }
                if forest_neighbors >= 2 && rng.chance(0.18) {
                    tiles[idx] = Tile::Berry as u8;
                } else if forest_neighbors >= 1 && shaded >= 1 && rng.chance(0.08) {
                    tiles[idx] = Tile::Mushroom as u8;
                }
            } else if tiles[idx] == Tile::Forest as u8 && rng.chance(0.12) {
                tiles[idx] = Tile::Tree as u8;
            }
        }
    }
    tiles
}

fn noise_grid(rng: &mut Rng, coarse: usize) -> Vec<f32> {
    let mut g = vec![0f32; coarse * coarse];
    for v in g.iter_mut() {
        *v = rng.gen_f32();
    }
    g
}

fn sample(grid: &[f32], x: usize, y: usize) -> f32 {
    let coarse = 16;
    let scale = (W / coarse) as f32;
    let fx = x as f32 / scale;
    let fy = y as f32 / scale;
    let x0 = (fx as usize).min(coarse - 1);
    let y0 = (fy as usize).min(coarse - 1);
    let x1 = (x0 + 1).min(coarse - 1);
    let y1 = (y0 + 1).min(coarse - 1);
    let tx = fx - x0 as f32;
    let ty = fy - y0 as f32;
    let sx = tx * tx * (3.0 - 2.0 * tx);
    let sy = ty * ty * (3.0 - 2.0 * ty);
    let a = grid[y0 * coarse + x0];
    let b = grid[y0 * coarse + x1];
    let c = grid[y1 * coarse + x0];
    let d = grid[y1 * coarse + x1];
    let ab = a + (b - a) * sx;
    let cd = c + (d - c) * sx;
    ab + (cd - ab) * sy
}
