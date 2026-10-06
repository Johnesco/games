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

## Known issues at build 5 (pinned by ignored tests)

- **Non-deterministic seeds:** `World::fire_fuel` / `tile_age` are `HashMap`s walked in std's random order
  (`step_weather`, `step_decay`). Probe numbers vary a little run to run until this is fixed.
- **Bots trapped in trees:** Forest matures into Tree under a standing bot (`step_environment`).
