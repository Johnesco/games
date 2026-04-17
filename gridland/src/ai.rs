use crate::bot::{Bot, Carry, Goal, Job, MemKind};
use crate::world::{Tile, Weather, World, H, W, FIRE_LOG_FUEL};

/// Radius within which a bot perceives tiles and other bots.
const SIGHT: i32 = 5;

/// How long a bubble stays visible (in ticks). With 2× speed ~= 120 ticks/sec,
/// 600 ticks is about 5 seconds on screen — long enough for observers to read.
const BUBBLE_TTL: u32 = 600;

/// Minimum ticks between any two bubbles anywhere in the world.
/// Keeps the pace calm: at most ~1 bubble per 1.3 seconds at 2× speed.
const BUBBLE_COOLDOWN: u64 = 160;

pub fn think_and_act(world: &mut World, idx: usize, snap: &[(i32, i32, bool, u32)]) {
    drain_drives(world, idx);
    perceive(world, idx, snap);
    forget_stale(world, idx);

    // --- Emergency survival override ---
    // A bot at hunger ≥ 90 or thirst ≥ 90 standing on food/water MUST act
    // NOW, even if it's carrying something. Without this, "delivering" bots
    // walk past life-saving tiles and die. Drop the cargo, eat/drink, live.
    emergency_drop_cargo(world, idx);

    // Act on current tile first (eat a berry if standing on one)
    try_interact(world, idx);
    try_drink(world, idx);
    try_mourn(world, idx);

    // Toolmakers craft at rocks and chop complaint trees they stand beside.
    try_craft_or_chop(world, idx);

    // Cooks upgrade berries at fires.
    try_cook_nearby(world, idx);

    // Fishermen catch fish at water edges.
    try_fish(world, idx);

    // Farmers till fields near water.
    try_farm(world, idx);

    // Hauling: pick up, possibly deliver on arrival.
    try_pickup(world, idx);
    try_deliver(world, idx);

    // Social moments — may enter/continue/end a conversation, possibly
    // hand a berry to a neighbour. These can gate movement.
    update_conversation(world, idx, snap);
    try_greet(world, idx, snap);
    try_gift(world, idx, snap);

    // Choose a goal; keep hysteresis so we don't flip every tick.
    // A genuine pivot triggers a change-of-motivation announcement and
    // freezes movement for ~2.5s so the viewer sees the bubble BEFORE the
    // bot acts on it.
    let new_goal = choose_goal(world, idx, snap);
    {
        let bot = &mut world.bots[idx];
        if new_goal != bot.goal {
            let seed = (world.tick as u32).wrapping_add(bot.id);
            let has_home = bot.home.is_some();
            let line = goal_change_declaration(bot.job, new_goal, has_home, seed);
            bot.goal = new_goal;
            bot.goal_ticks = 0;
            bot.target = None;
            bot.commitment_delay = 500; // ~4s of contemplation at 2× speed — bubble shows before action
            bot.announce_now(line);
        } else {
            bot.goal_ticks = bot.goal_ticks.saturating_add(1);
        }
    }

    pick_target(world, idx, snap);

    // Keep the inspector fed with fresh rotating thoughts (no bubble on its own).
    refresh_inner_thought(world, idx);
    update_mood(world, idx);

    // Movement — frozen during a commitment delay, or while actively chatting,
    // so the social scene holds together for the viewer.
    let chatting = world.bots[idx].chatting_with.is_some();
    if world.bots[idx].commitment_delay > 0 {
        world.bots[idx].commitment_delay -= 1;
    } else if chatting && world.bots[idx].chat_ticks < 60 {
        // Still settled in the conversation; skip movement this tick.
    } else if world.bots[idx].move_cooldown == 0 {
        step_toward_target(world, idx);
        world.bots[idx].move_cooldown = world.bots[idx].speed;
    } else {
        world.bots[idx].move_cooldown = world.bots[idx].move_cooldown.saturating_sub(1);
    }

    // Smooth visual position — lerp toward grid position each tick.
    // At 15% per tick a move visually finishes in ~12 ticks (~0.1s at 2×),
    // giving a gentle glide. At 8× speed the 8-ticks-per-frame makes it
    // effectively instant. Snap if far away (teleport/spawn).
    {
        let bot = &mut world.bots[idx];
        let tx = bot.x as f32;
        let ty = bot.y as f32;
        let dist = (tx - bot.visual_x).abs() + (ty - bot.visual_y).abs();
        if dist > 3.0 {
            // Teleport — snap visual to grid immediately
            bot.visual_x = tx;
            bot.visual_y = ty;
        } else {
            bot.visual_x += (tx - bot.visual_x) * 0.15;
            bot.visual_y += (ty - bot.visual_y) * 0.15;
        }
    }

    // Resolve pending bubble against the global cooldown (priority bypass applies).
    try_surface_bubble(world, idx);

    // Tick the thought-bubble lifetime and cooldowns
    let bot = &mut world.bots[idx];
    if bot.thought_ttl > 0 {
        bot.thought_ttl -= 1;
    }
    if bot.gift_cooldown > 0 {
        bot.gift_cooldown -= 1;
    }
    if bot.chat_cooldown > 0 {
        bot.chat_cooldown -= 1;
    }
    if bot.greet_cooldown > 0 {
        bot.greet_cooldown -= 1;
    }
}

fn try_surface_bubble(world: &mut World, idx: usize) {
    if !world.bots[idx].pending_bubble {
        return;
    }
    let high = world.bots[idx].pending_priority_high;
    if !high {
        let since = world.tick.saturating_sub(world.last_bubble_tick);
        if since < BUBBLE_COOLDOWN {
            // Drop this low-priority request — the world is already busy.
            world.bots[idx].pending_bubble = false;
            return;
        }
    }
    world.bots[idx].surface(BUBBLE_TTL);
    world.bots[idx].pending_priority_high = false;
    world.last_bubble_tick = world.tick;
}

fn drain_drives(world: &mut World, idx: usize) {
    let (bx, by) = (world.bots[idx].x, world.bots[idx].y);
    let here = world.tile(bx, by);
    let on_home = matches!(here, Tile::Home);
    let on_fire = matches!(here, Tile::Fire);
    let on_path = matches!(here, Tile::Path);
    let on_shrine = matches!(here, Tile::Shrine);
    let night = world.is_night();
    let weather = world.weather;
    // Adjacent-fire detection — warms without being on top of it.
    let mut near_fire = false;
    for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (-1, -1), (1, -1), (-1, 1)] {
        if matches!(world.tile(bx + dx, by + dy), Tile::Fire) {
            near_fire = true;
            break;
        }
    }
    // Home quality: count amenities within 3 tiles BEFORE mutable borrow.
    let home_quality_mult = if on_home {
        let mut amenities = 0i32;
        for dy in -3i32..=3 {
            for dx in -3i32..=3 {
                match world.tile(bx + dx, by + dy) {
                    Tile::Fire => amenities += 2,
                    Tile::Shrine => amenities += 2,
                    Tile::Field => amenities += 1,
                    Tile::Home if (dx != 0 || dy != 0) => amenities += 1,
                    _ => {}
                }
            }
        }
        1.0 + (amenities as f32 * 0.08).min(0.8)
    } else {
        1.0
    };

    let bot = &mut world.bots[idx];
    bot.age = bot.age.wrapping_add(1);
    // Drives scale with movement. Now that legs are ~2× slower, drains are
    // also ~halved — otherwise a hungry bot can't reach food before starving.
    // Thinking (goal choice, inner thoughts, alarms) still runs every tick;
    // only the underlying needs evolve at the slower world pace.
    bot.hunger = (bot.hunger + 0.0075).min(100.0);
    bot.energy = (bot.energy - 0.005).max(0.0);
    bot.social = (bot.social + 0.0035).min(100.0);
    bot.boredom = (bot.boredom + 0.004).min(100.0);
    // Thirst drains steadily; faster when hot (day), slower at night.
    // Halved from a punishing 0.0075/tick — at 2× game speed that was a
    // full thirst bar in ~55s, which left bots no margin while they were
    // also juggling hunger, warmth, and job duties.
    let thirst_drain = if night { 0.003 } else { 0.0045 };
    bot.thirst = (bot.thirst + thirst_drain).min(100.0);

    if on_home {
        bot.energy = (bot.energy + 0.12 * home_quality_mult).min(100.0);
        bot.warmth = (bot.warmth + 0.18 * home_quality_mult).min(100.0);
        bot.stress = (bot.stress - 0.08 * home_quality_mult).max(0.0);
    }
    if on_path {
        // Familiar ground relaxes stress a little.
        bot.stress = (bot.stress - 0.01).max(0.0);
    }
    if on_shrine {
        bot.mood = (bot.mood + 0.10).min(100.0);
        bot.stress = (bot.stress - 0.05).max(0.0);
    }

    // Warmth: cold at night & during rain, warm near fire.
    // Drains are small-per-tick. At ~120 ticks/sec game-speed a full night
    // of -0.01 gives -12 warmth over 1200 ticks, which is survivable without
    // a fire but makes you want to find one.
    let night_chill = if night { -0.010 } else { -0.002 };
    let rain_chill = if matches!(weather, Weather::Raining) { -0.012 } else { 0.0 };
    bot.warmth = (bot.warmth + night_chill + rain_chill).clamp(0.0, 100.0);
    if on_fire || near_fire {
        bot.warmth = (bot.warmth + 0.30).min(100.0);
    }

    // Carrying is tiring — hauling drains energy faster.
    if bot.carrying != Carry::None {
        bot.energy = (bot.energy - 0.004).max(0.0);
        bot.carry_ticks = bot.carry_ticks.saturating_add(1);
    }

    // Stress — compound drive. Rises from compound distress, falls with calm.
    // A small baseline decay ensures bots can recover stress over time even
    // when life is rough; without it a hungry-thirsty-cold triple produced
    // a runaway spiral that killed the population every long run.
    let stress_delta = {
        let hungry = if bot.hunger > 70.0 { 0.01 } else { 0.0 };
        let thirsty = if bot.thirst > 70.0 { 0.01 } else { 0.0 };
        let tired = if bot.energy < 20.0 { 0.01 } else { 0.0 };
        let cold = if bot.warmth < 30.0 { 0.010 } else { 0.0 };
        let low_mood = if bot.mood < -30.0 { 0.003 } else { 0.0 };
        let calm_home = if on_home { -0.03 } else { 0.0 };
        let calm_chat = if bot.chatting_with.is_some() { -0.01 } else { 0.0 };
        let baseline = -0.003;
        hungry + thirsty + tired + cold + low_mood + calm_home + calm_chat + baseline
    };
    bot.stress = (bot.stress + stress_delta).clamp(0.0, 100.0);

    if bot.hunger > 70.0 {
        bot.mood -= 0.01;
    }
    if bot.energy < 20.0 {
        bot.mood -= 0.01;
    }
    if bot.thirst > 70.0 {
        bot.mood -= 0.01;
    }
    if bot.warmth < 20.0 {
        bot.mood -= 0.015;
    }
    if bot.hunger >= 100.0 && bot.energy <= 0.0 {
        bot.mood -= 0.04;
    }

    // --- Mood contagion ---
    // Every 30 ticks, scan nearby bots. Happy neighbours lift your mood;
    // stressed/miserable ones drag it down. This makes fire-gatherings and
    // neighborhoods self-reinforcing: a cluster of content bots stays content,
    // but one deeply unhappy bot can sour a whole camp.
    if world.tick % 30 == 0 {
        let my_mood = world.bots[idx].mood;
        let mut mood_sum = 0.0f32;
        let mut count = 0u32;
        for j in 0..world.bots.len() {
            if j == idx || !world.bots[j].alive { continue; }
            let dist = (world.bots[j].x - bx).abs() + (world.bots[j].y - by).abs();
            if dist <= 3 {
                mood_sum += world.bots[j].mood;
                count += 1;
            }
        }
        if count > 0 {
            let avg_nearby = mood_sum / count as f32;
            // Pull toward nearby average, scaled by sociability.
            let pull = (avg_nearby - my_mood) * 0.003
                * world.bots[idx].traits.sociability;
            world.bots[idx].mood = (world.bots[idx].mood + pull).clamp(-100.0, 100.0);
        }
    }

    // --- Fire community bonus ---
    // Multiple bots around the same fire = communal warmth. Each additional
    // bot gives a mood tick. This creates emergent gathering behavior.
    if on_fire || near_fire {
        let mut fire_crowd = 0u32;
        for j in 0..world.bots.len() {
            if j == idx || !world.bots[j].alive { continue; }
            let dist = (world.bots[j].x - bx).abs() + (world.bots[j].y - by).abs();
            if dist <= 2 { fire_crowd += 1; }
        }
        if fire_crowd > 0 {
            let bonus = (fire_crowd as f32 * 0.04).min(0.15);
            world.bots[idx].mood = (world.bots[idx].mood + bonus).min(100.0);
            world.bots[idx].social = (world.bots[idx].social - 0.02 * fire_crowd as f32).max(0.0);
        }
    }

    // --- Skill-through-use ---
    // Experienced bots work faster. Every 50 items cooked/chopped/delivered
    // shaves ~10% off cook time (handled in try_cook_nearby via craft_progress).
    // Here we give a small mood/boredom benefit for doing your job well.
    let bot = &mut world.bots[idx];
    let experience = bot.berries_cooked.saturating_add(bot.trees_chopped)
        .saturating_add(bot.deliveries as u32) as f32;
    if experience > 10.0 {
        // Experienced bots are slightly less bored — they find purpose.
        bot.boredom = (bot.boredom - 0.001).max(0.0);
    }

    // --- Reputation social effect ---
    // High-rep bots radiate calm (they're trusted community members).
    // Low-rep bots feel isolated and drift toward stress.
    let rep = bot.reputation as f32;
    if rep > 20.0 {
        bot.stress = (bot.stress - 0.002).max(0.0);
        bot.social = (bot.social - 0.001).max(0.0); // they feel connected
    } else if rep < -10.0 {
        bot.stress = (bot.stress + 0.002).min(100.0);
    }

    // Drive-crossing alarms — each fires once per "episode" until drive recovers.
    if bot.hunger >= 85.0 && !bot.hunger_alarm {
        bot.hunger_alarm = true;
        bot.announce("Starving. I need food now.".to_string());
    } else if bot.hunger < 60.0 {
        bot.hunger_alarm = false;
    }
    if bot.energy <= 18.0 && !bot.tired_alarm {
        bot.tired_alarm = true;
        bot.announce("I'm running out of steam.".to_string());
    } else if bot.energy > 45.0 {
        bot.tired_alarm = false;
    }
    if bot.thirst >= 85.0 && !bot.thirst_alarm {
        bot.thirst_alarm = true;
        bot.announce("My throat is dust. I need water.".to_string());
    } else if bot.thirst < 55.0 {
        bot.thirst_alarm = false;
    }
    if bot.stress >= 75.0 && !bot.stress_alarm {
        bot.stress_alarm = true;
        bot.announce("I can't keep it together much longer.".to_string());
    } else if bot.stress < 40.0 {
        bot.stress_alarm = false;
    }
}

fn perceive(world: &mut World, idx: usize, snap: &[(i32, i32, bool, u32)]) {
    let (bx, by) = { let b = &world.bots[idx]; (b.x, b.y) };
    let tick = world.tick;

    for dy in -SIGHT..=SIGHT {
        for dx in -SIGHT..=SIGHT {
            let x = bx + dx;
            let y = by + dy;
            if x < 0 || y < 0 || x >= W as i32 || y >= H as i32 {
                continue;
            }
            let t = world.tile(x, y);
            if t.is_food() {
                world.bots[idx].remember(MemKind::Food, x, y, tick);
            }
            match t {
                Tile::Water | Tile::Puddle => {
                    world.bots[idx].remember(MemKind::Water, x, y, tick);
                }
                Tile::Fire => {
                    world.bots[idx].remember(MemKind::Fire, x, y, tick);
                }
                Tile::Log => {
                    world.bots[idx].remember(MemKind::Log, x, y, tick);
                }
                Tile::Stone => {
                    world.bots[idx].remember(MemKind::Stone, x, y, tick);
                }
                Tile::Grave => {
                    world.bots[idx].remember(MemKind::Grave, x, y, tick);
                }
                _ => {}
            }
        }
    }

    for (ox, oy, alive, oid) in snap {
        if !*alive || *oid == world.bots[idx].id {
            continue;
        }
        let dx = (*ox - bx).abs();
        let dy = (*oy - by).abs();
        if dx <= SIGHT && dy <= SIGHT {
            let affinity = *world.bots[idx].relationships.get(oid).unwrap_or(&0);
            if dx + dy <= 2 {
                let their_idx_opt = world.bots.iter().position(|b| b.id == *oid);
                if let Some(j) = their_idx_opt {
                    let my_soc = world.bots[idx].traits.sociability;
                    let their_agg = world.bots[j].traits.aggression;
                    let name_me = world.bots[idx].name.clone();
                    let name_them = world.bots[j].name.clone();
                    let delta = ((my_soc - their_agg) * 3.0) as i32;
                    let old = *world.bots[idx].relationships.get(oid).unwrap_or(&0);
                    let new_v = (old + delta).clamp(-50, 50);
                    world.bots[idx].relationships.insert(*oid, new_v);

                    // Threshold crossings are the only relationship events worth logging.
                    if old < 15 && new_v >= 15 {
                        world.log(format!("{} and {} became friends", name_me, name_them));
                    } else if old > -15 && new_v <= -15 {
                        world.log(format!("{} no longer trusts {}", name_me, name_them));
                    }

                    world.bots[idx].social = (world.bots[idx].social - 0.5).max(0.0);
                    world.bots[idx].boredom = (world.bots[idx].boredom - 0.2).max(0.0);
                }
            }
            if affinity > 10 {
                world.bots[idx].remember(MemKind::Friend(*oid), *ox, *oy, tick);
            } else if affinity < -10 {
                world.bots[idx].remember(MemKind::Enemy(*oid), *ox, *oy, tick);
            }
        }
    }
}

fn forget_stale(world: &mut World, idx: usize) {
    let tick = world.tick;
    let mut to_forget = Vec::new();
    for m in &world.bots[idx].memory {
        let tile_here = world.tile(m.x, m.y);
        match m.kind {
            MemKind::Food => {
                if !tile_here.is_food() {
                    to_forget.push((m.x, m.y));
                }
            }
            MemKind::Water => {
                if !matches!(tile_here, Tile::Water | Tile::Puddle) {
                    to_forget.push((m.x, m.y));
                }
            }
            MemKind::Fire => {
                if !matches!(tile_here, Tile::Fire) {
                    to_forget.push((m.x, m.y));
                }
            }
            MemKind::Log => {
                if tile_here != Tile::Log {
                    to_forget.push((m.x, m.y));
                }
            }
            MemKind::Stone => {
                if tile_here != Tile::Stone {
                    to_forget.push((m.x, m.y));
                }
            }
            MemKind::Grave => {
                if tile_here != Tile::Grave {
                    to_forget.push((m.x, m.y));
                }
            }
            _ => {}
        }
        if tick.saturating_sub(m.tick) > 2000 {
            to_forget.push((m.x, m.y));
        }
    }
    world.bots[idx].forget(|m| to_forget.iter().any(|(x, y)| m.x == *x && m.y == *y));
}

/// Toolmaker-specific action: craft a stone axe at a rock, or fell a
/// complaint tree when we've got one and we're standing next to it.
fn try_craft_or_chop(world: &mut World, idx: usize) {
    // Toolmakers are the specialists, but in an emergency any bot with an
    // axe can take a swing at a blocking tree (just can't craft new axes).
    let is_toolmaker = world.bots[idx].job == crate::bot::Job::Toolmaker;
    if !is_toolmaker && world.bots[idx].has_tool == 0 {
        return;
    }
    let (bx, by) = (world.bots[idx].x, world.bots[idx].y);

    // Craft: need to be adjacent to a Rock, have no tool, AND be a Toolmaker.
    // Non-toolmakers skip straight to the chop logic — they can swing an axe
    // someone left behind but can't knap a new one.
    if world.bots[idx].has_tool == 0 && is_toolmaker {
        let mut rock_adj = false;
        for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            if world.tile(bx + dx, by + dy) == Tile::Rock {
                rock_adj = true;
                break;
            }
        }
        if rock_adj {
            world.bots[idx].craft_progress = world.bots[idx].craft_progress.saturating_add(1);
            // Takes ~90 ticks of knapping — about a second of screen time at 2×.
            if world.bots[idx].craft_progress >= 90 {
                world.bots[idx].craft_progress = 0;
                world.bots[idx].has_tool = 3;
                world.bots[idx].boredom = (world.bots[idx].boredom - 8.0).max(0.0);
                world.bots[idx].mood = (world.bots[idx].mood + 4.0).min(100.0);
                // Knapping flakes produce a loose stone offcut on an
                // adjacent grass tile. Without this the Shrine chain has
                // no fuel and never fires — stones otherwise don't exist
                // on the map.
                for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                    let (nx, ny) = (bx + dx, by + dy);
                    if matches!(world.tile(nx, ny), Tile::Grass | Tile::Path) {
                        world.set_tile(nx, ny, Tile::Stone);
                        break;
                    }
                }
                let name = world.bots[idx].name.clone();
                world.log(format!("{} knapped a stone axe", name));
                let seed = (world.tick as u32).wrapping_add(world.bots[idx].id);
                let line = pick(
                    &[
                        "A good edge. Sharp and true.",
                        "Stone remembers how to cut.",
                        "There — a proper axe.",
                        "Ready to clear the path.",
                    ],
                    seed,
                );
                world.bots[idx].announce_now(line);
            }
        }
        return;
    }

    // Chop: complained tree first (the reason we walked here), otherwise
    // any adjacent tree. Without this fallback, toolmakers carry axes
    // past dense groves that are overwhelming the map but not "on the list"
    // and the forest never thins.
    let mut target_tree: Option<(i32, i32)> = None;
    let mut any_tree: Option<(i32, i32)> = None;
    for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
        let (tx, ty) = (bx + dx, by + dy);
        if world.tile(tx, ty) != Tile::Tree {
            continue;
        }
        if any_tree.is_none() {
            any_tree = Some((tx, ty));
        }
        if world.tree_complaints.iter().any(|(cx, cy)| *cx == tx && *cy == ty) {
            target_tree = Some((tx, ty));
            break;
        }
    }
    let target_tree = target_tree.or(any_tree);
    if let Some((tx, ty)) = target_tree {
        // Chop turns the tile into a Log (haulable, burnable). If the
        // bot just cleared it for path purposes, someone else can come
        // along and gather it for the fire.
        world.set_tile(tx, ty, Tile::Log);
        world.logs_chopped_total = world.logs_chopped_total.saturating_add(1);
        world.tree_complaints.retain(|(x, y)| !(*x == tx && *y == ty));
        world.bots[idx].has_tool = world.bots[idx].has_tool.saturating_sub(1);
        world.bots[idx].trees_chopped = world.bots[idx].trees_chopped.saturating_add(1);
        world.bots[idx].mood = (world.bots[idx].mood + 6.0).min(100.0);
        world.bots[idx].boredom = (world.bots[idx].boredom - 10.0).max(0.0);
        let name = world.bots[idx].name.clone();
        world.log(format!("{} cleared a tree at ({},{})", name, tx, ty));
        let seed = (world.tick as u32).wrapping_add(world.bots[idx].id);
        let line = pick(
            &[
                "Down it comes. Path open.",
                "That's one less wall.",
                "Better. Much better.",
                "A good swing. The grove thins.",
            ],
            seed,
        );
        world.bots[idx].announce_now(line);
    }
}

fn try_interact(world: &mut World, idx: usize) {
    let (bx, by) = { let b = &world.bots[idx]; (b.x, b.y) };
    let t = world.tile(bx, by);
    // A cook on mission-critical errand shouldn't eat the berry they came for.
    // Same for gather/deliver goals that were planning to haul this tile.
    // BUT — if hunger ≥ 85 survival overrides mission. A dead cook cooks nothing.
    let job = world.bots[idx].job;
    let goal = world.bots[idx].goal;
    let desperate = world.bots[idx].hunger >= 85.0;
    let reserved_pickup = !desperate
        && ((job == Job::Cook && goal == Goal::Cook)
            || matches!(goal, Goal::Gather | Goal::Deliver));
    match t {
        Tile::Berry => {
            if reserved_pickup {
                return;
            }
            if world.bots[idx].hunger > 15.0 {
                let was_starving = world.bots[idx].hunger > 80.0;
                world.bots[idx].hunger = (world.bots[idx].hunger - 55.0).max(0.0);
                world.bots[idx].mood = (world.bots[idx].mood + 10.0).min(100.0);
                world.set_tile(bx, by, Tile::Grass);
                let name = world.bots[idx].name.clone();
                if was_starving {
                    world.log(format!("{} ate just in time", name));
                    let seed = (world.tick as u32).wrapping_add(world.bots[idx].id);
                    let line = pick(
                        &[
                            "Saved. That berry saved me.",
                            "Oh thank the hills.",
                            "I can think again.",
                        ],
                        seed,
                    );
                    world.bots[idx].announce(line);
                }
            }
        }
        Tile::CookedBerry => {
            // Better than a raw berry — fuller hunger cure, mood spike.
            if world.bots[idx].hunger > 10.0 {
                world.bots[idx].hunger = (world.bots[idx].hunger - 70.0).max(0.0);
                world.bots[idx].mood = (world.bots[idx].mood + 18.0).min(100.0);
                world.bots[idx].stress = (world.bots[idx].stress - 8.0).max(0.0);
                world.set_tile(bx, by, Tile::Grass);
                let seed = (world.tick as u32).wrapping_add(world.bots[idx].id).wrapping_mul(157);
                let line = pick(
                    &[
                        "Warm and sweet. A cooked berry.",
                        "Whoever cooked this — thank you.",
                        "Properly done. Better than raw.",
                        "Real food, for once.",
                    ],
                    seed,
                );
                world.bots[idx].announce(line);
            }
        }
        Tile::Mushroom => {
            // A mushroom is less food than it is a pick-me-up. Always taken
            // when found — mild hunger cure, big energy jolt, small mood lift.
            world.bots[idx].hunger = (world.bots[idx].hunger - 15.0).max(0.0);
            world.bots[idx].energy = (world.bots[idx].energy + 30.0).min(100.0);
            world.bots[idx].mood = (world.bots[idx].mood + 3.0).min(100.0);
            world.set_tile(bx, by, Tile::Grass);
            let seed = (world.tick as u32).wrapping_add(world.bots[idx].id).wrapping_mul(91);
            let line = pick(
                &[
                    "Strange little bite. My legs buzz.",
                    "A mushroom. Curious aftertaste.",
                    "Woke me right up.",
                    "The forest shares.",
                ],
                seed,
            );
            world.bots[idx].announce(line);
        }
        Tile::Fish => {
            // Raw fish — NOT edible. Can only be picked up and cooked at fire.
            // try_pickup handles grabbing it; nothing happens here.
        }
        Tile::CookedFish => {
            // Cooked fish — the richest food source. Huge hunger cure, mood
            // boost, energy boost. The payoff for the entire fishing→cooking chain.
            if reserved_pickup {
                return;
            }
            if world.bots[idx].hunger > 5.0 {
                world.bots[idx].hunger = (world.bots[idx].hunger - 85.0).max(0.0);
                world.bots[idx].mood = (world.bots[idx].mood + 22.0).min(100.0);
                world.bots[idx].energy = (world.bots[idx].energy + 20.0).min(100.0);
                world.bots[idx].stress = (world.bots[idx].stress - 12.0).max(0.0);
                world.set_tile(bx, by, Tile::Grass);
                let seed = (world.tick as u32).wrapping_add(world.bots[idx].id).wrapping_mul(211);
                let line = pick(
                    &[
                        "Fresh fish. Nothing better.",
                        "That's a real meal. Cooked right.",
                        "The river feeds us all.",
                        "Worth every moment at the fire.",
                    ],
                    seed,
                );
                world.bots[idx].announce(line);
            }
        }
        Tile::Fire => {
            // Standing in the fire counts as warming up — steady mood gain and
            // social relief. Not consumed.
            world.bots[idx].mood = (world.bots[idx].mood + 0.6).min(100.0);
            world.bots[idx].social = (world.bots[idx].social - 0.4).max(0.0);
            world.bots[idx].boredom = (world.bots[idx].boredom - 0.2).max(0.0);
            world.bots[idx].warmth = (world.bots[idx].warmth + 0.4).min(100.0);
        }
        Tile::Puddle => {
            // Walking on a puddle is a free small drink.
            if world.bots[idx].thirst > 20.0 {
                world.bots[idx].thirst = (world.bots[idx].thirst - 10.0).max(0.0);
            }
        }
        Tile::Grave => {
            // Standing on a grave briefly saddens.
            world.bots[idx].mood = (world.bots[idx].mood - 0.2).max(-100.0);
            world.bots[idx].stress = (world.bots[idx].stress + 0.05).min(100.0);
        }
        _ => {}
    }
}

/// Drink from an adjacent water or puddle tile. Adjacency, not standing-on,
/// because Water isn't walkable. Puddles are walkable but this still works
/// as a passive top-off when passing one.
fn try_drink(world: &mut World, idx: usize) {
    if world.bots[idx].thirst < 12.0 {
        return;
    }
    let (bx, by) = (world.bots[idx].x, world.bots[idx].y);
    for (dx, dy) in [(0, 0), (1, 0), (-1, 0), (0, 1), (0, -1)] {
        let (nx, ny) = (bx + dx, by + dy);
        if matches!(world.tile(nx, ny), Tile::Water | Tile::Puddle) {
            let was_parched = world.bots[idx].thirst > 75.0;
            world.bots[idx].thirst = (world.bots[idx].thirst - 55.0).max(0.0);
            world.bots[idx].mood = (world.bots[idx].mood + 4.0).min(100.0);
            world.bots[idx].stress = (world.bots[idx].stress - 5.0).max(0.0);
            if was_parched {
                let seed = (world.tick as u32).wrapping_add(world.bots[idx].id);
                let line = pick(
                    &[
                        "Oh — cold and good.",
                        "First water since sunrise.",
                        "I needed that more than I knew.",
                    ],
                    seed,
                );
                world.bots[idx].announce(line);
            }
            return;
        }
    }
}

/// Pick up a haulable item from the tile under the bot. Only triggers when
/// the bot's goal invites it (Gather/Deliver) or the bot is a job whose
/// whole deal is hauling. Frees hands by dropping whatever we were holding
/// as Stone/Log on the ground — simple, no inventory.
fn try_pickup(world: &mut World, idx: usize) {
    if world.bots[idx].carrying != Carry::None {
        return;
    }
    let (bx, by) = (world.bots[idx].x, world.bots[idx].y);
    let t = world.tile(bx, by);
    if !t.is_haulable() {
        return;
    }
    // Only certain goals/jobs pick things up — otherwise half the bots are
    // walking around holding berries they never put down.
    // EXCEPTION: any bot in an "emergency" state (idle, high hunger, high
    // thirst, or high stress) can pick up anything — bots must be able to
    // act outside their profession when survival demands it.
    let job = world.bots[idx].job;
    let goal = world.bots[idx].goal;
    let emergency = world.bots[idx].hunger >= 70.0
        || world.bots[idx].thirst >= 70.0
        || world.bots[idx].stress >= 70.0
        || goal == Goal::Idle;
    let wants_pickup = emergency
        || matches!(goal, Goal::Gather | Goal::Deliver)
        || (job == Job::Cook && matches!(goal, Goal::Cook | Goal::Forage | Goal::Idle))
        || (matches!(job, Job::Digger | Job::Healer | Job::Builder)
            && matches!(goal, Goal::Forage | Goal::Build | Goal::Idle));
    if !wants_pickup {
        return;
    }
    let new_carry = match t {
        Tile::Berry => Carry::Berry,
        Tile::Log => Carry::Log,
        Tile::Stone => Carry::Stone,
        Tile::CookedBerry => Carry::CookedBerry,
        Tile::Mushroom => Carry::Mushroom,
        Tile::Fish => Carry::Fish,
        Tile::CookedFish => Carry::CookedFish,
        _ => return,
    };
    // Job-specific filter: cooks only pick up berries (their supply chain).
    // Fishermen only pick up fish. This prevents specialists from getting
    // sidetracked hauling logs when they should be cooking/fishing.
    if job == Job::Cook && !matches!(new_carry, Carry::Berry | Carry::CookedBerry | Carry::CookedFish) && !emergency {
        return;
    }
    if job == Job::Fisherman && !matches!(new_carry, Carry::Fish | Carry::Berry) && !emergency {
        return;
    }
    world.bots[idx].carrying = new_carry;
    world.bots[idx].carry_ticks = 0;
    world.set_tile(bx, by, Tile::Grass);
    let name = world.bots[idx].name.clone();
    world.log(format!("{} picked up a {}", name, new_carry.label()));
}

/// Drop the carried item — if we're at a fitting destination it counts as a
/// delivery (reputation bump, mood lift). Otherwise it becomes a tile again.
fn try_deliver(world: &mut World, idx: usize) {
    let c = world.bots[idx].carrying;
    if c == Carry::None {
        return;
    }
    let (bx, by) = (world.bots[idx].x, world.bots[idx].y);
    let here = world.tile(bx, by);

    // Delivery conditions by carry type:
    //   Log near fire  → fuel the fire
    //   Stone near home → shrine-building material (chance to convert a
    //                     grass neighbour into Shrine)
    //   Berry to a hungry friend → give it
    //   CookedBerry to a hungry friend or home → drop as tile
    //   Mushroom anywhere → drop (they wilt fast on hand)
    let mut delivered = false;

    if c == Carry::Log {
        // Adjacent fire? Feed it.
        for (dx, dy) in [(0, 0), (1, 0), (-1, 0), (0, 1), (0, -1)] {
            let (nx, ny) = (bx + dx, by + dy);
            if matches!(world.tile(nx, ny), Tile::Fire) {
                let cur = world.fire_fuel.get(&(nx, ny)).copied().unwrap_or(0);
                world
                    .fire_fuel
                    .insert((nx, ny), cur.saturating_add(FIRE_LOG_FUEL));
                world.bots[idx].carrying = Carry::None;
                world.bots[idx].deliveries = world.bots[idx].deliveries.saturating_add(1);
                world.bots[idx].reputation = (world.bots[idx].reputation + 1).min(200);
                world.bots[idx].mood = (world.bots[idx].mood + 4.0).min(100.0);
                let name = world.bots[idx].name.clone();
                world.log(format!("{} fed a fire", name));
                let seed = (world.tick as u32).wrapping_add(world.bots[idx].id);
                world.bots[idx].announce(pick(
                    &[
                        "Another log. Keep it going.",
                        "That should last the evening.",
                        "Warm again. Good.",
                    ],
                    seed,
                ));
                delivered = true;
                break;
            }
        }
    } else if c == Carry::Stone {
        // Adjacent home → build a Shrine on a nearby grass tile.
        let mut has_home = false;
        for (dx, dy) in [(0, 0), (1, 0), (-1, 0), (0, 1), (0, -1)] {
            if matches!(world.tile(bx + dx, by + dy), Tile::Home) {
                has_home = true;
                break;
            }
        }
        if has_home {
            for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let (nx, ny) = (bx + dx, by + dy);
                if matches!(world.tile(nx, ny), Tile::Grass | Tile::Path) {
                    world.set_tile(nx, ny, Tile::Shrine);
                    world.bots[idx].carrying = Carry::None;
                    world.bots[idx].deliveries = world.bots[idx].deliveries.saturating_add(1);
                    world.bots[idx].reputation = (world.bots[idx].reputation + 3).min(200);
                    world.bots[idx].mood = (world.bots[idx].mood + 6.0).min(100.0);
                    let name = world.bots[idx].name.clone();
                    world.log(format!("{} raised a shrine stone", name));
                    let seed = (world.tick as u32).wrapping_add(world.bots[idx].id);
                    world.bots[idx].announce(pick(
                        &[
                            "A small cairn. For remembrance.",
                            "The stone finds its place.",
                            "Others will stop here.",
                        ],
                        seed,
                    ));
                    delivered = true;
                    break;
                }
            }
        }
    }

    // Fallback: if we've carried for a long time without a proper delivery,
    // just drop what we're holding. 40 ticks was a third of a second — far
    // too short to walk across the map. 800 ticks is ~6.5s at 2× speed.
    // Cooks hanging onto an uncooked berry get a longer leash since their
    // whole job is walking berry → fire.
    let job = world.bots[idx].job;
    let threshold: u32 = if job == Job::Cook && c == Carry::Berry {
        2000
    } else {
        800
    };
    if !delivered
        && matches!(here, Tile::Grass | Tile::Path)
        && world.bots[idx].carry_ticks > threshold
    {
        // Stone fallback: if the haul timed out but we happen to be adjacent
        // to a home, still raise the shrine here — don't let the delivery
        // intent die from pathfinding fatigue.
        let mut made_shrine = false;
        if c == Carry::Stone {
            let mut near_home = false;
            for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                if matches!(world.tile(bx + dx, by + dy), Tile::Home) {
                    near_home = true;
                    break;
                }
            }
            if near_home {
                world.set_tile(bx, by, Tile::Shrine);
                world.bots[idx].carrying = Carry::None;
                world.bots[idx].deliveries = world.bots[idx].deliveries.saturating_add(1);
                world.bots[idx].reputation = (world.bots[idx].reputation + 2).min(200);
                world.bots[idx].mood = (world.bots[idx].mood + 4.0).min(100.0);
                let name = world.bots[idx].name.clone();
                world.log(format!("{} set a shrine stone", name));
                made_shrine = true;
            }
        }
        if !made_shrine {
            let drop_tile = match c {
                Carry::Berry => Tile::Berry,
                Carry::Log => Tile::Log,
                Carry::Stone => Tile::Stone,
                Carry::CookedBerry => Tile::CookedBerry,
                Carry::Mushroom => Tile::Mushroom,
                Carry::Fish => Tile::Fish,
                Carry::CookedFish => Tile::CookedFish,
                Carry::None => return,
            };
            world.set_tile(bx, by, drop_tile);
            world.bots[idx].carrying = Carry::None;
        }
    }
}

/// Cook-adjacent behaviour. Two paths for getting a CookedBerry on the map:
///   (a) Cook standing adjacent to a fire AND an adjacent Berry tile — the
///       classic "stand between them" setup.
///   (b) Cook standing adjacent to a fire while carrying a Berry — cook in
///       hand; the result replaces the carry with CookedBerry/CookedFish.
///       This is the common case because bots gather berries/fish and walk
///       to the fire with them.
///
/// Cooking is a UNIVERSAL ABILITY, not locked to the Cook job. Anyone
/// standing near a fire with a cookable item will cook it. Cooks are
/// specialists: they cook at full speed and actively seek out fires.
/// Everyone else cooks at half speed and only does it opportunistically
/// (when idle, hungry, or already warming).
///
/// Fish MUST be cooked before eating — raw fish is inedible.
/// When fish is cooking, an "aroma" draws nearby bots toward the fire
/// (small hunger-awareness nudge).
fn try_cook_nearby(world: &mut World, idx: usize) {
    let (bx, by) = (world.bots[idx].x, world.bots[idx].y);
    let job = world.bots[idx].job;
    let goal = world.bots[idx].goal;
    let is_cook = job == Job::Cook;

    // Everyone can cook. Non-cooks do it when they're near a fire for any
    // reason — warming, idle, resting, delivering, or just hungry.
    let eligible = is_cook
        || matches!(goal, Goal::Warm | Goal::Idle | Goal::Rest | Goal::Cook
                        | Goal::Deliver | Goal::Gather | Goal::Forage)
        || world.bots[idx].hunger >= 40.0;
    if !eligible {
        return;
    }

    // Non-cooks progress at half speed.
    let speed = if is_cook { 1 } else { if world.tick % 2 == 0 { 1 } else { 0 } };
    if speed == 0 {
        return;
    }

    // Must be near a fire (within 2 tiles — close enough to feel the heat).
    let mut fire_near = false;
    'outer: for dy in -2i32..=2 {
        for dx in -2i32..=2 {
            if matches!(world.tile(bx + dx, by + dy), Tile::Fire) {
                fire_near = true;
                break 'outer;
            }
        }
    }
    if !fire_near {
        return;
    }

    // (b) Cook-in-hand: carrying Berry → CookedBerry, Fish → CookedFish.
    let carry = world.bots[idx].carrying;
    let (is_cookable_carry, cooked_carry, label) = match carry {
        Carry::Berry => (true, Carry::CookedBerry, "a berry"),
        Carry::Fish => (true, Carry::CookedFish, "a fish"),
        _ => (false, Carry::None, ""),
    };
    if is_cookable_carry {
        // Key by bot ID (offset into a range that can't collide with tile coords)
        // so progress accumulates even if the bot shifts a tile while cooking.
        let key = (idx as i32 + 100000, 0);
        let prog = world.cook_progress.get(&key).copied().unwrap_or(0).saturating_add(1);
        // Short cook times — 30 ticks for berry (~0.25s at 2×), 50 for fish.
        // Cooks finish 40% faster thanks to experience.
        let base: u16 = if carry == Carry::Fish { 50 } else { 30 };
        let threshold = if is_cook { base * 6 / 10 } else { base };
        // Aroma: while fish is cooking, nearby bots feel hunger stir.
        if carry == Carry::Fish && prog % 20 == 0 {
            aroma_pulse(world, idx, bx, by);
        }
        if prog >= threshold {
            world.cook_progress.remove(&key);
            world.bots[idx].carrying = cooked_carry;
            world.berries_cooked_total = world.berries_cooked_total.saturating_add(1);
            world.bots[idx].berries_cooked = world.bots[idx].berries_cooked.saturating_add(1);
            world.bots[idx].reputation = (world.bots[idx].reputation + 2).min(200);
            world.bots[idx].mood = (world.bots[idx].mood + 4.0).min(100.0);
            world.bots[idx].boredom = (world.bots[idx].boredom - 6.0).max(0.0);
            let name = world.bots[idx].name.clone();
            world.log(format!("{} cooked {}", name, label));
            let seed = (world.tick as u32).wrapping_add(world.bots[idx].id);
            world.bots[idx].announce(pick(
                &if carry == Carry::Fish {
                    ["The fish sizzles. Almost ready.", "Smells incredible.",
                     "Golden and done.", "Fresh from the fire."]
                } else {
                    ["There — bubbling. Perfect.", "A little fire, a little patience.",
                     "Better than raw. Always better.", "Cook it slow, cook it right."]
                },
                seed,
            ));
        } else {
            world.cook_progress.insert(key, prog);
        }
        return;
    }

    // (a) Tile cook: find an adjacent Berry or Fish on the ground to cook.
    let mut cookable_pos: Option<(i32, i32, bool)> = None; // (x, y, is_fish)
    for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1), (0, 0)] {
        let (nx, ny) = (bx + dx, by + dy);
        match world.tile(nx, ny) {
            Tile::Fish => { cookable_pos = Some((nx, ny, true)); break; }
            Tile::Berry => { if cookable_pos.is_none() { cookable_pos = Some((nx, ny, false)); } }
            _ => {}
        }
    }
    let (bxp, byp, is_fish) = match cookable_pos {
        Some(p) => p,
        None => return,
    };
    let prog = world.cook_progress.get(&(bxp, byp)).copied().unwrap_or(0);
    let prog = prog.saturating_add(1);
    let base_t: u16 = if is_fish { 50 } else { 30 };
    let threshold = if is_cook { base_t * 6 / 10 } else { base_t };
    if is_fish && prog % 20 == 0 {
        aroma_pulse(world, idx, bx, by);
    }
    if prog >= threshold {
        let cooked_tile = if is_fish { Tile::CookedFish } else { Tile::CookedBerry };
        let label = if is_fish { "a fish" } else { "a berry" };
        world.set_tile(bxp, byp, cooked_tile);
        world.cook_progress.remove(&(bxp, byp));
        world.berries_cooked_total = world.berries_cooked_total.saturating_add(1);
        world.bots[idx].berries_cooked = world.bots[idx].berries_cooked.saturating_add(1);
        world.bots[idx].reputation = (world.bots[idx].reputation + 2).min(200);
        world.bots[idx].mood = (world.bots[idx].mood + 4.0).min(100.0);
        world.bots[idx].boredom = (world.bots[idx].boredom - 6.0).max(0.0);
        let name = world.bots[idx].name.clone();
        world.log(format!("{} cooked {}", name, label));
        let seed = (world.tick as u32).wrapping_add(world.bots[idx].id);
        world.bots[idx].announce(pick(
            &if is_fish {
                ["The fish sizzles. Almost ready.", "Smells incredible.",
                 "Golden and done.", "Fresh from the fire."]
            } else {
                ["There — bubbling. Perfect.", "A little fire, a little patience.",
                 "Better than raw. Always better.", "Cook it slow, cook it right."]
            },
            seed,
        ));
    } else {
        world.cook_progress.insert((bxp, byp), prog);
    }
}

/// Stressed bots gravitate to Shrines and Fires. Standing on one shaves
/// stress steadily; adjacency gives a milder effect (already handled in
/// drain_drives). This function makes sure the Shrine path-traffic bump
/// is reinforced so it doesn't erode under foot.
fn try_mourn(world: &mut World, idx: usize) {
    let (bx, by) = (world.bots[idx].x, world.bots[idx].y);
    let mut on_or_adj_grave = false;
    for (dx, dy) in [(0, 0), (1, 0), (-1, 0), (0, 1), (0, -1)] {
        if matches!(world.tile(bx + dx, by + dy), Tile::Grave) {
            on_or_adj_grave = true;
            break;
        }
    }
    if !on_or_adj_grave {
        return;
    }
    if world.bots[idx].goal != Goal::Mourn {
        return;
    }
    // Mourning — stands quietly. Each tick of mourning nudges mood up (closure),
    // stress down, reputation up (community cares about remembrance).
    world.bots[idx].mood = (world.bots[idx].mood + 0.05).min(100.0);
    world.bots[idx].stress = (world.bots[idx].stress - 0.08).max(0.0);
    if world.tick % 160 == 0 {
        world.bots[idx].reputation = (world.bots[idx].reputation + 1).min(200);
        let seed = (world.tick as u32).wrapping_add(world.bots[idx].id);
        world.bots[idx].set_thought(pick(
            &[
                "May the wind know them.",
                "A moment for the lost.",
                "We mark the ones who walked here.",
                "Rest easy, friend.",
            ],
            seed,
        ));
    }
}

/// Fisherman action: catch fish at a water edge. Must be adjacent to a Water
/// or Puddle tile with hands free (Carry::None). Takes ~60 ticks to land one.
/// Non-fishermen can try but at half speed (every other tick counted).
fn try_fish(world: &mut World, idx: usize) {
    if world.bots[idx].carrying != Carry::None {
        return;
    }
    let goal = world.bots[idx].goal;
    let is_fisher = world.bots[idx].job == Job::Fisherman;
    // Non-fishermen only fish when explicitly on a Fish goal or idle+hungry.
    if !is_fisher && !(goal == Goal::Fish || (goal == Goal::Idle && world.bots[idx].hunger >= 60.0)) {
        return;
    }
    let (bx, by) = (world.bots[idx].x, world.bots[idx].y);
    let mut water_adj = false;
    for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
        if matches!(world.tile(bx + dx, by + dy), Tile::Water | Tile::Puddle) {
            water_adj = true;
            break;
        }
    }
    if !water_adj {
        return;
    }
    // Non-fishermen are half speed.
    if !is_fisher && world.tick % 2 != 0 {
        return;
    }
    world.bots[idx].craft_progress = world.bots[idx].craft_progress.saturating_add(1);
    if world.bots[idx].craft_progress >= 60 {
        world.bots[idx].craft_progress = 0;
        // Place a fish on the ground under the bot.
        world.set_tile(bx, by, Tile::Fish);
        world.bots[idx].mood = (world.bots[idx].mood + 3.0).min(100.0);
        world.bots[idx].boredom = (world.bots[idx].boredom - 5.0).max(0.0);
        let name = world.bots[idx].name.clone();
        world.log(format!("{} caught a fish at ({},{})", name, bx, by));
        let seed = (world.tick as u32).wrapping_add(world.bots[idx].id);
        world.bots[idx].announce(pick(
            &[
                "Got one! A good catch.",
                "Patience pays. Fish on.",
                "Pulled it in. Dinner sorted.",
                "The river provides.",
            ],
            seed,
        ));
    }
}

/// Farmer action: till grass into a Field when adjacent to water, or tend
/// an existing field (minor boredom/mood benefit). Fields produce berries
/// on their own, so this is a one-time investment.
fn try_farm(world: &mut World, idx: usize) {
    let job = world.bots[idx].job;
    let goal = world.bots[idx].goal;
    // Only farmers create fields. Other bots might harvest from them but
    // don't till.
    if job != Job::Farmer && goal != Goal::Farm {
        return;
    }
    let (bx, by) = (world.bots[idx].x, world.bots[idx].y);
    let tile = world.tile(bx, by);

    // Standing on a field — tend it. Small mood tick.
    if tile == Tile::Field {
        world.bots[idx].boredom = (world.bots[idx].boredom - 0.1).max(0.0);
        return;
    }

    // Standing on grass with water adjacent → till into a field.
    if !matches!(tile, Tile::Grass) {
        return;
    }
    let mut near_water = false;
    for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (-1, 1), (1, -1), (-1, -1)] {
        if matches!(world.tile(bx + dx, by + dy), Tile::Water | Tile::Puddle) {
            near_water = true;
            break;
        }
    }
    if !near_water {
        return;
    }
    // Don't over-field: check that there isn't already a field adjacent.
    let mut field_adj = false;
    for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
        if world.tile(bx + dx, by + dy) == Tile::Field {
            field_adj = true;
            break;
        }
    }
    if field_adj {
        return;
    }
    world.set_tile(bx, by, Tile::Field);
    world.bots[idx].mood = (world.bots[idx].mood + 6.0).min(100.0);
    world.bots[idx].boredom = (world.bots[idx].boredom - 12.0).max(0.0);
    let name = world.bots[idx].name.clone();
    world.log(format!("{} tilled a field at ({},{})", name, bx, by));
    let seed = (world.tick as u32).wrapping_add(world.bots[idx].id);
    world.bots[idx].announce(pick(
        &[
            "Good earth here. It'll grow.",
            "Turned the soil. Seeds next.",
            "A field of my own.",
            "The land remembers kindness.",
        ],
        seed,
    ));
}

/// A quick passing greeting when two bots brush past each other — not a
/// full chat, just a pixel of social warmth to keep streets alive.
fn try_greet(world: &mut World, idx: usize, snap: &[(i32, i32, bool, u32)]) {
    if world.bots[idx].greet_cooldown > 0 {
        return;
    }
    if world.bots[idx].chatting_with.is_some() {
        return;
    }
    if world.bots[idx].goal == Goal::Flee {
        return;
    }
    let (bx, by) = (world.bots[idx].x, world.bots[idx].y);
    let my_id = world.bots[idx].id;
    for (ox, oy, alive, oid) in snap {
        if !*alive || *oid == my_id {
            continue;
        }
        let d = (ox - bx).abs() + (oy - by).abs();
        if d != 1 {
            continue;
        }
        let aff = *world.bots[idx].relationships.get(oid).unwrap_or(&0);
        if aff < -5 {
            continue;
        }
        // Cheap greet — nudges social, skips the chat commitment.
        world.bots[idx].social = (world.bots[idx].social - 1.2).max(0.0);
        world.bots[idx].greet_cooldown = 300;
        return;
    }
}

/// Social glue. If two bots stand adjacent and like each other well enough,
/// they settle into a chat: thoughts get relational, social pressure drops,
/// and affinity deepens. The chat ends when a partner moves away, runs out
/// of time, or a strong negative drive (starving, fleeing) takes over.
fn update_conversation(world: &mut World, idx: usize, snap: &[(i32, i32, bool, u32)]) {
    // Terminate first if state says we should.
    let mut end_reason: Option<&'static str> = None;
    if let Some(partner_id) = world.bots[idx].chatting_with {
        // Partner still alive and adjacent?
        let mut partner_pos: Option<(i32, i32, usize)> = None;
        for (i, b) in world.bots.iter().enumerate() {
            if b.id == partner_id {
                if b.alive {
                    partner_pos = Some((b.x, b.y, i));
                }
                break;
            }
        }
        let (bx, by) = (world.bots[idx].x, world.bots[idx].y);
        let still_close = partner_pos
            .map(|(px, py, _)| (px - bx).abs() + (py - by).abs() <= 1)
            .unwrap_or(false);
        let hungry_urgent = world.bots[idx].hunger > 80.0;
        let tired_urgent = world.bots[idx].energy < 15.0;
        let fleeing = world.bots[idx].goal == crate::bot::Goal::Flee;
        if !still_close {
            end_reason = Some("drifted apart");
        } else if world.bots[idx].chat_ticks > 140 {
            end_reason = Some("wound down");
        } else if hungry_urgent || tired_urgent || fleeing {
            end_reason = Some("urgent");
        }

        if end_reason.is_none() {
            // Continue the conversation: slow burn of affinity + needs relief.
            world.bots[idx].chat_ticks = world.bots[idx].chat_ticks.saturating_add(1);
            world.bots[idx].social = (world.bots[idx].social - 1.6).max(0.0);
            world.bots[idx].boredom = (world.bots[idx].boredom - 0.8).max(0.0);
            world.bots[idx].mood = (world.bots[idx].mood + 0.22).min(100.0);
            world.bots[idx].last_chat_tick = world.tick;
            // Every 30 ticks of chat, their bond deepens a touch.
            if world.bots[idx].chat_ticks % 30 == 15 {
                let old = *world.bots[idx].relationships.get(&partner_id).unwrap_or(&0);
                let new_v = (old + 2).clamp(-50, 50);
                world.bots[idx].relationships.insert(partner_id, new_v);
                // Occasionally verbalise the moment.
                if world.bots[idx].chat_ticks == 45 {
                    let their_name = partner_pos
                        .and_then(|(_, _, pi)| world.bots.get(pi).map(|b| b.name.clone()))
                        .unwrap_or_default();
                    let seed = (world.tick as u32).wrapping_add(world.bots[idx].id);
                    let line = pick(
                        &[
                            "Good to catch up.",
                            "I always forget how much I miss this.",
                            "The day turned around.",
                            "This helps more than they know.",
                        ],
                        seed,
                    );
                    let _ = their_name;
                    world.bots[idx].announce(line);
                }
            }
        }
    }

    if let Some(_reason) = end_reason {
        let my_id = world.bots[idx].id;
        let partner_id = world.bots[idx].chatting_with.unwrap();
        world.bots[idx].chatting_with = None;
        world.bots[idx].chat_ticks = 0;
        world.bots[idx].chat_cooldown = 240;
        // If our partner still sees us as their chat partner, let them cool off too.
        for b in world.bots.iter_mut() {
            if b.id == partner_id && b.chatting_with == Some(my_id) {
                b.chatting_with = None;
                b.chat_ticks = 0;
                b.chat_cooldown = 240;
                break;
            }
        }
        return;
    }

    // Look for a new conversation partner if we're not busy.
    if world.bots[idx].chatting_with.is_some() || world.bots[idx].chat_cooldown > 0 {
        return;
    }
    if world.bots[idx].commitment_delay > 0 {
        return;
    }
    if world.bots[idx].goal == crate::bot::Goal::Flee {
        return;
    }
    let (bx, by) = (world.bots[idx].x, world.bots[idx].y);
    let my_id = world.bots[idx].id;
    let my_soc = world.bots[idx].traits.sociability;
    // Shortlist candidates: adjacent, alive, not already chatting, affinity >= 0.
    let mut candidate: Option<(usize, u32, i32)> = None;
    for (ox, oy, alive, oid) in snap {
        if !*alive || *oid == my_id {
            continue;
        }
        let d = (ox - bx).abs() + (oy - by).abs();
        if d != 1 {
            continue;
        }
        let other_idx = match world.bots.iter().position(|b| b.id == *oid) {
            Some(i) => i,
            None => continue,
        };
        if world.bots[other_idx].chatting_with.is_some() {
            continue;
        }
        if world.bots[other_idx].chat_cooldown > 0 {
            continue;
        }
        let aff = *world.bots[idx].relationships.get(oid).unwrap_or(&0);
        if aff < 0 {
            continue;
        }
        let their_soc = world.bots[other_idx].traits.sociability;
        let score = aff + ((my_soc + their_soc) * 10.0) as i32;
        if candidate.map_or(true, |(_, _, s)| score > s) {
            candidate = Some((other_idx, *oid, score));
        }
    }
    if let Some((partner_idx, partner_id, score)) = candidate {
        // Probability gates on score — friendlier pairs chat more often.
        // Range: score ~ [0, 75]. Map to ~ [4, 40]% per eligible tick.
        let p = (score as f32 * 0.005 + 0.04).min(0.40);
        let rand = ((world.tick as u32)
            .wrapping_add(my_id)
            .wrapping_mul(2654435761)) as f32
            / (u32::MAX as f32);
        if rand > p {
            return;
        }
        world.bots[idx].chatting_with = Some(partner_id);
        world.bots[idx].chat_ticks = 0;
        world.bots[idx].last_chat_tick = world.tick;
        world.bots[partner_idx].chatting_with = Some(my_id);
        world.bots[partner_idx].chat_ticks = 0;
        world.bots[partner_idx].last_chat_tick = world.tick;
        let n1 = world.bots[idx].name.clone();
        let n2 = world.bots[partner_idx].name.clone();
        world.log(format!("{} and {} stopped to talk", n1, n2));
        // Only the initiator announces — keeps the bubble load symmetrical.
        let seed = (world.tick as u32).wrapping_add(my_id);
        let lines = [
            format!("Hey, {}!", n2),
            format!("Oh, it's you, {}.", n2),
            format!("{}! Just the person.", n2),
            format!("Come here, {} — a word.", n2),
        ];
        world.bots[idx].announce_now(lines[(seed as usize) % lines.len()].clone());
    }
}

/// Friendly bots sometimes hand off a berry memory to a hungrier neighbour.
/// It's cheap (forager gives up one remembered spot), warms the pair's bond,
/// and the receiver gets a hunger hit without having to walk for it.
fn try_gift(world: &mut World, idx: usize, snap: &[(i32, i32, bool, u32)]) {
    if world.bots[idx].gift_cooldown > 0 {
        return;
    }
    if world.bots[idx].hunger > 65.0 {
        return; // keep it for ourselves
    }
    // Must have a remembered food spot to share.
    let food_mem = match world.bots[idx]
        .nearest_mem(|m| matches!(m.kind, crate::bot::MemKind::Food))
    {
        Some(m) => m,
        None => return,
    };
    if world.tile(food_mem.x, food_mem.y) != Tile::Berry {
        return;
    }
    let (bx, by) = (world.bots[idx].x, world.bots[idx].y);
    let my_id = world.bots[idx].id;
    // Find an adjacent friend in hunger distress.
    for (ox, oy, alive, oid) in snap {
        if !*alive || *oid == my_id {
            continue;
        }
        let d = (ox - bx).abs() + (oy - by).abs();
        if d != 1 {
            continue;
        }
        let aff = *world.bots[idx].relationships.get(oid).unwrap_or(&0);
        if aff < 5 {
            continue;
        }
        let their_idx = match world.bots.iter().position(|b| b.id == *oid) {
            Some(i) => i,
            None => continue,
        };
        let their_hunger = world.bots[their_idx].hunger;
        if their_hunger < 45.0 {
            continue;
        }
        // Roll (rarely happens).
        let rand = ((world.tick as u32)
            .wrapping_add(my_id)
            .wrapping_mul(1597334677)) as f32
            / (u32::MAX as f32);
        if rand > 0.18 {
            continue;
        }
        // Do it. We "share" knowledge of the food location plus a boost.
        world.bots[their_idx].remember(
            crate::bot::MemKind::Food,
            food_mem.x,
            food_mem.y,
            world.tick,
        );
        world.bots[their_idx].hunger = (their_hunger - 20.0).max(0.0);
        world.bots[their_idx].mood = (world.bots[their_idx].mood + 6.0).min(100.0);
        world.bots[their_idx].gifts_received = world.bots[their_idx].gifts_received.saturating_add(1);

        // Tighten the bond both ways.
        let old_theirs = *world.bots[their_idx].relationships.get(&my_id).unwrap_or(&0);
        world.bots[their_idx]
            .relationships
            .insert(my_id, (old_theirs + 5).clamp(-50, 50));
        let old_mine = *world.bots[idx].relationships.get(oid).unwrap_or(&0);
        world.bots[idx]
            .relationships
            .insert(*oid, (old_mine + 3).clamp(-50, 50));

        world.bots[idx].mood = (world.bots[idx].mood + 3.0).min(100.0);
        world.bots[idx].gifts_given = world.bots[idx].gifts_given.saturating_add(1);
        world.bots[idx].gift_cooldown = 600;
        let n1 = world.bots[idx].name.clone();
        let n2 = world.bots[their_idx].name.clone();
        world.log(format!("{} shared a berry tip with {}", n1, n2));
        let seed = (world.tick as u32).wrapping_add(my_id);
        let templates: [&str; 4] = [
            "Here, {} — there's a berry just over there.",
            "{}, take this one. I know another.",
            "{} looks peaked. I'll point them somewhere.",
            "Have this, {}. I can wait.",
        ];
        let tmpl = templates[(seed as usize) % templates.len()];
        let spoken = tmpl.replacen("{}", &n2, 1);
        world.bots[idx].announce_now(spoken);
        break;
    }
}

fn choose_goal(world: &World, idx: usize, snap: &[(i32, i32, bool, u32)]) -> Goal {
    let bot = &world.bots[idx];
    let h = bot.hunger / 100.0;
    let e = 1.0 - bot.energy / 100.0;
    let s = bot.social / 100.0;
    let b = bot.boredom / 100.0;
    let th = bot.thirst / 100.0;
    let st = bot.stress / 100.0;
    let cold = 1.0 - bot.warmth / 100.0;

    let has_food = bot.nearest_mem(|m| matches!(m.kind, MemKind::Food)).is_some();
    let has_home = bot.home.is_some();
    let has_friend = bot.nearest_mem(|m| matches!(m.kind, MemKind::Friend(_))).is_some();
    let has_water = bot.nearest_mem(|m| matches!(m.kind, MemKind::Water)).is_some();
    let has_fire = bot.nearest_mem(|m| matches!(m.kind, MemKind::Fire)).is_some();
    let has_log = bot.nearest_mem(|m| matches!(m.kind, MemKind::Log)).is_some();
    let has_stone = bot.nearest_mem(|m| matches!(m.kind, MemKind::Stone)).is_some();
    let has_grave = bot.nearest_mem(|m| matches!(m.kind, MemKind::Grave)).is_some();
    let carrying_something = bot.carrying != Carry::None;

    let mut flee_score = 0.0f32;
    for (ox, oy, alive, oid) in snap {
        if !*alive || *oid == bot.id {
            continue;
        }
        let dist = (ox - bot.x).abs() + (oy - bot.y).abs();
        if dist <= 4 {
            let affinity = *bot.relationships.get(oid).unwrap_or(&0);
            if affinity < -15 {
                flee_score = (flee_score + (5 - dist) as f32 * 0.25).min(1.0);
            }
        }
    }

    let j = bot.job;
    let is_toolmaker = j == Job::Toolmaker;
    let complaints_exist = !world.tree_complaints.is_empty();
    let craft_score = if is_toolmaker && bot.has_tool == 0 && complaints_exist {
        1.4 + bot.traits.industriousness * 0.4
    } else {
        0.0
    };
    let chop_score = if is_toolmaker && bot.has_tool > 0 && complaints_exist {
        1.5 + bot.traits.industriousness * 0.3
    } else {
        0.0
    };
    // Delivery score — high if we're holding something, BUT collapses
    // when the bot is starving or parched. Without this, deliver (1.8) beats
    // eat/drink (1.55) and bots die carrying berries they never put down.
    let survival_crisis = h > 0.85 || th > 0.85;
    let deliver_score = if carrying_something && !survival_crisis {
        1.2 + (bot.carry_ticks as f32 / 200.0).min(0.6)
    } else if carrying_something {
        // Still want to deliver eventually, just not right now.
        0.3
    } else {
        0.0
    };
    // Gather score — jobs that haul pick things up opportunistically.
    let gather_score = if !carrying_something {
        let job_mul = match j {
            Job::Cook => 0.55,
            Job::Digger => 0.85,
            Job::Healer => 0.35,
            Job::Builder => 0.45,
            Job::Farmer => 0.30,
            _ => 0.10,
        };
        let mem_bonus = if has_log || has_stone { 0.25 } else { 0.0 };
        job_mul + mem_bonus
    } else {
        0.0
    };
    // Cook score — anyone carrying a cookable item (Fish MUST be cooked) wants
    // a fire. Cooks get the highest pull; Fish carriers get urgency because raw
    // fish is inedible and spoils their inventory.
    let carrying_cookable = matches!(bot.carrying, Carry::Berry | Carry::Fish);
    let carrying_fish = bot.carrying == Carry::Fish;
    let cook_score = if j == Job::Cook && has_fire && (has_food || carrying_cookable) {
        0.95 + bot.traits.industriousness * 0.25
    } else if carrying_fish && has_fire {
        // Fish is inedible raw — strong pull to fire
        0.85 + h * 0.3
    } else if carrying_cookable && has_fire && h > 0.4 {
        // Hungry non-cook with a berry near a fire
        0.55 + h * 0.3
    } else {
        0.0
    };
    // Warm — cold bots head to a fire.
    let warm_score = cold.powf(1.4) * 1.0 + if has_fire && cold > 0.3 { 0.3 } else { 0.0 };
    // Mourn — low mood + grave known = visit it. Also Healers do it more.
    let mourn_score = if has_grave {
        let base = (bot.mood.min(0.0).abs() / 100.0) * 0.4;
        let healer_bias = if j == Job::Healer { 0.2 } else { 0.0 };
        base + healer_bias
    } else {
        0.0
    };
    // Heal — stressed bots head to shrine/fire/home for comfort.
    let heal_score = st.powf(1.3) * 0.9;
    // Drink — simple thirst score.
    let drink_score = th.powf(1.4) * 1.3 + if has_water { 0.25 } else { 0.0 };

    let scores = [
        (Goal::Eat, h.powf(1.4) * 1.3 + if has_food { 0.25 } else { 0.0 }
            + job_bonus(Goal::Eat, j)),
        (Goal::Rest, e.powf(1.4) * 1.1 + if has_home { 0.15 } else { 0.0 }
            + job_bonus(Goal::Rest, j)),
        (Goal::Socialize, s * bot.traits.sociability * 1.0 + if has_friend { 0.15 } else { 0.0 }
            + job_bonus(Goal::Socialize, j)),
        (Goal::Explore, b * bot.traits.curiosity * 0.85
            + job_bonus(Goal::Explore, j)),
        (Goal::Forage, bot.traits.industriousness * 0.35 + h * 0.25
            + job_bonus(Goal::Forage, j)),
        (Goal::Build, bot.traits.industriousness * (1.0 - b * 0.5) * 0.55
            + if !has_home { 0.3 } else { 0.0 }
            + job_bonus(Goal::Build, j)),
        (Goal::Flee, flee_score * (1.0 - bot.traits.bravery) * 1.3
            + job_bonus(Goal::Flee, j)),
        (Goal::Craft, craft_score),
        (Goal::Chop, chop_score),
        (Goal::Drink, drink_score + job_bonus(Goal::Drink, j)),
        (Goal::Cook, cook_score),
        (Goal::Gather, gather_score + job_bonus(Goal::Gather, j)),
        (Goal::Deliver, deliver_score),
        (Goal::Warm, warm_score + job_bonus(Goal::Warm, j)),
        (Goal::Mourn, mourn_score + job_bonus(Goal::Mourn, j)),
        (Goal::Heal, heal_score + job_bonus(Goal::Heal, j)),
        (Goal::Idle, 0.18 + job_bonus(Goal::Idle, j)),
        (Goal::Fish, if j == Job::Fisherman && has_water { 0.55 } else { 0.0 }
            + job_bonus(Goal::Fish, j)),
        (Goal::Farm, if j == Job::Farmer { 0.40 } else { 0.0 }
            + job_bonus(Goal::Farm, j)),
    ];

    let mut best = scores[0];
    for s in &scores[1..] {
        let bonus = if s.0 == bot.goal && bot.goal_ticks < 40 { 0.12 } else { 0.0 };
        let adj = s.1 + bonus;
        let best_adj = best.1
            + if best.0 == bot.goal && bot.goal_ticks < 40 { 0.12 } else { 0.0 };
        if adj > best_adj {
            best = *s;
        }
    }
    best.0
}

fn job_bonus(g: Goal, j: Job) -> f32 {
    match (g, j) {
        (Goal::Eat, Job::Forager) => 0.25,
        (Goal::Forage, Job::Forager) => 0.45,
        (Goal::Build, Job::Builder) => 0.55,
        (Goal::Build, Job::Farmer) => 0.40,
        (Goal::Explore, Job::Scout) => 0.50,
        (Goal::Explore, Job::Hermit) => -0.35,
        (Goal::Socialize, Job::Socialite) => 0.50,
        (Goal::Socialize, Job::Hermit) => -0.35,
        (Goal::Rest, Job::Hermit) => 0.25,
        (Goal::Idle, Job::Hermit) => 0.20,
        (Goal::Flee, Job::Guardian) => -0.45,
        (Goal::Idle, Job::Guardian) => -0.10,
        (Goal::Forage, Job::Farmer) => 0.20,
        // Toolmakers work by themselves and socialise little; they are
        // slightly drawn to Build (as fellow craftsfolk) and away from Idle.
        (Goal::Idle, Job::Toolmaker) => -0.15,
        (Goal::Build, Job::Toolmaker) => 0.15,
        // Cooks love fire and cooking; they avoid pure exploration.
        (Goal::Cook, Job::Cook) => 0.50,
        (Goal::Warm, Job::Cook) => 0.20,
        (Goal::Explore, Job::Cook) => -0.20,
        // Diggers are hauling fanatics — stones, logs, anything heavy.
        (Goal::Gather, Job::Digger) => 0.35,
        (Goal::Deliver, Job::Digger) => 0.35,
        (Goal::Idle, Job::Digger) => -0.15,
        // Healers are drawn to distress: mourning, comforting.
        (Goal::Mourn, Job::Healer) => 0.35,
        (Goal::Heal, Job::Healer) => 0.30,
        (Goal::Socialize, Job::Healer) => 0.25,
        (Goal::Flee, Job::Healer) => -0.20,
        // Fishermen are drawn to fishing.
        (Goal::Fish, Job::Fisherman) => 0.15,
        (Goal::Idle, Job::Fisherman) => 0.05,
        // Farmers are drawn to farming.
        (Goal::Farm, Job::Farmer) => 0.20,
        _ => 0.0,
    }
}

fn pick_target(world: &mut World, idx: usize, _snap: &[(i32, i32, bool, u32)]) {
    let goal = world.bots[idx].goal;
    let (bx, by) = (world.bots[idx].x, world.bots[idx].y);

    let still_valid = if let Some((tx, ty)) = world.bots[idx].target {
        match goal {
            Goal::Eat => matches!(world.tile(tx, ty), Tile::Berry | Tile::CookedBerry | Tile::Mushroom),
            Goal::Rest => matches!(world.tile(tx, ty), Tile::Home) || world.bots[idx].home == Some((tx, ty)),
            Goal::Craft => world.tile(tx, ty) == Tile::Rock,
            Goal::Chop => world.tile(tx, ty) == Tile::Tree
                && world.tree_complaints.iter().any(|(cx, cy)| *cx == tx && *cy == ty),
            Goal::Drink => matches!(world.tile(tx, ty), Tile::Water | Tile::Puddle)
                || is_adjacent_to(world, tx, ty, |t| matches!(t, Tile::Water | Tile::Puddle)),
            Goal::Cook => matches!(world.tile(tx, ty), Tile::Fire),
            Goal::Gather => world.tile(tx, ty).is_haulable(),
            Goal::Deliver => (tx - bx).abs() + (ty - by).abs() > 0,
            Goal::Warm => matches!(world.tile(tx, ty), Tile::Fire),
            Goal::Mourn => matches!(world.tile(tx, ty), Tile::Grave),
            Goal::Heal => matches!(world.tile(tx, ty), Tile::Shrine | Tile::Fire | Tile::Home),
            Goal::Fish => is_adjacent_to(world, tx, ty, |t| matches!(t, Tile::Water | Tile::Puddle))
                || matches!(world.tile(tx, ty), Tile::Puddle),
            Goal::Farm => matches!(world.tile(tx, ty), Tile::Field | Tile::Grass),
            _ => (tx - bx).abs() + (ty - by).abs() > 0,
        }
    } else {
        false
    };
    if still_valid && world.bots[idx].goal_ticks < 80 {
        return;
    }

    let new_target = match goal {
        Goal::Eat => world
            .bots[idx]
            .nearest_mem(|m| matches!(m.kind, MemKind::Food))
            .map(|m| (m.x, m.y))
            .or_else(|| find_nearest_food(world, bx, by, 20)),
        Goal::Rest => world.bots[idx].home.or_else(|| {
            Some(random_walkable(world, bx, by, 6))
        }),
        Goal::Socialize => world
            .bots[idx]
            .nearest_mem(|m| matches!(m.kind, MemKind::Friend(_)))
            .map(|m| (m.x, m.y))
            .or_else(|| Some(random_walkable(world, bx, by, 10))),
        Goal::Explore => Some(explore_target(world, bx, by, idx)),
        Goal::Forage => Some(
            find_nearest_tile(world, bx, by, Tile::Forest, 25)
                .unwrap_or_else(|| random_walkable(world, bx, by, 12)),
        ),
        Goal::Build => build_target(world, idx),
        Goal::Flee => Some(flee_target(world, idx, _snap)),
        Goal::Visit => world.bots[idx].target,
        Goal::Craft => find_nearest_tile(world, bx, by, Tile::Rock, 30),
        Goal::Chop => nearest_complaint_tree(world, bx, by),
        Goal::Drink => world.bots[idx]
            .nearest_mem(|m| matches!(m.kind, MemKind::Water))
            .map(|m| (m.x, m.y))
            .or_else(|| find_water_edge(world, bx, by, 25)),
        Goal::Cook => {
            // Two phases of cook's journey:
            //   not carrying anything → go to a berry (pick it up on arrival)
            //   carrying a berry → go to a fire (cook in hand on arrival)
            let carry = world.bots[idx].carrying;
            if carry == Carry::Berry {
                world.bots[idx]
                    .nearest_mem(|m| matches!(m.kind, MemKind::Fire))
                    .map(|m| (m.x, m.y))
                    .or_else(|| find_nearest_tile(world, bx, by, Tile::Fire, 30))
            } else if carry == Carry::None {
                world.bots[idx]
                    .nearest_mem(|m| matches!(m.kind, MemKind::Food))
                    .map(|m| (m.x, m.y))
                    .or_else(|| find_nearest_food(world, bx, by, 25))
                    .or_else(|| find_cook_spot(world, bx, by, 30))
            } else {
                find_cook_spot(world, bx, by, 30)
                    .or_else(|| find_nearest_tile(world, bx, by, Tile::Fire, 30))
            }
        }
        Goal::Gather => find_nearest_haulable(world, bx, by, 25),
        Goal::Deliver => deliver_target(world, idx),
        Goal::Warm => world.bots[idx]
            .nearest_mem(|m| matches!(m.kind, MemKind::Fire))
            .map(|m| (m.x, m.y))
            .or_else(|| find_nearest_tile(world, bx, by, Tile::Fire, 30))
            .or(world.bots[idx].home),
        Goal::Mourn => world.bots[idx]
            .nearest_mem(|m| matches!(m.kind, MemKind::Grave))
            .map(|m| (m.x, m.y))
            .or_else(|| find_nearest_tile(world, bx, by, Tile::Grave, 25)),
        Goal::Heal => find_heal_spot(world, idx),
        Goal::Fish => world.bots[idx]
            .nearest_mem(|m| matches!(m.kind, MemKind::Water))
            .map(|m| (m.x, m.y))
            .or_else(|| find_water_edge(world, bx, by, 25)),
        Goal::Farm => find_nearest_tile(world, bx, by, Tile::Field, 25)
            .or_else(|| find_grass_near_water(world, bx, by, 20))
            .or_else(|| Some(random_walkable(world, bx, by, 8))),
        Goal::Idle => Some(random_walkable(world, bx, by, 4)),
    };

    world.bots[idx].target = new_target;
}

fn is_adjacent_to(world: &World, x: i32, y: i32, pred: impl Fn(Tile) -> bool) -> bool {
    for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
        if pred(world.tile(x + dx, y + dy)) {
            return true;
        }
    }
    false
}

/// Find an edge tile that borders water/puddle — you stand there to drink.
fn find_water_edge(world: &World, bx: i32, by: i32, radius: i32) -> Option<(i32, i32)> {
    let mut best: Option<(i32, (i32, i32))> = None;
    for dy in -radius..=radius {
        for dx in -radius..=radius {
            let x = bx + dx;
            let y = by + dy;
            if x < 0 || y < 0 || x >= W as i32 || y >= H as i32 {
                continue;
            }
            let t = world.tile(x, y);
            if t == Tile::Puddle {
                let d = dx.abs() + dy.abs();
                if best.as_ref().map_or(true, |(bd, _)| d < *bd) {
                    best = Some((d, (x, y)));
                }
            } else if t.walkable() {
                // Does any neighbour have water?
                let mut adj_water = false;
                for (ax, ay) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                    if matches!(world.tile(x + ax, y + ay), Tile::Water | Tile::Puddle) {
                        adj_water = true;
                        break;
                    }
                }
                if adj_water {
                    let d = dx.abs() + dy.abs();
                    if best.as_ref().map_or(true, |(bd, _)| d < *bd) {
                        best = Some((d, (x, y)));
                    }
                }
            }
        }
    }
    best.map(|(_, p)| p)
}

fn find_nearest_haulable(world: &World, bx: i32, by: i32, radius: i32) -> Option<(i32, i32)> {
    let mut best: Option<(i32, (i32, i32))> = None;
    for dy in -radius..=radius {
        for dx in -radius..=radius {
            let x = bx + dx;
            let y = by + dy;
            if x < 0 || y < 0 || x >= W as i32 || y >= H as i32 {
                continue;
            }
            if world.tile(x, y).is_haulable() {
                let d = dx.abs() + dy.abs();
                if best.as_ref().map_or(true, |(bd, _)| d < *bd) {
                    best = Some((d, (x, y)));
                }
            }
        }
    }
    best.map(|(_, p)| p)
}

/// Pick a drop-off destination that matches what we're carrying:
///   Log → nearest fire (fuel)
///   Stone → own home (shrine)
///   Berry/CookedBerry/Mushroom → home, or just wander toward it
fn deliver_target(world: &World, idx: usize) -> Option<(i32, i32)> {
    let bot = &world.bots[idx];
    let (bx, by) = (bot.x, bot.y);
    match bot.carrying {
        Carry::Log => find_nearest_tile(world, bx, by, Tile::Fire, 30)
            .or_else(|| bot.home),
        Carry::Stone => bot
            .home
            .or_else(|| find_nearest_tile(world, bx, by, Tile::Home, 30)),
        // A cook carrying a raw berry wants a fire, not a home. Without this
        // branch, cooks walk the berry to their house and set it down without
        // ever cooking — the whole chain is gated on this routing decision.
        // Mirror the Log case: head toward the nearest fire and let
        // pathfinding stop on an adjacent walkable tile.
        Carry::Berry if bot.job == Job::Cook => find_nearest_tile(world, bx, by, Tile::Fire, 30)
            .or_else(|| bot.home),
        Carry::Berry | Carry::CookedBerry | Carry::CookedFish | Carry::Mushroom | Carry::Fish => bot
            .home
            .or_else(|| find_nearest_tile(world, bx, by, Tile::Home, 20)),
        Carry::None => None,
    }
}

/// A good cooking stand is a walkable tile that is adjacent to both a Fire
/// and a Berry at once. Returns the tile to walk to (not the fire, not the
/// berry). Falls back to "just a fire" via the caller if nothing pairs up.
fn find_cook_spot(world: &World, bx: i32, by: i32, radius: i32) -> Option<(i32, i32)> {
    let mut best: Option<(i32, (i32, i32))> = None;
    for dy in -radius..=radius {
        for dx in -radius..=radius {
            let x = bx + dx;
            let y = by + dy;
            if x < 0 || y < 0 || x >= W as i32 || y >= H as i32 {
                continue;
            }
            if !world.tile(x, y).walkable() {
                continue;
            }
            let mut has_fire = false;
            let mut has_berry = false;
            for (ax, ay) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                match world.tile(x + ax, y + ay) {
                    Tile::Fire => has_fire = true,
                    Tile::Berry => has_berry = true,
                    _ => {}
                }
            }
            if has_fire && has_berry {
                let d = dx.abs() + dy.abs();
                if best.as_ref().map_or(true, |(bd, _)| d < *bd) {
                    best = Some((d, (x, y)));
                }
            }
        }
    }
    best.map(|(_, p)| p)
}

/// Find a grass tile adjacent to water — good spot to till into a Field.
fn find_grass_near_water(world: &World, bx: i32, by: i32, radius: i32) -> Option<(i32, i32)> {
    let mut best: Option<(i32, (i32, i32))> = None;
    for dy in -radius..=radius {
        for dx in -radius..=radius {
            let x = bx + dx;
            let y = by + dy;
            if x < 0 || y < 0 || x >= W as i32 || y >= H as i32 {
                continue;
            }
            if world.tile(x, y) != Tile::Grass {
                continue;
            }
            let mut near_water = false;
            for (ax, ay) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                if matches!(world.tile(x + ax, y + ay), Tile::Water | Tile::Puddle) {
                    near_water = true;
                    break;
                }
            }
            if near_water {
                let d = dx.abs() + dy.abs();
                if best.as_ref().map_or(true, |(bd, _)| d < *bd) {
                    best = Some((d, (x, y)));
                }
            }
        }
    }
    best.map(|(_, p)| p)
}

fn find_heal_spot(world: &World, idx: usize) -> Option<(i32, i32)> {
    let (bx, by) = (world.bots[idx].x, world.bots[idx].y);
    // Shrine first — dedicated comfort tile. Fall back to Fire, then Home.
    if let Some(s) = find_nearest_tile(world, bx, by, Tile::Shrine, 30) {
        return Some(s);
    }
    if let Some(f) = find_nearest_tile(world, bx, by, Tile::Fire, 30) {
        return Some(f);
    }
    world.bots[idx].home
}

fn nearest_complaint_tree(world: &World, bx: i32, by: i32) -> Option<(i32, i32)> {
    let mut best: Option<(i32, (i32, i32))> = None;
    for (tx, ty) in &world.tree_complaints {
        if world.tile(*tx, *ty) != Tile::Tree {
            continue;
        }
        let d = (tx - bx).abs() + (ty - by).abs();
        if best.as_ref().map_or(true, |(bd, _)| d < *bd) {
            best = Some((d, (*tx, *ty)));
        }
    }
    best.map(|(_, p)| p)
}

fn step_toward_target(world: &mut World, idx: usize) {
    let (bx, by) = (world.bots[idx].x, world.bots[idx].y);
    let target = match world.bots[idx].target {
        Some(t) => t,
        None => return,
    };

    if world.bots[idx].goal == Goal::Build {
        let dx = (target.0 - bx).abs();
        let dy = (target.1 - by).abs();
        if dx + dy <= 1 {
            perform_build(world, idx);
            world.bots[idx].target = None;
            return;
        }
    }

    if (target.0 - bx).abs() + (target.1 - by).abs() == 0 {
        world.bots[idx].target = None;
        return;
    }

    let start_dist = (target.0 - bx).abs() + (target.1 - by).abs();

    let mut best_dx = 0i32;
    let mut best_dy = 0i32;
    let mut best_score = i32::MIN;

    let choices: [(i32, i32); 5] = [(1, 0), (-1, 0), (0, 1), (0, -1), (0, 0)];
    for (dx, dy) in choices.iter() {
        let nx = bx + dx;
        let ny = by + dy;
        if nx < 0 || ny < 0 || nx >= W as i32 || ny >= H as i32 {
            continue;
        }
        let t = world.tile(nx, ny);
        if !t.walkable() {
            continue;
        }
        if (*dx != 0 || *dy != 0) && world.bot_at(nx, ny).is_some() {
            continue;
        }
        let dist = (target.0 - nx).abs() + (target.1 - ny).abs();
        // Paths give a +3 preference — bots drift onto existing roads when
        // the detour is small (within 1 tile of the optimal line). This
        // creates self-reinforcing road networks: more traffic → stronger
        // path → more bots route through → even more traffic. Sand is slow.
        let score = -dist * 10
            + match t {
                Tile::Path => 3,
                Tile::Sand => -2,
                Tile::Home => 1,
                _ => 0,
            };
        if score > best_score {
            best_score = score;
            best_dx = *dx;
            best_dy = *dy;
        }
    }

    world.bots[idx].x = (bx + best_dx).clamp(0, W as i32 - 1);
    world.bots[idx].y = (by + best_dy).clamp(0, H as i32 - 1);
    if best_dx != 0 || best_dy != 0 {
        world.bots[idx].facing = (best_dx, best_dy);
        let (nx, ny) = (world.bots[idx].x, world.bots[idx].y);
        world.mark_step(nx, ny);
    }

    // If we made no forward progress and there's a Tree sitting on the
    // direct line to the target, complain about it so a Toolmaker can
    // come fell it. Skip for toolmakers themselves (they'd be the ones
    // chopping) and for Flee (nobody cares about trees when running).
    let end_dist = (target.0 - world.bots[idx].x).abs() + (target.1 - world.bots[idx].y).abs();
    if end_dist >= start_dist
        && world.bots[idx].job != crate::bot::Job::Toolmaker
        && world.bots[idx].goal != Goal::Flee
    {
        let ddx = (target.0 - bx).signum();
        let ddy = (target.1 - by).signum();
        let mut complained = false;
        if ddx != 0 {
            let (cx, cy) = (bx + ddx, by);
            if world.tile(cx, cy) == Tile::Tree {
                world.push_complaint(cx, cy);
                complained = true;
            }
        }
        if ddy != 0 {
            let (cx, cy) = (bx, by + ddy);
            if world.tile(cx, cy) == Tile::Tree {
                world.push_complaint(cx, cy);
                complained = true;
            }
        }
        if complained {
            // Quiet inner annoyance — feeds the inspector, occasionally surfaces.
            let seed = (world.tick as u32).wrapping_add(world.bots[idx].id);
            let line = pick(
                &[
                    "A tree in the way. Again.",
                    "I can't get through here.",
                    "Someone should clear this grove.",
                    "Blocked. Always blocked by the same tree.",
                ],
                seed,
            );
            world.bots[idx].set_thought(line);
            world.bots[idx].boredom = (world.bots[idx].boredom + 0.4).min(100.0);
        }
    }
}

fn perform_build(world: &mut World, idx: usize) {
    let (tx, ty) = match world.bots[idx].target {
        Some(t) => t,
        None => return,
    };
    if !(tx >= 0 && ty >= 0 && tx < W as i32 && ty < H as i32) {
        return;
    }
    let tile = world.tile(tx, ty);
    let bot = &world.bots[idx];
    let no_home = bot.home.is_none();
    if no_home && matches!(tile, Tile::Grass | Tile::Sand) {
        world.set_tile(tx, ty, Tile::Home);
        let name = world.bots[idx].name.clone();
        world.bots[idx].home = Some((tx, ty));
        world.bots[idx].mood = (world.bots[idx].mood + 15.0).min(100.0);
        world.bots[idx].boredom = (world.bots[idx].boredom - 30.0).max(0.0);
        world.log(format!("{} built a home at ({},{})", name, tx, ty));
        // Building a first home is a big moment — always announce.
        let seed = (world.tick as u32).wrapping_add(world.bots[idx].id);
        let line = pick(
            &[
                "I built a home. A real one.",
                "These walls are mine now.",
                "Home. Finally home.",
                "The first roof is the sweetest.",
            ],
            seed,
        );
        world.bots[idx].announce(line);
    } else if matches!(tile, Tile::Grass) {
        world.set_tile(tx, ty, Tile::Sapling);
        world.bots[idx].boredom = (world.bots[idx].boredom - 10.0).max(0.0);
        // Planting is quiet — no log line (farmers do it many times a day)
        // and no automatic bubble. The goal-change announcement already said
        // "Off to plant a sapling" before this action.
    }
}

fn find_nearest_tile(world: &World, bx: i32, by: i32, target: Tile, radius: i32) -> Option<(i32, i32)> {
    let mut best: Option<(i32, (i32, i32))> = None;
    for dy in -radius..=radius {
        for dx in -radius..=radius {
            let x = bx + dx;
            let y = by + dy;
            if x < 0 || y < 0 || x >= W as i32 || y >= H as i32 {
                continue;
            }
            if world.tile(x, y) == target {
                let d = dx.abs() + dy.abs();
                if best.as_ref().map_or(true, |(bd, _)| d < *bd) {
                    best = Some((d, (x, y)));
                }
            }
        }
    }
    best.map(|(_, p)| p)
}

/// Nearest edible tile (Berry or Mushroom) within `radius`.
fn find_nearest_food(world: &World, bx: i32, by: i32, radius: i32) -> Option<(i32, i32)> {
    let mut best: Option<(i32, (i32, i32))> = None;
    for dy in -radius..=radius {
        for dx in -radius..=radius {
            let x = bx + dx;
            let y = by + dy;
            if x < 0 || y < 0 || x >= W as i32 || y >= H as i32 {
                continue;
            }
            if world.tile(x, y).is_food() {
                let d = dx.abs() + dy.abs();
                if best.as_ref().map_or(true, |(bd, _)| d < *bd) {
                    best = Some((d, (x, y)));
                }
            }
        }
    }
    best.map(|(_, p)| p)
}

fn random_walkable(world: &World, bx: i32, by: i32, radius: i32) -> (i32, i32) {
    let seed = (world.tick.wrapping_mul(31) ^ (bx as u64) ^ ((by as u64) << 16)) as i32;
    let mut k = seed.wrapping_abs();
    for _ in 0..20 {
        k = k.wrapping_mul(1103515245).wrapping_add(12345);
        let dx = (k.wrapping_shr(3)).rem_euclid(radius * 2 + 1) - radius;
        let dy = (k.wrapping_shr(11)).rem_euclid(radius * 2 + 1) - radius;
        let nx = (bx + dx).clamp(0, W as i32 - 1);
        let ny = (by + dy).clamp(0, H as i32 - 1);
        if world.tile(nx, ny).walkable() {
            return (nx, ny);
        }
    }
    (bx, by)
}

fn explore_target(world: &World, bx: i32, by: i32, idx: usize) -> (i32, i32) {
    let bot_id = world.bots[idx].id as i32;
    let quad = (bot_id + world.tick as i32 / 200) % 4;
    let (cx, cy) = match quad {
        0 => (8, 8),
        1 => (W as i32 - 8, 8),
        2 => (8, H as i32 - 8),
        _ => (W as i32 - 8, H as i32 - 8),
    };
    let drift = ((bot_id * 7 + world.tick as i32) % 13) - 6;
    let tx = (cx + drift).clamp(2, W as i32 - 3);
    let ty = (cy + drift * -1).clamp(2, H as i32 - 3);
    if world.tile(tx, ty).walkable() {
        (tx, ty)
    } else {
        random_walkable(world, bx, by, 10)
    }
}

fn build_target(world: &World, idx: usize) -> Option<(i32, i32)> {
    let bot = &world.bots[idx];
    let (bx, by) = (bot.x, bot.y);
    let is_hermit = bot.job == Job::Hermit;

    // Two-pass: first try to find a spot that forms a "neighborhood" — near
    // other homes (dist 3-5) but NOT immediately adjacent (dist ≤ 1). This
    // creates clusters with breathing room. Hermits skip the neighborhood
    // preference and just pick the first valid spot, which naturally pushes
    // them to the fringes.
    let mut best: Option<(i32, i32, i32)> = None;
    for r in 1i32..=8 {
        for dy in -r..=r {
            for dx in -r..=r {
                if dx.abs() != r && dy.abs() != r {
                    continue;
                }
                let x = bx + dx;
                let y = by + dy;
                if x < 1 || y < 1 || x >= W as i32 - 1 || y >= H as i32 - 1 {
                    continue;
                }
                if !matches!(world.tile(x, y), Tile::Grass) {
                    continue;
                }
                // Reject: adjacent to water.
                let mut adj_water = false;
                // Reject: home within Manhattan distance 1 (too crowded).
                let mut adj_home = false;
                // Count: homes within Manhattan distance 2..5 (neighborhood).
                let mut nearby_homes = 0i32;
                for nny in -2..=2i32 {
                    for nnx in -2..=2i32 {
                        let t = world.tile(x + nnx, y + nny);
                        if matches!(t, Tile::Water) && nnx.abs() <= 1 && nny.abs() <= 1 {
                            adj_water = true;
                        }
                        if matches!(t, Tile::Home) {
                            if nnx.abs() <= 1 && nny.abs() <= 1 {
                                adj_home = true;
                            } else {
                                nearby_homes += 1;
                            }
                        }
                    }
                }
                if adj_water || adj_home {
                    continue;
                }
                // Score: hermits prefer isolation, everyone else prefers neighbors.
                let score = if is_hermit {
                    -nearby_homes
                } else {
                    nearby_homes * 3 - r // favor nearby homes, penalize distance
                };
                if best.as_ref().map_or(true, |(_, _, bs)| score > *bs) {
                    best = Some((x, y, score));
                }
            }
        }
    }
    best.map(|(x, y, _)| (x, y))
}

fn flee_target(world: &World, idx: usize, snap: &[(i32, i32, bool, u32)]) -> (i32, i32) {
    let bot = &world.bots[idx];
    let mut vx = 0i32;
    let mut vy = 0i32;
    for (ox, oy, alive, oid) in snap {
        if !*alive || *oid == bot.id {
            continue;
        }
        let aff = *bot.relationships.get(oid).unwrap_or(&0);
        if aff >= 0 {
            continue;
        }
        let dx = bot.x - ox;
        let dy = bot.y - oy;
        let dist = dx.abs() + dy.abs();
        if dist <= 6 && dist > 0 {
            vx += dx * (6 - dist);
            vy += dy * (6 - dist);
        }
    }
    let tx = (bot.x + vx.signum() * 4).clamp(0, W as i32 - 1);
    let ty = (bot.y + vy.signum() * 4).clamp(0, H as i32 - 1);
    if world.tile(tx, ty).walkable() {
        (tx, ty)
    } else {
        random_walkable(world, bot.x, bot.y, 6)
    }
}

// -- Thoughts ---------------------------------------------------------------

fn refresh_inner_thought(world: &mut World, idx: usize) {
    // Thoughts rotate internally every ~40-80 ticks per bot. This feeds the
    // inspector's "Recent thoughts" list but does NOT on its own trigger a
    // thought bubble — only salient events do (see announce / surface).
    let bot = &world.bots[idx];
    let cadence: u64 = 40 + (bot.id as u64 * 11) % 40;
    let offset: u64 = (bot.id as u64 * 17) % cadence;
    if (world.tick.wrapping_add(offset)) % cadence != 0 {
        return;
    }
    let line = pick_thought(world, idx);
    world.bots[idx].set_thought(line);
}

fn pick_thought(world: &World, idx: usize) -> String {
    let bot = &world.bots[idx];
    // A bucket changes every ~24 ticks, so thought lines rotate organically.
    let bucket = (world.tick / 24) as u32;
    let seed = bot.id.wrapping_mul(2654435761).wrapping_add(bucket);

    // Conversation overrides everything — relational lines while the two
    // bots stand together.
    if let Some(partner_id) = bot.chatting_with {
        if let Some(line) = chat_thought(world, idx, partner_id, seed) {
            return line;
        }
    }

    // Occasionally peek sideways into a themed pool so bots don't only talk about their goal.
    match seed % 9 {
        0 | 1 => contextual_thought(world, idx, seed).unwrap_or_else(|| goal_thought(bot, seed)),
        2 => job_thought(bot, seed),
        3 => introspective_thought(bot, seed),
        _ => goal_thought(bot, seed),
    }
}

fn chat_thought(world: &World, idx: usize, partner_id: u32, seed: u32) -> Option<String> {
    let me = &world.bots[idx];
    let other = world.bots.iter().find(|b| b.id == partner_id && b.alive)?;
    let aff = *me.relationships.get(&partner_id).unwrap_or(&0);
    let name = &other.name;
    // Templates use the literal "{}" placeholder which we substitute with the
    // partner's name below. Keeps each line as a plain &str so all variants
    // share one formatting path.
    let lines: &[&str] = if aff >= 30 {
        &[
            "{} understands without me saying it.",
            "I could talk with {} all afternoon.",
            "Good old {}. Best company.",
            "{} lifts the day.",
        ]
    } else if aff >= 10 {
        &[
            "{} has that story again — I don't mind.",
            "Funny, how {} sees it.",
            "Nice of {} to stop by.",
            "{} always has news.",
        ]
    } else {
        &[
            "Chatting, surprisingly. Polite counts.",
            "Polite for {}'s sake.",
            "I'll hear {} out.",
            "Unexpected moment with {}.",
        ]
    };
    let tmpl = lines[(seed as usize) % lines.len()];
    Some(tmpl.replacen("{}", name, 1))
}

fn pick<'a>(lines: &'a [&'a str], seed: u32) -> String {
    lines[seed as usize % lines.len()].to_string()
}

/// When fish cooks at a fire, the aroma drifts to nearby bots. Any bot
/// within 6 tiles gets a small hunger awareness nudge (+2 hunger, which
/// makes them more likely to seek food) and remembers the fire location.
fn aroma_pulse(world: &mut World, cook_idx: usize, fx: i32, fy: i32) {
    let tick = world.tick;
    for i in 0..world.bots.len() {
        if i == cook_idx || !world.bots[i].alive { continue; }
        let dist = (world.bots[i].x - fx).abs() + (world.bots[i].y - fy).abs();
        if dist <= 6 {
            world.bots[i].hunger = (world.bots[i].hunger + 2.0).min(100.0);
            world.bots[i].remember(crate::bot::MemKind::Fire, fx, fy, tick);
            if dist <= 3 {
                world.bots[i].set_thought("Something smells good...".to_string());
            }
        }
    }
}

/// Emergency survival: if a bot is starving (≥90 hunger) while standing on
/// food, or parched (≥90 thirst) while adjacent to water, drop whatever
/// cargo they're carrying so the normal eat/drink logic can fire. Without
/// this, bots die of thirst while dutifully delivering a berry to the other
/// side of the map.
fn emergency_drop_cargo(world: &mut World, idx: usize) {
    let hunger = world.bots[idx].hunger;
    let thirst = world.bots[idx].thirst;
    let carry = world.bots[idx].carrying;
    if carry == Carry::None {
        return;
    }
    let (bx, by) = (world.bots[idx].x, world.bots[idx].y);
    let tile = world.tile(bx, by);

    let need_drop = (hunger >= 90.0 && tile.is_food())
        || (thirst >= 90.0 && {
            let mut wa = false;
            for (dx, dy) in [(0, 0), (1, 0), (-1, 0), (0, 1), (0, -1)] {
                if matches!(world.tile(bx + dx, by + dy), Tile::Water | Tile::Puddle) {
                    wa = true;
                    break;
                }
            }
            wa
        });

    if !need_drop {
        return;
    }

    let drop_tile = match carry {
        Carry::Berry => Tile::Berry,
        Carry::Log => Tile::Log,
        Carry::Stone => Tile::Stone,
        Carry::CookedBerry => Tile::CookedBerry,
        Carry::Mushroom => Tile::Mushroom,
        Carry::Fish => Tile::Fish,
        Carry::CookedFish => Tile::CookedFish,
        Carry::None => return,
    };
    // Set it down on a nearby walkable tile.
    for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
        if matches!(world.tile(bx + dx, by + dy), Tile::Grass | Tile::Path) {
            world.set_tile(bx + dx, by + dy, drop_tile);
            break;
        }
    }
    world.bots[idx].carrying = Carry::None;
}

/// Short declarative line a bot "says" the moment they pivot to a new goal.
/// These are the only lines that bypass the global bubble cooldown — they
/// take priority because a change of motivation is the most interesting
/// thing a character does.
fn goal_change_declaration(job: Job, new_goal: Goal, has_home: bool, seed: u32) -> String {
    match new_goal {
        Goal::Eat => pick(
            &["Time to find food.", "Off to eat something.",
              "I need a berry.", "Hunger wins. Food first."],
            seed,
        ),
        Goal::Rest => if has_home {
            pick(&["Heading home to rest.", "Time to lie down at home.", "Back to the roof."], seed)
        } else {
            pick(&["Need to sit down.", "I'll find a quiet spot.", "A short rest will do."], seed)
        },
        Goal::Socialize => match job {
            Job::Socialite => pick(
                &["Let me go visit someone.", "Time for a proper visit.", "Who haven't I seen today?"], seed),
            _ => pick(&["I'll go find the others.", "I need some company.", "Off to say hello."], seed),
        },
        Goal::Explore => match job {
            Job::Scout => pick(
                &["Off to map new ground.", "New territory awaits.", "Time to scout further."], seed),
            _ => pick(&["Let me see what's out there.", "Time for a walk.", "I'll head somewhere new."], seed),
        },
        Goal::Forage => match job {
            Job::Forager => pick(
                &["Off to the berry patch.", "Time to stock up.", "Straight to the forest."], seed),
            _ => pick(&["Let me find some berries.", "Off to look for food in the forest."], seed),
        },
        Goal::Build => if !has_home {
            pick(&["I'm going to build a home.", "Time to raise my walls.", "No more wandering — I'll build."], seed)
        } else {
            match job {
                Job::Farmer => pick(
                    &["Off to plant a sapling.", "Another tree for the forest.", "Time to put something in the ground."], seed),
                _ => pick(&["I'll plant a sapling.", "Let me grow something."], seed),
            }
        },
        Goal::Flee => pick(&["Getting out of here!", "Not safe. Running.", "Away! Away!"], seed),
        Goal::Visit => pick(&["Quick stop.", "Just passing by."], seed),
        Goal::Craft => pick(
            &["Off to the rocks — I'll need an axe.", "Someone needs a path. I'll make a tool.",
              "To the stone. Time to work.", "Axe first, then the grove."], seed),
        Goal::Chop => pick(
            &["I heard about a blocking tree. On my way.", "That grove won't clear itself.",
              "Axe ready. Time to swing.", "One tree between them and their path."], seed),
        Goal::Idle => pick(
            &["Nothing urgent. I'll wander.", "Taking it slow.", "Just looking around."], seed),
        Goal::Drink => pick(
            &["Off to the water.", "I need a drink.", "Parched. Going for water.", "A proper sip, then back to it."], seed),
        Goal::Cook => match job {
            Job::Cook => pick(
                &["Fire and berry — my favourite sentence.", "Time to make real food.",
                  "Into the kitchen. Metaphorically.", "A cooked berry is worth two raw."], seed),
            _ => pick(&["I'll try my hand at cooking.", "Fire's going — I'll help.", "Let me warm this berry."], seed),
        },
        Goal::Gather => match job {
            Job::Digger => pick(
                &["Something heavy wants carrying.", "Off to lift and haul.", "I'll fetch what needs fetching."], seed),
            _ => pick(&["Going to pick this up.", "Let me grab that.", "I can carry it from here."], seed),
        },
        Goal::Deliver => pick(
            &["Dropping this where it belongs.", "Carrying this on home.", "Someone will want this.", "Delivery run."], seed),
        Goal::Warm => match job {
            Job::Cook => pick(
                &["Back to the fire. Keeping the flame.", "I'll warm up — I practically live there."], seed),
            _ => pick(&["Too cold. Heading to the fire.", "Need to warm my hands.", "Chill's settling in. Fire, now."], seed),
        },
        Goal::Mourn => match job {
            Job::Healer => pick(
                &["Someone should keep vigil.", "I'll sit with the graves a while.", "Grief needs witnesses."], seed),
            _ => pick(&["A moment at the grave.", "I'll pay my respects.", "Let me go remember."], seed),
        },
        Goal::Heal => match job {
            Job::Healer => pick(
                &["I'll find somewhere quiet to mend.", "The shrine, maybe. Or the fire."], seed),
            _ => pick(&["I need a quiet place.", "Off to settle my nerves.", "Just a breath near the shrine."], seed),
        },
        Goal::Fish => pick(
            &["The water's calling.", "Time to fish.", "I'll try the shore.", "Cast a line, see what comes."], seed),
        Goal::Farm => pick(
            &["Time to work the soil.", "The field won't till itself.", "Planting season.", "Good ground waiting."], seed),
    }
}

fn goal_thought(bot: &Bot, seed: u32) -> String {
    match bot.goal {
        Goal::Eat => {
            if bot.hunger > 80.0 {
                pick(
                    &[
                        "I'm starving. Need food.",
                        "Can't think straight. Food.",
                        "My stomach is a drum now.",
                        "Berry. Any berry. Please.",
                    ],
                    seed,
                )
            } else {
                pick(
                    &[
                        "Berries sound good right now.",
                        "Snack, then on with the day.",
                        "I know exactly where the best bush is.",
                        "A little red fruit, a little happiness.",
                    ],
                    seed,
                )
            }
        }
        Goal::Rest => {
            if bot.energy < 20.0 {
                pick(
                    &[
                        "So tired... need to lie down.",
                        "Legs won't carry me further.",
                        "Sleep. Just a little sleep.",
                        "Every step is a small mountain.",
                    ],
                    seed,
                )
            } else if bot.home.is_some() {
                pick(
                    &[
                        "Heading home for a while.",
                        "Home always feels smaller than I remember.",
                        "A moment under my own roof.",
                        "Tea first. Then decisions.",
                    ],
                    seed,
                )
            } else {
                pick(
                    &[
                        "Could use a quiet spot.",
                        "Any patch of grass will do.",
                        "Catching my breath.",
                        "Just five minutes.",
                    ],
                    seed,
                )
            }
        }
        Goal::Socialize => pick(
            &[
                "It's been too quiet.",
                "Where is everyone today?",
                "I could use a friendly face.",
                "A chat would do me good.",
                "Haven't seen the others in ages.",
            ],
            seed,
        ),
        Goal::Explore => match bot.traits.dominant() {
            "curious" => pick(
                &[
                    "What's beyond that ridge?",
                    "I'm going where the map ends.",
                    "That shape on the horizon — what is it?",
                    "New ground means new questions.",
                ],
                seed,
            ),
            "brave" => pick(
                &[
                    "Onward! New ground to cover.",
                    "Nothing out here scares me.",
                    "I'll go first.",
                    "The far side, today.",
                ],
                seed,
            ),
            _ => pick(
                &[
                    "Let's see what's out there.",
                    "A walk clears the head.",
                    "I'll just wander a bit.",
                    "The wind smells different here.",
                ],
                seed,
            ),
        },
        Goal::Forage => pick(
            &[
                "There must be berries in the forest.",
                "Good pickings near old trees.",
                "Following the thickest green.",
                "I remember a patch near the water.",
                "Red jewels hidden in leaves.",
            ],
            seed,
        ),
        Goal::Build => {
            if bot.home.is_none() {
                pick(
                    &[
                        "Time to build a proper home.",
                        "I'll lay the first stones here.",
                        "Every wall begins with a choice.",
                        "Shelter first. Dreams after.",
                    ],
                    seed,
                )
            } else {
                pick(
                    &[
                        "A sapling here would do nicely.",
                        "This land could grow a tree.",
                        "Let the next one find shade.",
                        "Plant today, forget tomorrow.",
                    ],
                    seed,
                )
            }
        }
        Goal::Flee => pick(
            &[
                "Too close! Away, away!",
                "Not today. Not today.",
                "Faster, legs, faster!",
                "Get behind the rocks!",
            ],
            seed,
        ),
        Goal::Visit => pick(
            &[
                "Just a quick visit.",
                "Pass through and carry on.",
                "See how they're doing.",
                "Walk softly past.",
            ],
            seed,
        ),
        Goal::Craft => pick(
            &[
                "A good hammerstone, then a flake.",
                "Stone edges first. Everything else follows.",
                "Knap, knap, knap. Patience.",
                "The axe is half-made in my head already.",
            ],
            seed,
        ),
        Goal::Chop => pick(
            &[
                "Two swings per limb, if I'm lucky.",
                "Mind the spring-back.",
                "Let the blade do the thinking.",
                "I can almost see the clearing.",
            ],
            seed,
        ),
        Goal::Idle => {
            if bot.mood > 30.0 {
                pick(
                    &[
                        "What a lovely day.",
                        "The sky has a pleasing weight.",
                        "Life is a slow river.",
                        "Nothing to chase. How rare.",
                    ],
                    seed,
                )
            } else if bot.mood < -30.0 {
                pick(
                    &[
                        "Hmm. Feeling a bit low.",
                        "Something's off today.",
                        "Everything tastes grey.",
                        "I'll come around. Eventually.",
                    ],
                    seed,
                )
            } else {
                pick(
                    &[
                        "Just taking it easy.",
                        "Watching the grass move.",
                        "I wonder what's for later.",
                        "Shadows are longer today.",
                        "Clouds shaped like something.",
                    ],
                    seed,
                )
            }
        }
        Goal::Drink => {
            if bot.thirst > 80.0 {
                pick(
                    &[
                        "My tongue is wood.",
                        "Water, water, just water.",
                        "I'd drink rain off a stone.",
                        "The river keeps me honest.",
                    ],
                    seed,
                )
            } else {
                pick(
                    &[
                        "A sip wouldn't hurt.",
                        "Cold water, then on with it.",
                        "The puddle was pleasant yesterday.",
                        "Might catch fish at the water's edge.",
                    ],
                    seed,
                )
            }
        }
        Goal::Cook => pick(
            &[
                "Low flame, patient hand.",
                "Berries split sweeter when they're warm.",
                "A fire is a kitchen with a view.",
                "Count the bubbles — they tell the time.",
                "Stir once, wait, stir again.",
            ],
            seed,
        ),
        Goal::Gather => pick(
            &[
                "Heavier than it looks.",
                "These arms remember how.",
                "One trip or two?",
                "Stone today, logs tomorrow.",
                "Hauling makes the shoulders honest.",
            ],
            seed,
        ),
        Goal::Deliver => pick(
            &[
                "Last stretch. Don't drop it.",
                "Someone's waiting for this.",
                "A delivery is a promise, small.",
                "I can see the fire from here.",
                "Home's closer than it looks.",
            ],
            seed,
        ),
        Goal::Warm => {
            if bot.warmth < 20.0 {
                pick(
                    &[
                        "I can't feel my fingers.",
                        "Cold into the bone.",
                        "I need flame and I need it now.",
                        "The wind has teeth tonight.",
                    ],
                    seed,
                )
            } else {
                pick(
                    &[
                        "A little heat — just a little.",
                        "Chilly. The fire's the cure.",
                        "I'll hold my hands out.",
                        "Warmth is medicine.",
                    ],
                    seed,
                )
            }
        }
        Goal::Mourn => pick(
            &[
                "A name should have a stone.",
                "We remember by stopping.",
                "Small flowers would suit this spot.",
                "The wind says something I can almost hear.",
                "One breath out, one breath in. For them.",
            ],
            seed,
        ),
        Goal::Heal => {
            if bot.stress > 70.0 {
                pick(
                    &[
                        "I'm coming apart at the seams.",
                        "Too much. I need quiet.",
                        "Just a minute. Just a minute.",
                        "Steady. Steady. The shrine is close.",
                    ],
                    seed,
                )
            } else {
                pick(
                    &[
                        "Let me breathe a while.",
                        "The shrine settles me.",
                        "A pause mends more than it costs.",
                        "Fire and a seat and nothing to do.",
                    ],
                    seed,
                )
            }
        }
        Goal::Fish => pick(
            &[
                "The current is right.",
                "Patience. They'll come.",
                "A tug on the line...",
                "Still water hides a full net.",
            ],
            seed,
        ),
        Goal::Farm => pick(
            &[
                "Good earth here.",
                "The rows are coming along.",
                "Water nearby — perfect.",
                "Soil wants working.",
            ],
            seed,
        ),
    }
}

fn job_thought(bot: &Bot, seed: u32) -> String {
    let lines: &[&str] = match bot.job {
        Job::Forager => &[
            "Berries everywhere if you know where to look.",
            "This grove is past its prime.",
            "Saving one for tomorrow.",
            "Always the red ones. Always.",
            "I have a map in my head of every bush.",
        ],
        Job::Builder => &[
            "Needs a stronger frame.",
            "The roof should be a hand taller.",
            "A home is a shelter for the mind.",
            "That wall is crooked. It bothers me.",
            "Good joinery is silent joinery.",
        ],
        Job::Scout => &[
            "Every hill has a better hill behind it.",
            "Beyond the water, something I haven't seen.",
            "I should map this stretch.",
            "What's north of the ridge?",
            "The interesting thing is usually just out of sight.",
        ],
        Job::Guardian => &[
            "Stay alert.",
            "I don't recognise that one.",
            "Nothing gets past me.",
            "If there's trouble, it meets me first.",
            "The quiet ones are the ones to watch.",
        ],
        Job::Socialite => &[
            "I should check on the others.",
            "Who have I not visited today?",
            "Conversation warms the soul.",
            "Everyone's always so busy lately.",
            "A small hello goes a long way.",
        ],
        Job::Farmer => &[
            "Saplings first, fruit later.",
            "This soil looks promising.",
            "Plant now, rest when it grows.",
            "The forest remembers who planted it.",
            "Water, sun, patience.",
        ],
        Job::Hermit => &[
            "Peace is the whole point.",
            "I need less than I think I do.",
            "Silence is full of things.",
            "The crowd can have the crowd.",
            "My thoughts are better company than most.",
        ],
        Job::Toolmaker => &[
            "A tool for every grove.",
            "The right stone has the right edge.",
            "Someone's always blocked somewhere.",
            "Sharp things in kind hands.",
            "I'd trade ten axes for one open path.",
        ],
        Job::Cook => &[
            "Heat, time, attention — the three ingredients.",
            "A cooked berry feeds the soul twice.",
            "The fire knows what I want.",
            "I like a kitchen that's also a campfire.",
            "Stir, wait, smell. Never rush.",
        ],
        Job::Digger => &[
            "Every stone has a destination.",
            "Logs to the fire, stones to the shrine.",
            "Hauling is a slow kind of strength.",
            "I move what others only look at.",
            "The road is paved by people like me.",
        ],
        Job::Healer => &[
            "Grief doesn't mend alone.",
            "A calm word lands like a cool hand.",
            "The shrine is just a place to breathe.",
            "I carry other people's weather.",
            "Sometimes they just need to be heard.",
        ],
        Job::Fisherman => &[
            "Cast. Wait. Reel. Simple.",
            "The river gives if you listen.",
            "A good spot is half the catch.",
            "Patience is the only bait that always works.",
            "Still water, still mind.",
        ],
    };
    pick(lines, seed)
}

fn introspective_thought(bot: &Bot, seed: u32) -> String {
    let mut lines: Vec<&str> = Vec::new();
    if bot.mood > 40.0 {
        lines.extend_from_slice(&[
            "I don't know why, but I feel good.",
            "Today is the kind of day worth remembering.",
            "I could do this forever.",
        ]);
    } else if bot.mood < -40.0 {
        lines.extend_from_slice(&[
            "Heavy morning. Heavier afternoon.",
            "Why does nothing fit right?",
            "I'll sleep and see if it leaves.",
        ]);
    }
    if bot.boredom > 70.0 {
        lines.push("If one more hour passes like this, I'll scream.");
        lines.push("There has to be something new under the sun.");
    }
    if bot.social > 70.0 {
        lines.push("I miss hearing my name.");
        lines.push("Has anyone noticed I'm gone?");
    }
    match bot.traits.dominant() {
        "curious" => lines.extend_from_slice(&[
            "Why does the water flicker like that?",
            "I wonder if the rocks remember things.",
        ]),
        "brave" => lines.extend_from_slice(&[
            "Fear is a small voice I've learned to argue with.",
            "If I don't go, who will?",
        ]),
        "fierce" => lines.extend_from_slice(&[
            "I hold my ground. Always.",
            "Challenge me and find out.",
        ]),
        "busy" => lines.extend_from_slice(&[
            "Idleness itches.",
            "I'll rest when the work is done.",
        ]),
        "social" => lines.extend_from_slice(&[
            "People are the whole weather.",
            "Laughter is better than sleep.",
        ]),
        _ => {}
    }
    if lines.is_empty() {
        lines.extend_from_slice(&[
            "What a strange life this is.",
            "I should think more often.",
            "One step, one thought, one step.",
        ]);
    }
    lines[seed as usize % lines.len()].to_string()
}

fn contextual_thought(world: &World, idx: usize, seed: u32) -> Option<String> {
    let bot = &world.bots[idx];
    // Find the nearest other bot (within 5) and reference them by name.
    let mut nearest: Option<(u32, String, i32)> = None;
    for b in &world.bots {
        if b.id == bot.id || !b.alive {
            continue;
        }
        let d = (b.x - bot.x).abs() + (b.y - bot.y).abs();
        if d <= 5 && nearest.as_ref().map_or(true, |(_, _, bd)| d < *bd) {
            nearest = Some((b.id, b.name.clone(), d));
        }
    }
    if let Some((oid, oname, _)) = nearest {
        let aff = *bot.relationships.get(&oid).unwrap_or(&0);
        let line = if aff > 15 {
            match seed % 4 {
                0 => format!("There's {}. Always good company.", oname),
                1 => format!("I should say hi to {}.", oname),
                2 => format!("{} has that look again.", oname),
                _ => format!("Glad it's {} and not someone else.", oname),
            }
        } else if aff < -15 {
            match seed % 4 {
                0 => format!("{}. Not my favourite.", oname),
                1 => format!("Keeping my distance from {}.", oname),
                2 => format!("What's {} up to now?", oname),
                _ => format!("I wish {} would go elsewhere.", oname),
            }
        } else {
            match seed % 4 {
                0 => format!("Oh, it's {}.", oname),
                1 => format!("Hey, {}.", oname),
                2 => format!("{} is around again.", oname),
                _ => format!("I see {} out there.", oname),
            }
        };
        return Some(line);
    }
    // Fallback: mention a nearby tile feature.
    let tile_here = world.tile(bot.x, bot.y);
    let line = match tile_here {
        Tile::Forest => "Cool and quiet under these trees.",
        Tile::Sand => "Sand. Strange to stand on.",
        Tile::Flower => "A flower. Little things matter.",
        Tile::Home => "The roof holds. That's enough.",
        _ => return None,
    };
    Some(line.to_string())
}

fn update_mood(world: &mut World, idx: usize) {
    let bot = &mut world.bots[idx];
    bot.mood *= 0.995;
    if bot.hunger < 30.0 && bot.energy > 60.0 && bot.boredom < 40.0 {
        bot.mood = (bot.mood + 0.1).min(100.0);
    }
    bot.mood = bot.mood.clamp(-100.0, 100.0);
}
