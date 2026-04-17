use crate::rng::Rng;
use std::collections::HashMap;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Goal {
    Idle,
    Eat,
    Rest,
    Socialize,
    Explore,
    Forage,
    Build,
    Flee,
    Visit, // going somewhere specific (remembered location)
    Craft, // toolmaker: go to a rock, knap a stone axe
    Chop,  // toolmaker: go to a complaint tree and fell it
    // --- new goals (the crazy expansion) ---
    Drink,   // thirst-relief: find water/puddle
    Cook,    // cook-job: upgrade a berry near a fire
    Gather,  // pick up a haulable (Log/Stone/Berry)
    Deliver, // carry carried item to home / fire / friend
    Warm,    // at night or cold: head to a fire
    Mourn,   // visit a grave
    Heal,    // stressed bot heads for restorative place
    Fish,    // fisherman: go to water edge, catch fish
    Farm,    // farmer: tend/create a field plot
}

impl Goal {
    pub fn label(self) -> &'static str {
        match self {
            Goal::Idle => "wandering",
            Goal::Eat => "eating",
            Goal::Rest => "resting",
            Goal::Socialize => "socializing",
            Goal::Explore => "exploring",
            Goal::Forage => "foraging",
            Goal::Build => "building",
            Goal::Flee => "fleeing",
            Goal::Visit => "visiting",
            Goal::Craft => "crafting",
            Goal::Chop => "chopping",
            Goal::Drink => "drinking",
            Goal::Cook => "cooking",
            Goal::Gather => "gathering",
            Goal::Deliver => "delivering",
            Goal::Warm => "warming",
            Goal::Mourn => "mourning",
            Goal::Heal => "healing",
            Goal::Fish => "fishing",
            Goal::Farm => "farming",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Job {
    Forager,   // specialises in finding/eating food
    Builder,   // raises homes, likes structure
    Scout,     // explores the map far and wide
    Guardian,  // patrols, confronts rather than flees
    Socialite, // visits others, forges bonds
    Farmer,    // plants saplings, tends the land
    Hermit,    // stays near home, contemplates
    Toolmaker, // knaps stone axes, clears blocking trees
    // --- new jobs ---
    Cook,      // tends fires, upgrades berries to CookedBerries
    Digger,    // hauls stones, builds up shrines
    Healer,    // notices stressed bots, delivers food
    Fisherman, // crafts a pole at rock, catches fish at water edges
}

impl Job {
    pub fn label(self) -> &'static str {
        match self {
            Job::Forager => "forager",
            Job::Builder => "builder",
            Job::Scout => "scout",
            Job::Guardian => "guardian",
            Job::Socialite => "socialite",
            Job::Farmer => "farmer",
            Job::Hermit => "hermit",
            Job::Toolmaker => "toolmaker",
            Job::Cook => "cook",
            Job::Digger => "digger",
            Job::Healer => "healer",
            Job::Fisherman => "fisherman",
        }
    }

    pub fn from_traits(t: &Trait, rng: &mut Rng) -> Self {
        // Weighted sampling biased by traits so personality tends to match vocation.
        let weights = [
            (Job::Forager, 0.20 + t.industriousness * 0.35 + (1.0 - t.aggression) * 0.10),
            (Job::Builder, 0.15 + t.industriousness * 0.55),
            (Job::Scout, 0.15 + t.curiosity * 0.70 + t.bravery * 0.25),
            (Job::Guardian, 0.10 + t.bravery * 0.55 + t.aggression * 0.35),
            (Job::Socialite, 0.15 + t.sociability * 0.75),
            (Job::Farmer, 0.15 + t.industriousness * 0.20 + (1.0 - t.aggression) * 0.30),
            (Job::Hermit, 0.15 + (1.0 - t.sociability) * 0.55),
            (Job::Toolmaker, 0.12 + t.industriousness * 0.45 + t.curiosity * 0.25),
            (Job::Cook, 0.14 + t.industriousness * 0.35 + t.sociability * 0.25),
            (Job::Digger, 0.14 + t.industriousness * 0.50 + (1.0 - t.curiosity) * 0.15),
            (Job::Healer, 0.12 + t.sociability * 0.50 + (1.0 - t.aggression) * 0.30),
            (Job::Fisherman, 0.12 + (1.0 - t.aggression) * 0.30 + t.curiosity * 0.20 + (1.0 - t.industriousness) * 0.15),
        ];
        let total: f32 = weights.iter().map(|(_, w)| *w).sum();
        let mut pick = rng.gen_f32() * total;
        for (j, w) in &weights {
            if pick < *w {
                return *j;
            }
            pick -= *w;
        }
        Job::Forager
    }
}

#[derive(Copy, Clone, Debug)]
pub enum MemKind {
    Food,
    Home,
    Friend(u32),
    Enemy(u32),
    Water,   // drinkable tile spot
    Fire,    // campfire location
    Log,     // a log on the ground — haulable
    Stone,   // a stone on the ground — haulable
    Grave,   // a grave for mourning
}

#[derive(Copy, Clone, Debug)]
pub struct Memory {
    pub kind: MemKind,
    pub x: i32,
    pub y: i32,
    pub tick: u64,
}

/// What a bot is physically holding. Only one thing at a time —
/// the bot walks with a visible carry indicator pixel on their head.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Carry {
    None,
    Berry,
    Log,
    Stone,
    CookedBerry,
    Mushroom,
    Fish,
    CookedFish,
}

impl Carry {
    pub fn label(self) -> &'static str {
        match self {
            Carry::None => "nothing",
            Carry::Berry => "berry",
            Carry::Log => "log",
            Carry::Stone => "stone",
            Carry::CookedBerry => "cooked berry",
            Carry::Mushroom => "mushroom",
            Carry::Fish => "raw fish",
            Carry::CookedFish => "cooked fish",
        }
    }

    /// Pixel color for the carry indicator over the bot's head.
    pub fn color(self) -> [u8; 3] {
        match self {
            Carry::None => [0, 0, 0],
            Carry::Berry => [214, 52, 64],
            Carry::Log => [136, 94, 56],
            Carry::Stone => [180, 180, 188],
            Carry::CookedBerry => [240, 132, 72],
            Carry::Mushroom => [220, 70, 74],
            Carry::Fish => [100, 160, 210],
            Carry::CookedFish => [240, 170, 90],
        }
    }
}

/// Personality traits — all in [0,1]. Immutable once born.
#[derive(Copy, Clone, Debug)]
pub struct Trait {
    pub curiosity: f32,
    pub sociability: f32,
    pub aggression: f32,
    pub industriousness: f32,
    pub bravery: f32,
}

impl Trait {
    pub fn roll(rng: &mut Rng) -> Self {
        Self {
            curiosity: rng.gen_f32(),
            sociability: rng.gen_f32(),
            aggression: rng.gen_f32() * 0.8, // bias peaceful
            industriousness: rng.gen_f32(),
            bravery: rng.gen_f32(),
        }
    }

    pub fn dominant(&self) -> &'static str {
        let vals = [
            ("curious", self.curiosity),
            ("social", self.sociability),
            ("fierce", self.aggression),
            ("busy", self.industriousness),
            ("brave", self.bravery),
        ];
        let mut best = vals[0];
        for v in &vals[1..] {
            if v.1 > best.1 {
                best = *v;
            }
        }
        best.0
    }
}

pub struct Bot {
    pub id: u32,
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub facing: (i32, i32),
    /// Visual position for smooth rendering. Lerps toward (x,y) each tick.
    pub visual_x: f32,
    pub visual_y: f32,

    pub alive: bool,

    // drives (0..100, higher = more urgent)
    pub energy: f32,   // 100 = full, 0 = exhausted
    pub hunger: f32,   // 0 = sated, 100 = starving
    pub social: f32,   // 0 = content, 100 = lonely
    pub boredom: f32,  // 0 = engaged, 100 = very bored
    pub mood: f32,     // -100..100

    // --- new drives ---
    pub thirst: f32,   // 0 = hydrated, 100 = parched
    pub stress: f32,   // 0 = calm, 100 = breaking point
    pub warmth: f32,   // 100 = cozy, 0 = shivering cold

    pub traits: Trait,

    pub color: [u8; 3],

    pub memory: Vec<Memory>,
    pub home: Option<(i32, i32)>,
    pub relationships: HashMap<u32, i32>, // bot id -> affinity

    pub goal: Goal,
    pub target: Option<(i32, i32)>,
    pub goal_ticks: u32, // how long in current goal (hysteresis)

    pub thought: String,
    pub recent_thoughts: Vec<String>,
    pub thought_ttl: u32, // ticks remaining to show thought bubble
    /// Frozen text to show in the bubble. Snapshotted when the bubble
    /// surfaces so that `thought` rotating internally doesn't make the
    /// bubble text flicker while it's still on-screen.
    pub bubble_text: String,
    pub age: u32,         // ticks

    pub job: Job,
    pub speed: u8,         // ticks between movement steps (higher = slower)
    pub move_cooldown: u8, // ticks remaining before next move

    // Bubble surfacing — set when something salient happens.
    // Only events that flip this to true generate a visible bubble (subject to
    // a global world-level cooldown). Internal thoughts still update
    // `thought` / `recent_thoughts` for the inspector panel.
    pub pending_bubble: bool,
    /// When true, the pending bubble bypasses the global bubble cooldown.
    /// Used for change-of-motivation announcements which always show.
    pub pending_priority_high: bool,
    pub hunger_alarm: bool, // true while hunger is critically high
    pub tired_alarm: bool,  // true while energy is critically low
    pub thirst_alarm: bool, // true while thirst is critically high
    pub stress_alarm: bool, // true while stress is critically high
    /// Ticks remaining during which the bot stands still and contemplates
    /// a newly-chosen goal. Movement is suspended until this hits zero —
    /// the bubble shows first, then action follows.
    pub commitment_delay: u16,

    // -- Social texture --------------------------------------------------
    /// Id of the bot we're currently chatting with. Conversations are
    /// symmetric — both bots set this to each other. Movement is gently
    /// suppressed and thoughts become relational while set.
    pub chatting_with: Option<u32>,
    pub chat_ticks: u32,           // ticks spent in current conversation
    pub last_chat_tick: u64,       // tick we were last in any conversation
    pub gifts_given: u32,
    pub gifts_received: u32,
    /// Emergent name for the region near home. Filled the first time
    /// introspection runs while standing in/near the home.
    pub home_name: Option<String>,
    /// Cooldown before we try gifting again — avoids spam.
    pub gift_cooldown: u16,
    /// Cooldown before we start another chat — avoids instant re-chat.
    pub chat_cooldown: u16,
    /// Cooldown before we say a passing greeting to a neighbour.
    pub greet_cooldown: u16,

    // -- Frustration / obstacle tracking ----------------------------------
    /// Ticks spent making no forward progress toward the current target.
    /// Rises every movement tick the bot stays in place or moves sideways;
    /// resets whenever actual forward progress is made. High values drive
    /// desperate obstacle-clearing attempts — anyone can bash through
    /// a tree or rock, they just do it slowly and painfully.
    pub stuck_ticks: u16,
    /// Position + type of the tile that's currently blocking us.
    /// Populated by step_toward_target when blocked; cleared on progress.
    /// Shared during social interactions so friends learn about obstacles.
    pub blocked_by: Option<(i32, i32, u8)>, // (x, y, tile_type_u8)
    /// Accumulated clearing effort on the current obstacle. Fills up toward
    /// a threshold that depends on tool + industriousness. Resets when the
    /// obstacle changes or is cleared.
    pub clear_progress: u16,

    // -- Craft / tools ---------------------------------------------------
    /// Stone-axe durability. 0 = no tool. Each chop consumes 1.
    pub has_tool: u8,
    /// Ticks spent adjacent to a rock while crafting. Fills up, then yields
    /// a tool and resets.
    pub craft_progress: u16,
    /// Trees this bot has personally felled. Small bragging stat.
    pub trees_chopped: u32,
    /// Rocks this bot has personally broken.
    pub rocks_broken: u32,

    // -- Hauling / carrying ---------------------------------------------
    /// What the bot is physically carrying right now.
    pub carrying: Carry,
    /// Ticks spent carrying the current item — hauling is tiring.
    pub carry_ticks: u32,
    /// Total items this bot has ever dropped off somewhere useful.
    pub deliveries: u32,

    // -- Reputation ------------------------------------------------------
    /// Social reputation in the village. Positive = admired, negative =
    /// suspicious. Decays slowly, bumped by gifts/deliveries/mourning.
    pub reputation: i32,
    /// Number of berries this bot has personally cooked.
    pub berries_cooked: u32,
}

impl Bot {
    pub fn new(id: u32, name: String, x: i32, y: i32, rng: &mut Rng) -> Self {
        let t = Trait::roll(rng);
        let color = trait_color(&t, rng);
        let job = Job::from_traits(&t, rng);
        // Base movement cadence (ticks between steps). Doubled from earlier
        // tuning: we want visibly slow saunter so viewers can read bubbles
        // and track bots by eye. Thinking (goal choice, inner thoughts,
        // alarms) still runs every tick — only the legs are slow.
        let base_speed: u8 = match job {
            Job::Scout => 16,
            Job::Guardian => 18,
            Job::Forager => 20,
            Job::Socialite => 22,
            Job::Builder => 24,
            Job::Farmer => 24,
            Job::Toolmaker => 22,
            Job::Hermit => 28,
            Job::Cook => 22,
            Job::Digger => 22,
            Job::Healer => 20,
            Job::Fisherman => 22,
        };
        let jitter = (rng.next_u64() % 9) as u8; // +0..8
        let speed = base_speed.saturating_add(jitter);
        Self {
            id,
            name,
            x,
            y,
            visual_x: x as f32,
            visual_y: y as f32,
            facing: (0, 1),
            alive: true,
            energy: 70.0 + rng.gen_f32() * 30.0,
            hunger: rng.gen_f32() * 30.0,
            social: rng.gen_f32() * 40.0,
            boredom: rng.gen_f32() * 30.0,
            mood: 0.0,
            thirst: rng.gen_f32() * 25.0,
            stress: 0.0,
            warmth: 75.0 + rng.gen_f32() * 25.0,
            traits: t,
            color,
            memory: Vec::new(),
            home: None,
            relationships: HashMap::new(),
            goal: Goal::Idle,
            target: None,
            goal_ticks: 0,
            thought: String::from("..."),
            recent_thoughts: Vec::new(),
            thought_ttl: 0,
            bubble_text: String::new(),
            age: 0,
            job,
            speed,
            move_cooldown: (rng.next_u64() % speed as u64) as u8,
            pending_bubble: false,
            pending_priority_high: false,
            hunger_alarm: false,
            tired_alarm: false,
            thirst_alarm: false,
            stress_alarm: false,
            commitment_delay: 0,
            chatting_with: None,
            chat_ticks: 0,
            last_chat_tick: 0,
            gifts_given: 0,
            gifts_received: 0,
            home_name: None,
            gift_cooldown: 0,
            chat_cooldown: 0,
            greet_cooldown: 0,
            stuck_ticks: 0,
            blocked_by: None,
            clear_progress: 0,
            has_tool: 0,
            craft_progress: 0,
            trees_chopped: 0,
            rocks_broken: 0,
            carrying: Carry::None,
            carry_ticks: 0,
            deliveries: 0,
            reputation: 0,
            berries_cooked: 0,
        }
    }

    /// Quietly update the current thought and append to recent history.
    /// Does NOT trigger a bubble on its own — that's the job of `surface()`.
    pub fn set_thought(&mut self, s: String) {
        if self.thought != s {
            self.recent_thoughts.push(s.clone());
            if self.recent_thoughts.len() > 12 {
                self.recent_thoughts.remove(0);
            }
            self.thought = s;
        }
    }

    /// Force the thought text AND flag it to pop as a bubble.
    /// Subject to the global bubble cooldown — used for ordinary events
    /// like "ate while starving" or "planted a sapling".
    pub fn announce(&mut self, s: String) {
        self.set_thought(s);
        self.pending_bubble = true;
    }

    /// Like `announce`, but this bubble bypasses the cooldown and always
    /// surfaces. Use for change-of-motivation / goal-pivot declarations —
    /// the "I'm going to build a home now" moment that precedes action.
    pub fn announce_now(&mut self, s: String) {
        self.set_thought(s);
        self.pending_bubble = true;
        self.pending_priority_high = true;
    }

    /// Actually show the bubble for `ticks` ticks. Snapshots the current
    /// thought into `bubble_text` so the bubble stays stable while displayed.
    pub fn surface(&mut self, ticks: u32) {
        self.thought_ttl = ticks;
        self.bubble_text = self.thought.clone();
        self.pending_bubble = false;
    }

    pub fn remember(&mut self, kind: MemKind, x: i32, y: i32, tick: u64) {
        // de-dup roughly by kind+position
        for m in self.memory.iter_mut() {
            let same = match (m.kind, kind) {
                (MemKind::Food, MemKind::Food) => true,
                (MemKind::Home, MemKind::Home) => true,
                (MemKind::Water, MemKind::Water) => true,
                (MemKind::Fire, MemKind::Fire) => true,
                (MemKind::Log, MemKind::Log) => true,
                (MemKind::Stone, MemKind::Stone) => true,
                (MemKind::Grave, MemKind::Grave) => true,
                (MemKind::Friend(a), MemKind::Friend(b)) => a == b,
                (MemKind::Enemy(a), MemKind::Enemy(b)) => a == b,
                _ => false,
            };
            if same && m.x == x && m.y == y {
                m.tick = tick;
                return;
            }
        }
        self.memory.push(Memory { kind, x, y, tick });
        // cap memory
        if self.memory.len() > 40 {
            // forget the oldest
            let mut oldest = 0usize;
            for (i, m) in self.memory.iter().enumerate() {
                if m.tick < self.memory[oldest].tick {
                    oldest = i;
                }
            }
            self.memory.remove(oldest);
        }
    }

    pub fn forget(&mut self, predicate: impl Fn(&Memory) -> bool) {
        self.memory.retain(|m| !predicate(m));
    }

    pub fn nearest_mem<F: Fn(&Memory) -> bool>(&self, f: F) -> Option<Memory> {
        let mut best: Option<(i32, Memory)> = None;
        for m in &self.memory {
            if f(m) {
                let d = (m.x - self.x).abs() + (m.y - self.y).abs();
                if best.as_ref().map_or(true, |(bd, _)| d < *bd) {
                    best = Some((d, *m));
                }
            }
        }
        best.map(|(_, m)| m)
    }
}

fn trait_color(t: &Trait, rng: &mut Rng) -> [u8; 3] {
    // Hue derived from dominant trait; saturation/lightness from others.
    // Use HSV → RGB with fixed saturation for punchy 8-bit palette feel.
    let hue = match t.dominant() {
        "curious" => 200.0,   // cyan
        "social" => 330.0,    // pink
        "fierce" => 10.0,     // red
        "busy" => 45.0,       // orange
        "brave" => 100.0,     // green
        _ => 260.0,
    };
    let hue = hue + (rng.gen_f32() - 0.5) * 30.0;
    let s = 0.7 + t.aggression * 0.25;
    let v = 0.75 + t.bravery * 0.2;
    hsv_to_rgb(hue, s, v.min(1.0))
}

pub fn hsv_to_rgb(h: f32, s: f32, v: f32) -> [u8; 3] {
    let c = v * s;
    let hp = (h.rem_euclid(360.0)) / 60.0;
    let x = c * (1.0 - (hp.rem_euclid(2.0) - 1.0).abs());
    let (r, g, b) = if hp < 1.0 {
        (c, x, 0.0)
    } else if hp < 2.0 {
        (x, c, 0.0)
    } else if hp < 3.0 {
        (0.0, c, x)
    } else if hp < 4.0 {
        (0.0, x, c)
    } else if hp < 5.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };
    let m = v - c;
    [
        ((r + m) * 255.0) as u8,
        ((g + m) * 255.0) as u8,
        ((b + m) * 255.0) as u8,
    ]
}
