//! Headless simulation tests against the public `Gridland` API.
//! Run with `cargo test --release` (debug builds are slow for long soaks).

use gridland::Gridland;

fn run(seed: u32, ticks: u32) -> Gridland {
    let mut g = Gridland::new(seed);
    for _ in 0..ticks {
        g.tick();
    }
    g
}

/// Same seed → same world. Everything visible in `stats()` and the event
/// log must match, or replays, comparisons between builds and bug repros
/// all stop meaning anything. (Found at build 5: HashMap iteration order
/// made runs drift; sim state now uses BTreeMap — keep it that way.)
#[test]
fn same_seed_same_world() {
    for seed in [1, 42] {
        let a = run(seed, 8_000);
        let b = run(seed, 8_000);
        assert_eq!(a.stats(), b.stats(), "stats diverged for seed {seed}");
        assert_eq!(a.event_log(), b.event_log(), "event log diverged for seed {seed}");
        assert_eq!(a.bots_summary(), b.bots_summary(), "bots diverged for seed {seed}");
    }
}

/// Different seeds → different worlds (guards against the seed being ignored).
#[test]
fn different_seeds_differ() {
    assert_ne!(run(1, 500).bots_summary(), run(2, 500).bots_summary());
}

/// Long soak on several seeds: no panics, a render still fills the buffer,
/// and the village isn't wiped out within ~4 minutes of 2× play.
#[test]
fn soak_survives() {
    for seed in [3, 99, 2024] {
        let mut g = run(seed, 30_000);
        g.render();
        assert_eq!(g.buffer_len() as u32, g.canvas_w() * g.canvas_h() * 4);
        let stats = g.stats();
        let bots: u32 = stats
            .split("\"bots\":")
            .nth(1)
            .and_then(|s| s.split(',').next())
            .and_then(|s| s.parse().ok())
            .expect("stats has a bots field");
        assert!(bots > 0, "seed {seed}: everyone died by tick 30k");
    }
}
