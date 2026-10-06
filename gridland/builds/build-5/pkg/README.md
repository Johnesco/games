# Gridland

A 64×64 tile world where little bots with thoughts, moods, memories, and
relationships quietly live out their days. Sit back and watch, or interact
with them.

## Stack

- **Rust** + **wasm-bindgen** compiled to **WebAssembly** (single `cdylib`)
- **Canvas 2D** via `ImageData` for crisp 8-bit rendering (no external engine)
- Plain **HTML/CSS/JS** glue — no bundler, no framework

### Why not Bevy / Macroquad?

For a 64×64 grid with simple sprites, raw `ImageData` wins on build time,
bundle size, and control. Consider upgrading if you want:

- **[Bevy](https://bevyengine.org/)** — full ECS, sound, shaders, many bots
- **[Macroquad](https://macroquad.rs/)** — tiny 2D engine, input & audio built-in
- **[pixels](https://crates.io/crates/pixels)** — nice pixel-buffer wrapper

## Build

```bash
wasm-pack build --target web --out-dir www/pkg --release
```

## Run

The app loads WASM modules, so it needs HTTP (not `file://`):

```bash
# any static server works — e.g.
python -m http.server 8080 --directory www
# or
npx http-server www -p 8080
```

Then open <http://localhost:8080>.

## Bot brain model

- **Drives** (hunger, energy, social, boredom) grow with time.
- **Traits** (curiosity, sociability, aggression, industriousness, bravery)
  are rolled at birth and shape goal weighting.
- **Goal selection** uses weighted utilities with hysteresis so bots don't
  flip-flop every tick.
- **Memory** — bots remember food locations, home, and other bots as
  friends/enemies based on accumulated affinity from encounters.
- **Actions** — move toward target greedily, eat berries, build homes,
  plant saplings, flee from enemies.

## Interact

- **Click a bot** → inspector panel with live thoughts, traits, drives,
  memory, and relationships
- **Drop berry** tool → plant food anywhere
- **Place rock** tool → drop an obstacle
- **Clear** tool → erase a user-placed tile
- **Reseed** → generate a new world
