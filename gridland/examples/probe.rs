//! Headless soak probe: runs the sim on fixed seeds and prints one JSON
//! document of periodic `stats()` samples. `tools/cut.mjs` runs this for
//! every build so the diary can show how the village behaves, not just
//! what changed. Uses only `new` / `tick` / `stats`, which every build
//! since build 1 exposes — so it also runs against historical sources.
//!
//!   cargo run --release --example probe            # default: 3 seeds × 60k ticks
//!   cargo run --release --example probe -- 20000   # shorter run

use gridland::Gridland;

const SEEDS: [u32; 3] = [1, 42, 777];
const SAMPLE_EVERY: u32 = 6000;

fn main() {
    let ticks: u32 = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(60_000);
    let mut runs = Vec::new();
    for seed in SEEDS {
        let mut g = Gridland::new(seed);
        let mut samples = vec![g.stats()];
        for t in 1..=ticks {
            g.tick();
            if t % SAMPLE_EVERY == 0 {
                samples.push(g.stats());
            }
        }
        runs.push(format!("{{\"seed\":{},\"samples\":[{}]}}", seed, samples.join(",")));
    }
    println!(
        "{{\"ticks\":{},\"sample_every\":{},\"runs\":[{}]}}",
        ticks,
        SAMPLE_EVERY,
        runs.join(",")
    );
}
