//! Pathfinding: A* over the tile grid.
//!
//! Movement used to be a greedy one-step-toward-the-target rule, and walls
//! of water, rock or trees trapped bots until they starved within sight of
//! food (found 2026-10-06 while planning generations). Now bots plan a
//! route. Trees and rocks are *expensive but passable* in the plan: a bot
//! walks around them when a detour is reasonable, and otherwise walks up to
//! one and the frustration-driven clearing in `ai::step_toward_target`
//! takes over, as before.

use crate::world::{Tile, World, H, W};
use std::cmp::Reverse;
use std::collections::BinaryHeap;

/// Cost of stepping onto a tile, or None if it can never be crossed.
/// Units: a grass step is 10.
pub fn step_cost(t: Tile) -> Option<u32> {
    match t {
        Tile::Path => Some(7),
        Tile::Sand => Some(14),
        Tile::Tree => Some(150), // a long detour beats clearing by hand
        Tile::Rock => Some(300),
        t if t.walkable() => Some(10),
        _ => None, // water
    }
}

const MIN_COST: u32 = 7;

/// What the first step of the best route is.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Step {
    /// Already there (or adjacent to a target that can't be stood on).
    Arrived,
    /// Walk to this tile.
    Walk(i32, i32),
    /// The route goes through this obstacle (a Tree or Rock) first.
    Obstacle(i32, i32),
    /// No route exists at all.
    Unreachable,
}

/// The cheapest route from `from` to `to`, as the tiles to walk in order,
/// cut off at (and including) the first obstacle. A target that can't be
/// stood on (a tree to chop, open water) counts as reached from any
/// orthogonal neighbour. `None` = no route; an empty Vec = already there.
pub fn route(world: &World, from: (i32, i32), to: (i32, i32)) -> Option<Vec<(i32, i32)>> {
    let target_standable = world.tile(to.0, to.1).walkable();
    let reached = |x: i32, y: i32| {
        (x, y) == to || (!target_standable && (x - to.0).abs() + (y - to.1).abs() == 1)
    };
    if reached(from.0, from.1) {
        return Some(Vec::new());
    }
    let idx = |x: i32, y: i32| y as usize * W + x as usize;
    let h = |x: i32, y: i32| ((x - to.0).abs() + (y - to.1).abs()) as u32 * MIN_COST;

    let mut g = vec![u32::MAX; W * H];
    let mut came: Vec<u32> = vec![u32::MAX; W * H];
    let mut open = BinaryHeap::new();
    g[idx(from.0, from.1)] = 0;
    open.push(Reverse((h(from.0, from.1), 0u32, from.0, from.1)));

    while let Some(Reverse((_, cost, x, y))) = open.pop() {
        if cost > g[idx(x, y)] {
            continue; // stale entry
        }
        if reached(x, y) {
            // Walk back to the start, then keep the steps up to the first obstacle.
            let mut steps = vec![(x, y)];
            let (mut cx, mut cy) = (x, y);
            loop {
                let p = came[idx(cx, cy)] as usize;
                let (px, py) = ((p % W) as i32, (p / W) as i32);
                if (px, py) == from {
                    break;
                }
                steps.push((px, py));
                cx = px;
                cy = py;
            }
            steps.reverse();
            if let Some(k) = steps.iter().position(|&(sx, sy)| !world.tile(sx, sy).walkable() && (sx, sy) != to) {
                steps.truncate(k + 1);
            }
            return Some(steps);
        }
        // Obstacles stay in the search at their (high) clearing cost.
        for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let (nx, ny) = (x + dx, y + dy);
            if nx < 0 || ny < 0 || nx >= W as i32 || ny >= H as i32 {
                continue;
            }
            let t = world.tile(nx, ny);
            let step = match step_cost(t) {
                Some(c) => c,
                None if reached(nx, ny) => 10, // e.g. water as a fishing target
                None => continue,
            };
            let ng = cost + step;
            let i = idx(nx, ny);
            if ng < g[i] {
                g[i] = ng;
                came[i] = idx(x, y) as u32;
                open.push(Reverse((ng + h(nx, ny), ng, nx, ny)));
            }
        }
    }
    None
}

/// First step of the cheapest route (see `route`).
pub fn first_step(world: &World, from: (i32, i32), to: (i32, i32)) -> Step {
    match route(world, from, to) {
        None => Step::Unreachable,
        Some(r) if r.is_empty() => Step::Arrived,
        Some(r) => classify(world, r[0]),
    }
}

/// Whether stepping to `p` is a walk or runs into an obstacle.
pub fn classify(world: &World, p: (i32, i32)) -> Step {
    if world.tile(p.0, p.1).walkable() {
        Step::Walk(p.0, p.1)
    } else {
        Step::Obstacle(p.0, p.1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat(seed: u64) -> World {
        let mut w = World::new(seed);
        w.bots.clear();
        for v in w.tiles.iter_mut() {
            *v = Tile::Grass as u8;
        }
        w
    }

    #[test]
    fn walks_around_water() {
        let mut w = flat(1);
        // A wall of water at x=5 from y=0..9, open at y=10.
        for y in 0..10 {
            w.set_tile(5, y, Tile::Water);
        }
        // Greedy would push straight into the wall; A* goes down and around.
        assert_eq!(first_step(&w, (4, 2), (6, 2)), Step::Walk(4, 3));
    }

    #[test]
    fn prefers_a_detour_to_clearing_a_tree() {
        let mut w = flat(1);
        w.set_tile(5, 5, Tile::Tree);
        let s = first_step(&w, (4, 5), (6, 5));
        assert!(matches!(s, Step::Walk(4, 4) | Step::Walk(4, 6)), "{s:?}");
    }

    #[test]
    fn goes_through_a_tree_when_no_reasonable_detour() {
        let mut w = flat(1);
        // A full-height line of trees with no gap.
        for y in 0..H as i32 {
            w.set_tile(5, y, Tile::Tree);
        }
        assert_eq!(first_step(&w, (4, 5), (6, 5)), Step::Obstacle(5, 5));
    }

    #[test]
    fn unreachable_across_water() {
        let mut w = flat(1);
        for y in 0..H as i32 {
            w.set_tile(5, y, Tile::Water);
        }
        assert_eq!(first_step(&w, (4, 5), (6, 5)), Step::Unreachable);
    }

    #[test]
    fn non_standable_target_reached_from_beside_it() {
        let mut w = flat(1);
        w.set_tile(6, 5, Tile::Water);
        assert_eq!(first_step(&w, (5, 5), (6, 5)), Step::Arrived);
    }
}
