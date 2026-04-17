use wasm_bindgen::prelude::*;

mod ai;
mod bot;
mod render;
mod rng;
mod world;

use bot::{Bot, Goal, Job, MemKind};
use world::{Tile, Weather, World, CANVAS_H, CANVAS_W, TILE};

#[wasm_bindgen]
pub struct Gridland {
    world: World,
    buffer: Vec<u8>,
}

#[wasm_bindgen]
impl Gridland {
    #[wasm_bindgen(constructor)]
    pub fn new(seed: u32) -> Gridland {
        let world = World::new(seed as u64);
        let buffer = vec![0u8; CANVAS_W * CANVAS_H * 4];
        Gridland { world, buffer }
    }

    pub fn tick(&mut self) {
        self.world.step();
    }

    pub fn render(&mut self) {
        render::render_to_buffer(&self.world, &mut self.buffer);
    }

    pub fn buffer_ptr(&self) -> *const u8 {
        self.buffer.as_ptr()
    }

    pub fn buffer_len(&self) -> usize {
        self.buffer.len()
    }

    pub fn canvas_w(&self) -> u32 {
        CANVAS_W as u32
    }

    pub fn canvas_h(&self) -> u32 {
        CANVAS_H as u32
    }

    pub fn tile_size(&self) -> u32 {
        TILE as u32
    }

    pub fn current_tick(&self) -> u32 {
        self.world.tick as u32
    }

    /// Click at pixel coords → select bot if present, else return false.
    /// Returns the selected bot id, or -1 if none.
    pub fn click_select(&mut self, px: i32, py: i32) -> i32 {
        let tx = px / TILE as i32;
        let ty = py / TILE as i32;
        for (i, b) in self.world.bots.iter().enumerate() {
            if b.alive && b.x == tx && b.y == ty {
                self.world.selected_bot = Some(i);
                return b.id as i32;
            }
        }
        // else: if tapping on empty grass, drop a berry (quick interaction)
        let t = self.world.tile(tx, ty);
        if matches!(t, Tile::Grass | Tile::Sand) {
            self.world.set_tile(tx, ty, Tile::Berry);
            self.world.log(format!("Visitor dropped a berry at ({},{})", tx, ty));
        }
        self.world.selected_bot = None;
        -1
    }

    pub fn clear_selection(&mut self) {
        self.world.selected_bot = None;
    }

    /// Drop a berry at tile coords
    pub fn drop_food(&mut self, tx: i32, ty: i32) {
        let t = self.world.tile(tx, ty);
        if matches!(t, Tile::Grass | Tile::Sand) {
            self.world.set_tile(tx, ty, Tile::Berry);
            self.world.log(format!("Visitor dropped a berry at ({},{})", tx, ty));
        }
    }

    /// Drop a rock obstacle
    pub fn drop_rock(&mut self, tx: i32, ty: i32) {
        let t = self.world.tile(tx, ty);
        if matches!(t, Tile::Grass | Tile::Sand | Tile::Flower) {
            // don't drop on a bot
            if self.world.bot_at(tx, ty).is_none() {
                self.world.set_tile(tx, ty, Tile::Rock);
                self.world.log(format!("Visitor placed a rock at ({},{})", tx, ty));
            }
        }
    }

    /// Light a campfire — attracts bots, lifts mood, soothes loneliness.
    pub fn drop_fire(&mut self, tx: i32, ty: i32) {
        let t = self.world.tile(tx, ty);
        if matches!(t, Tile::Grass | Tile::Sand | Tile::Flower) {
            self.world.set_tile(tx, ty, Tile::Fire);
            self.world.log(format!("Visitor lit a campfire at ({},{})", tx, ty));
        }
    }

    /// Clear a tile back to grass (erase user-placed things)
    pub fn clear_tile(&mut self, tx: i32, ty: i32) {
        let t = self.world.tile(tx, ty);
        if matches!(t, Tile::Rock | Tile::Berry | Tile::Sapling | Tile::Flower) {
            self.world.set_tile(tx, ty, Tile::Grass);
        }
    }

    /// Return JSON string with the selected bot's full state.
    pub fn selected_info(&self) -> String {
        let idx = match self.world.selected_bot {
            Some(i) => i,
            None => return "null".to_string(),
        };
        if idx >= self.world.bots.len() {
            return "null".to_string();
        }
        let b = &self.world.bots[idx];
        let partner_name = b.chatting_with.and_then(|pid| {
            self.world.bots.iter().find(|o| o.id == pid && o.alive).map(|o| o.name.clone())
        });
        bot_to_json(b, partner_name.as_deref())
    }

    /// Overall world stats summary.
    pub fn stats(&self) -> String {
        let w = &self.world;
        let alive = w.bots.iter().filter(|b| b.alive).count();
        // Single pass tile census — cheaper than N filters and keeps lib.rs
        // cheap to call every frame.
        let mut berries = 0u32;
        let mut homes = 0u32;
        let mut saplings = 0u32;
        let mut trees = 0u32;
        let mut mushrooms = 0u32;
        let mut fires = 0u32;
        let mut logs = 0u32;
        let mut stones = 0u32;
        let mut cooked = 0u32;
        let mut paths = 0u32;
        let mut puddles = 0u32;
        let mut ashes = 0u32;
        let mut graves = 0u32;
        let mut shrines = 0u32;
        let mut fields = 0u32;
        let mut fish_tiles = 0u32;
        let mut cooked_fish = 0u32;
        for v in w.tiles.iter() {
            match *v {
                x if x == Tile::Berry as u8 => berries += 1,
                x if x == Tile::Home as u8 => homes += 1,
                x if x == Tile::Sapling as u8 => saplings += 1,
                x if x == Tile::Tree as u8 => trees += 1,
                x if x == Tile::Mushroom as u8 => mushrooms += 1,
                x if x == Tile::Fire as u8 => fires += 1,
                x if x == Tile::Log as u8 => logs += 1,
                x if x == Tile::Stone as u8 => stones += 1,
                x if x == Tile::CookedBerry as u8 => cooked += 1,
                x if x == Tile::Path as u8 => paths += 1,
                x if x == Tile::Puddle as u8 => puddles += 1,
                x if x == Tile::Ash as u8 => ashes += 1,
                x if x == Tile::Grave as u8 => graves += 1,
                x if x == Tile::Shrine as u8 => shrines += 1,
                x if x == Tile::Field as u8 => fields += 1,
                x if x == Tile::Fish as u8 => fish_tiles += 1,
                x if x == Tile::CookedFish as u8 => cooked_fish += 1,
                _ => {}
            }
        }
        let chatting = w.bots.iter().filter(|b| b.alive && b.chatting_with.is_some()).count() / 2;
        let toolmakers = w
            .bots
            .iter()
            .filter(|b| b.alive && b.job == Job::Toolmaker)
            .count();
        let cooks = w
            .bots
            .iter()
            .filter(|b| b.alive && b.job == Job::Cook)
            .count();
        let diggers = w
            .bots
            .iter()
            .filter(|b| b.alive && b.job == Job::Digger)
            .count();
        let healers = w
            .bots
            .iter()
            .filter(|b| b.alive && b.job == Job::Healer)
            .count();
        let fishermen = w
            .bots
            .iter()
            .filter(|b| b.alive && b.job == bot::Job::Fisherman)
            .count();
        let axes = w
            .bots
            .iter()
            .filter(|b| b.alive && b.has_tool > 0)
            .count();
        let hauling = w
            .bots
            .iter()
            .filter(|b| b.alive && b.carrying != bot::Carry::None)
            .count();
        let complaints = w.tree_complaints.len();
        let (avg_mood, avg_thirst, avg_stress, avg_warmth) = if alive > 0 {
            let mut m = 0.0; let mut t = 0.0; let mut s = 0.0; let mut w_ = 0.0;
            for b in w.bots.iter().filter(|b| b.alive) {
                m += b.mood; t += b.thirst; s += b.stress; w_ += b.warmth;
            }
            let n = alive as f32;
            (m / n, t / n, s / n, w_ / n)
        } else {
            (0.0, 0.0, 0.0, 0.0)
        };
        let weather = match w.weather {
            Weather::Clear => "clear",
            Weather::Raining => "rain",
            Weather::Clearing => "clearing",
        };
        let night = w.is_night();
        format!(
            "{{\"tick\":{},\"bots\":{},\"berries\":{},\"homes\":{},\"saplings\":{},\"trees\":{},\"mushrooms\":{},\"fires\":{},\
             \"logs\":{},\"stones\":{},\"cooked\":{},\"paths\":{},\"puddles\":{},\"ashes\":{},\"graves\":{},\"shrines\":{},\
             \"fields\":{},\"fish_tiles\":{},\"cooked_fish\":{},\
             \"chatting\":{},\"toolmakers\":{},\"cooks\":{},\"diggers\":{},\"healers\":{},\"fishermen\":{},\"axes\":{},\"hauling\":{},\"complaints\":{},\
             \"weather\":\"{}\",\"night\":{},\
             \"logs_chopped_total\":{},\"berries_cooked_total\":{},\
             \"avg_mood\":{:.1},\"avg_thirst\":{:.1},\"avg_stress\":{:.1},\"avg_warmth\":{:.1}}}",
            w.tick, alive, berries, homes, saplings, trees, mushrooms, fires,
            logs, stones, cooked, paths, puddles, ashes, graves, shrines,
            fields, fish_tiles, cooked_fish,
            chatting, toolmakers, cooks, diggers, healers, fishermen, axes, hauling, complaints,
            weather, night,
            w.logs_chopped_total, w.berries_cooked_total,
            avg_mood, avg_thirst, avg_stress, avg_warmth
        )
    }

    /// List of all bots (id, name, x, y) as JSON.
    pub fn bots_summary(&self) -> String {
        let mut s = String::from("[");
        let mut first = true;
        for b in &self.world.bots {
            if !b.alive {
                continue;
            }
            if !first {
                s.push(',');
            }
            first = false;
            // Pick the most "current" text: the frozen bubble text if one is
            // on-screen (stable), else the live inner thought. Empty string
            // means the row will just omit the snippet.
            let thought = if !b.bubble_text.is_empty() {
                &b.bubble_text
            } else {
                &b.thought
            };
            s.push_str(&format!(
                "{{\"id\":{},\"name\":\"{}\",\"x\":{},\"y\":{},\"goal\":\"{}\",\"trait\":\"{}\",\"job\":\"{}\",\"thought\":\"{}\"}}",
                b.id, js_escape(&b.name), b.x, b.y,
                b.goal.label(), b.traits.dominant(), b.job.label(), js_escape(thought)
            ));
        }
        s.push(']');
        s
    }

    /// Thought bubbles to render over bots — only those with an active thought_ttl or the selected bot.
    /// Capped at 6 concurrent bubbles (plus selected) so they don't overlap into soup.
    pub fn bubbles(&self) -> String {
        let selected = self.world.selected_bot;
        // Gather candidates with active TTL, sorted by freshness (highest TTL first).
        let mut active: Vec<usize> = self
            .world
            .bots
            .iter()
            .enumerate()
            .filter(|(_, b)| b.alive && b.thought_ttl > 0)
            .map(|(i, _)| i)
            .collect();
        active.sort_by_key(|i| std::cmp::Reverse(self.world.bots[*i].thought_ttl));
        active.truncate(3);

        // Always include the selected bot if any.
        if let Some(s_idx) = selected {
            if !active.contains(&s_idx) && s_idx < self.world.bots.len() && self.world.bots[s_idx].alive {
                active.push(s_idx);
            }
        }

        let mut s = String::from("[");
        let mut first = true;
        for i in active {
            let b = &self.world.bots[i];
            let is_selected = selected == Some(i);
            if !first {
                s.push(',');
            }
            first = false;
            let ttl = if is_selected { 90 } else { b.thought_ttl };
            // Prefer the frozen bubble_text (snapshotted when the bubble surfaced);
            // fall back to the live thought for the selected bot if no bubble
            // has surfaced for them yet.
            let bubble = if !b.bubble_text.is_empty() {
                &b.bubble_text
            } else {
                &b.thought
            };
            s.push_str(&format!(
                "{{\"id\":{},\"name\":\"{}\",\"x\":{:.2},\"y\":{:.2},\"thought\":\"{}\",\"ttl\":{},\"job\":\"{}\",\"selected\":{},\"mood\":{:.0}}}",
                b.id,
                js_escape(&b.name),
                b.visual_x,
                b.visual_y,
                js_escape(bubble),
                ttl,
                b.job.label(),
                is_selected,
                b.mood,
            ));
        }
        s.push(']');
        s
    }

    pub fn event_log(&self) -> String {
        let mut s = String::from("[");
        let mut first = true;
        let start = if self.world.event_log.len() > 20 {
            self.world.event_log.len() - 20
        } else {
            0
        };
        for e in &self.world.event_log[start..] {
            if !first {
                s.push(',');
            }
            first = false;
            s.push('"');
            s.push_str(&js_escape(e));
            s.push('"');
        }
        s.push(']');
        s
    }

    pub fn select_by_id(&mut self, id: u32) -> bool {
        for (i, b) in self.world.bots.iter().enumerate() {
            if b.alive && b.id == id {
                self.world.selected_bot = Some(i);
                return true;
            }
        }
        false
    }
}

fn goal_emoji(g: Goal) -> &'static str {
    match g {
        Goal::Eat => "\u{1F347}",       // 🍇
        Goal::Rest => "\u{1F4A4}",      // 💤
        Goal::Socialize => "\u{1F44B}", // 👋
        Goal::Explore => "\u{1F9ED}",   // 🧭
        Goal::Forage => "\u{1F33F}",    // 🌿
        Goal::Build => "\u{1F3E0}",     // 🏠
        Goal::Flee => "\u{1F6A8}",      // 🚨
        Goal::Visit => "\u{1F463}",     // 👣
        Goal::Craft => "\u{2692}\u{FE0F}",  // ⚒️
        Goal::Chop => "\u{1FA93}",      // 🪓
        Goal::Drink => "\u{1F4A7}",     // 💧
        Goal::Cook => "\u{1F373}",      // 🍳
        Goal::Gather => "\u{1F4E6}",    // 📦
        Goal::Deliver => "\u{1F4E8}",   // 📨
        Goal::Warm => "\u{1F525}",      // 🔥
        Goal::Mourn => "\u{1F56F}\u{FE0F}", // 🕯️
        Goal::Heal => "\u{1F49A}",      // 💚
        Goal::Fish => "\u{1F3A3}",      // 🎣
        Goal::Farm => "\u{1F33E}",      // 🌾
        Goal::Idle => "\u{1F6B6}",      // 🚶
    }
}

fn bot_to_json(b: &Bot, partner_name: Option<&str>) -> String {
    let mem: Vec<String> = b
        .memory
        .iter()
        .map(|m| {
            let kind = match m.kind {
                MemKind::Food => "food".to_string(),
                MemKind::Home => "home".to_string(),
                MemKind::Friend(id) => format!("friend:{}", id),
                MemKind::Enemy(id) => format!("enemy:{}", id),
                MemKind::Water => "water".to_string(),
                MemKind::Fire => "fire".to_string(),
                MemKind::Log => "log".to_string(),
                MemKind::Stone => "stone".to_string(),
                MemKind::Grave => "grave".to_string(),
            };
            format!(
                "{{\"kind\":\"{}\",\"x\":{},\"y\":{},\"tick\":{}}}",
                kind, m.x, m.y, m.tick
            )
        })
        .collect();
    let thoughts: Vec<String> = b
        .recent_thoughts
        .iter()
        .map(|t| format!("\"{}\"", js_escape(t)))
        .collect();
    let relations: Vec<String> = b
        .relationships
        .iter()
        .map(|(k, v)| format!("{{\"id\":{},\"affinity\":{}}}", k, v))
        .collect();

    let chatting_json = match (b.chatting_with, partner_name) {
        (Some(pid), Some(name)) => format!(
            "{{\"id\":{},\"name\":\"{}\",\"ticks\":{}}}",
            pid,
            js_escape(name),
            b.chat_ticks
        ),
        (Some(pid), None) => format!("{{\"id\":{},\"name\":\"\",\"ticks\":{}}}", pid, b.chat_ticks),
        _ => "null".to_string(),
    };
    let home_name_json = match &b.home_name {
        Some(n) => format!("\"{}\"", js_escape(n)),
        None => "null".to_string(),
    };

    let carrying_json = format!(
        "{{\"kind\":\"{}\",\"color\":[{},{},{}]}}",
        b.carrying.label(),
        b.carrying.color()[0],
        b.carrying.color()[1],
        b.carrying.color()[2],
    );

    format!(
        "{{\"id\":{},\"name\":\"{}\",\"x\":{},\"y\":{},\"color\":[{},{},{}],\
         \"energy\":{:.1},\"hunger\":{:.1},\"social\":{:.1},\"boredom\":{:.1},\"mood\":{:.1},\
         \"thirst\":{:.1},\"stress\":{:.1},\"warmth\":{:.1},\
         \"traits\":{{\"curiosity\":{:.2},\"sociability\":{:.2},\"aggression\":{:.2},\"industriousness\":{:.2},\"bravery\":{:.2}}},\
         \"dominant\":\"{}\",\"job\":\"{}\",\"speed\":{},\"goal\":\"{}\",\"thought\":\"{}\",\
         \"home\":{},\"home_name\":{},\"age\":{},\
         \"chatting\":{},\"gifts_given\":{},\"gifts_received\":{},\
         \"has_tool\":{},\"trees_chopped\":{},\
         \"carrying\":{},\"deliveries\":{},\"reputation\":{},\"berries_cooked\":{},\
         \"memory\":[{}],\"recent\":[{}],\"relations\":[{}]}}",
        b.id,
        js_escape(&b.name),
        b.x,
        b.y,
        b.color[0],
        b.color[1],
        b.color[2],
        b.energy,
        b.hunger,
        b.social,
        b.boredom,
        b.mood,
        b.thirst,
        b.stress,
        b.warmth,
        b.traits.curiosity,
        b.traits.sociability,
        b.traits.aggression,
        b.traits.industriousness,
        b.traits.bravery,
        b.traits.dominant(),
        b.job.label(),
        b.speed,
        b.goal.label(),
        js_escape(&b.thought),
        match b.home {
            Some((x, y)) => format!("[{},{}]", x, y),
            None => "null".to_string(),
        },
        home_name_json,
        b.age,
        chatting_json,
        b.gifts_given,
        b.gifts_received,
        b.has_tool,
        b.trees_chopped,
        carrying_json,
        b.deliveries,
        b.reputation,
        b.berries_cooked,
        mem.join(","),
        thoughts.join(","),
        relations.join(",")
    )
}

fn js_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(c),
        }
    }
    out
}
