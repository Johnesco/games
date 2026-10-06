# Gridland: how changes ship

Rust → WebAssembly village sim, served from `master` by GitHub Pages at
<https://johnesco.github.io/games/gridland/>. See README.md for the sim itself.

## Layout

| path | what it is |
|---|---|
| `index.html` | **The diary**: every build, newest first, each playable. Renders `builds.json`. The games index card links here. |
| `builds.json` | Build list: number, date, title, notes, known issues, commit, soak stats. Written by `tools/cut.mjs`; notes are edited by hand. |
| `builds/build-N/` | A frozen, playable copy of each build. **Never edit**: a fix ships as the next build. |
| `www/` | The dev working copy (shows "dev build"). What you edit and preview. |
| `src/` | The sim (Rust). `www/pkg/` is its compiled output, committed so Pages needs no build step. |
| `tests/sim.rs`, `src/world.rs` `mod tests` | Headless tests: `cargo test --release`. |
| `examples/probe.rs` | Soak probe: 3 seeds × 60k ticks → periodic `stats()` JSON. Feeds each build's diary numbers. |
| `docs/PLAYER.md` | **Design direction**: the player as a small god whose power comes from belief; journal, milestones, biographies; saved village that lives on while you're away. Check features against it. |
| `tools/cut.mjs` | Build pipeline (below). `tools/history.json` holds the notes for builds 1–5, which were recovered from git. |

## The loop

1. Change `src/` and/or `www/`. Rebuild the wasm: `wasm-pack build --target web --out-dir www/pkg --release`.
2. Preview: the `gridland` launch config serves this folder on :8080 (`/www/` = dev, `/` = diary).
3. `cargo test --release` must pass. Tests marked `#[ignore = "known bug: …"]` are open bugs:
   un-ignore a test in the build that fixes its bug.
4. Once a design change is done, cut a build:
   ```
   node tools/cut.mjs cut --title "Headline" --notes "What changed and why, for a player." --notes "…" [--known "…"]
   ```
   This runs wasm-pack → tests → probe, copies `www/` to `builds/build-N/` with the header stamped
   "build N · diary", and appends to `builds.json`.
5. Commit everything (one commit, e.g. `feat(gridland): build N — headline`), then tag it with
   `git tag gridland-build-N` and push the tag along with the commit.

**One build = one design change.** Small fixes ride along with the next one. Diary notes are written
for someone watching the village, not as a code changelog. Put engineering detail in the commit message.

## Unreleased: in `www/`, ships with the next build

Fixes ride along until a significant design change is ready to cut as build 6 (owner's call, 2026-10-06).
When cutting, fold these into the diary notes.

- **Seeds are deterministic.** All sim maps (`fire_fuel`, `tile_age`, `cook_progress`, `relationships`) are
  `BTreeMap`s; std `HashMap` iteration order is random per instance and made runs drift. Don't reintroduce
  `HashMap`/`HashSet` into sim state. `same_seed_same_world` guards this, and the probe is now byte-identical run to run.
- **Bots no longer get trapped in trees.** Forest won't mature into Tree under a standing bot
  (`step_environment`). `bots_stay_in_bounds_and_on_walkable_tiles` guards this.
- **Same speed on every screen.** Sim ticks run on a fixed timestep against real time (`TICKS_PER_SEC` = 60 per 1×
  in `www/main.js`), not once per animation frame; a 144 Hz monitor used to run the world 2.4× faster. Checked by
  driving the loop with synthetic 144 / 60 / 30 Hz frames: 120 ticks per second at 2× each time. Frame gaps over
  250 ms are dropped until the away/catch-up model exists (docs/PLAYER.md).
