//! Headless soak probe: runs the sim on fixed seeds and prints one JSON
//! document of periodic `stats()` samples. `tools/cut.mjs` runs this for
//! every build so the diary can show how the village behaves, not just
//! what changed. Uses only `new` / `tick` / `stats`, which every build
//! since build 1 exposes, so it also runs against historical sources
//! (backfill passes the 60k ticks those builds were measured over).
//!
//!   cargo run --release --example probe                # 1 village day (432k ticks), sampled hourly
//!   cargo run --release --example probe -- 60000 6000   # ticks, sample interval

use gridland::Gridland;

const SEEDS: [u32; 3] = [1, 42, 777];
// Mirrors calendar.rs (private to the crate, and absent from old builds):
// 120 ticks/s × 3600 s = 1 village day.
const VILLAGE_DAY: u32 = 432_000;
const VILLAGE_HOUR: u32 = VILLAGE_DAY / 24;

fn main() {
    let mut args = std::env::args().skip(1).map(|s| s.parse::<u32>().ok());
    let ticks = args.next().flatten().unwrap_or(VILLAGE_DAY);
    let every = args.next().flatten().unwrap_or(VILLAGE_HOUR);
    // One thread per seed: a village day is ~100 s of sim per seed.
    let handles: Vec<_> = SEEDS
        .iter()
        .map(|&seed| {
            std::thread::spawn(move || {
                let mut g = Gridland::new(seed);
                let mut samples = vec![g.stats()];
                for t in 1..=ticks {
                    g.tick();
                    if t % every == 0 {
                        samples.push(g.stats());
                    }
                }
                format!("{{\"seed\":{},\"samples\":[{}]}}", seed, samples.join(","))
            })
        })
        .collect();
    let runs: Vec<String> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    println!(
        "{{\"ticks\":{},\"sample_every\":{},\"runs\":[{}]}}",
        ticks,
        every,
        runs.join(",")
    );
}
