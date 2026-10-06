# Generations: design draft

Status: **draft for review** (2026-10-06). Builds on docs/PLAYER.md (real time; 1 real hour = 1 village day) and
docs/ART.md (32 px procedural sprites that show age and inherited looks).

## Step 0: stop the accidental deaths (done in dev)

Before this work, every death was a bug, not old age. Over one village day, 24 bots fell to 7–9, and a diagnostic
(`world::death_diag`) showed every one starving **while trying to eat**:

| Cause | Fix |
|---|---|
| Greedy one-step movement; water, rock and tree walls trapped bots in sight of food | A* routing (`src/path.rs`): detours preferred; trees and rocks are expensive but passable, so frustration-driven clearing still happens when there's no reasonable way around |
| Nothing ever ate carried food; bots starved holding cooked fish | Eat what you carry when hungry (≥ 70), even mid-errand when starving (≥ 85) |
| Bots treated each other as walls; whole queues starved behind one water-edge tile | Wait about a second, then squeeze past (tiles may be shared briefly) |
| A starving bot froze for ~4 s "contemplating" before going to eat | Urgent eat/drink pivots wait 40 ticks, not 500 |

## The life cycle (proposed numbers, in village days; 1 day = 1 real hour)

| Stage | Age | Real time | What changes |
|---|---|---|---|
| Child | 0–48 days | ~2 real days | Smaller sprite. No job; eats, drinks, rests, plays near home, follows a parent. Slower |
| Adult | 48–260 | ~9 real days | Takes a job at coming of age (from traits; later also from what the village lacks). Can partner and raise children |
| Elder | 260+ | ~3+ real days | Grey hair, cane, slower, tires sooner. Keeps their job; later: teaches apprentices, keeps the stories |
| Death of old age | from ~300, rising | mean ≈ 340 days ≈ 2 real weeks | Grave, mourning by family and friends; biography closes |

Initial villagers get **staggered ages** (adults of 48–200 days) so the founders don't all die in the same hour.

## Partners and children

- **Partnering:** two unpartnered adults whose affinity for each other is high (both ways) become partners. The sim
  has no sexes or genders: any two adults can pair. Partners share a home.
- **Children:** a partnered pair with a home, both adults (not elders), can have a child. The chance per village day
  rises when they're well fed and happy and falls with each child they already have and with village crowding. The
  child appears at home, which gets logged and a bubble announces it.
- **Inheritance:** personality traits are the average of the parents' ± a little noise. Looks (skin, hair, eyes,
  hairstyle, build; see the sprite test) take each gene from one parent. Recorded now, drawn when the 32 px renderer
  lands.
- **Raising:** parents prefer to give food gifts (an existing mechanic) to their own children; children stay near home.
- **Names:** given names generated from syllables. Each bot remembers its parents, partner and children, which feeds
  biographies and family lines.

## Population size

No hard cap in design terms: births slow when food per head falls (carrying capacity emerges from the land). There is
a **safety cap** (e.g. 60) for performance until the renderer and the sim are optimised.

## How it's tested

A lifetime at the real ratio is ~145M ticks, about 9 hours of native simulation, so:

- **Mechanics** (aging, partnering, births, inheritance, death of old age) are tested with a **compressed calendar**:
  a test-only day length (e.g. 1,200 ticks), so a whole lifetime runs in seconds. Life-cycle rules are written in
  village days, so they compress cleanly.
- **Balance** (does the population hold steady?) can't be judged compressed, because food grows per tick, not per day.
  It needs a **long soak** at the real ratio: one village year is ~41M ticks, roughly 2.5 hours natively, run in the
  background. Its numbers also calibrate the chronicle model later.
- **Speed matters now:** the sim runs ~1,300 ticks/s in wasm with 24 bots (~11× real time), and every soak is long.
  Bots now cache their routes, but that didn't speed things up, so the bottleneck is elsewhere; profile first.

## Open questions

1. Lifespan ≈ 2 real weeks with childhood ≈ 2 real days: right feel?
2. Partners for life, or can pairs drift apart when affinity falls?
3. Population: let it find its own level (around 20–40 on this map), or aim for a specific size?
