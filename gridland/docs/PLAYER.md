# The player in Gridland

Design direction, decided 2026-10-06. This is the reference for what the player is and does; check
features against it.

## The role: a small god

The player is the **Visitor**: a presence outside the world that shapes it. You're not a manager. You never
command a bot, assign a job or pick a target. You change *the world* (weather, land, gifts, signs), and the
villagers decide what it means and what to do about it.

The village must stay worth watching with no input at all. The documentary camera remains the idle state.
Being a god is optional and occasional; a session can be pure watching.

## Influence: belief is power

Influence is **scarce, and it comes from the villagers.**

- **Favour** is the player's resource. It refills slowly, and how fast depends on **how much the village
  believes in you.** A believing village sustains a strong god; an indifferent one leaves you with only
  small acts. (Populous / Black & White lineage, kept gentle.)
- **Bots notice what you do.** Every act happens somewhere, to someone, and gets witnessed. Witnesses form
  a belief about it ("the sky sent fire on the coldest night", "stones fall on the greedy"), talk about it,
  and pass it down to their children. Belief can be warm (gratitude, shrines, offerings) or fearful
  (avoidance, appeasement). Both produce favour; they make very different villages.
- **Acts have weight.** Small acts are cheap: a berry, a breeze. Big acts are expensive: rain on demand,
  raising land, lightning, a blessing on one bot. Spamming is impossible by design, so the scarcity and
  seasons systems keep their meaning.
- **Neglect matters.** A god who does nothing gets slowly forgotten: belief fades over generations unless the
  stories are retold. Coming back to an abandoned village and seeing what it made of you is a goal in itself.

## Progress: three ways it builds up

1. **Discovery journal.** Notable behaviours and events you *personally witnessed* (on screen while they
   happened): a first funeral, a feud, a lightning fire, a bare-handed tree-felling, a birth, a pilgrimage
   to your shrine. It's a field guide that fills in. Missing something is fine; it will happen again.
2. **Village milestones.** The village's own achievements: population peaks, generations, the first cabin,
   the oldest bot ever, the longest-burning fire, the first child born to two families. These are recorded
   in the village's history.
3. **Bot biographies.** Every bot has a life story: birth, parents, job, friendships, feuds, what it
   believed about you, and how it died. Family lines connect them. Graves link to their biographies.

There's no win or lose state. A village can die out; that's a story too, and you can start another.

## Time: real time, can't be rushed

The village runs on **the real clock**, with no fast-forward. If you step away for a week, the village lives that week.

### Two clocks

- **The activity clock** is how bots move, eat, talk and work, at a pace you can watch. It's a fixed number of
  ticks per real second, no longer tied to screen refresh. (Today a tick runs every animation frame, so a 144 Hz
  monitor runs the world 2.4× faster, and a village day lasts 10 seconds.)
- **The calendar clock** is days, seasons, aging, births, deaths and generations, tied to the wall clock at a
  fixed ratio: 1 real hour = 1 village day. Hunger and moods change minute to minute; lives unfold over
  real days and weeks.

### Live vs. away

- **Live**: the tab is open and visible. The full sim runs at the activity clock, 1× only. The speed buttons go
  away. A hidden tab counts as **away** (browsers throttle hidden tabs anyway).
- **Away**: on return, the elapsed real time is simulated by a **chronicle model**: a coarse, village-level
  simulation stepped once per village hour. It covers population, food balance per household, aging, births and
  deaths, construction, growth and decay of the land, and belief. It also samples notable events from the same
  rules the full sim uses. It writes real history entries and applies their results to the map (homes built,
  trees grown, graves placed).
  - **Fast by design**: a week is a few hundred coarse steps, which takes well under a second.
  - **Settling**: after the chronicle catches up, the full sim runs for a short stretch so bots are mid-errand,
    mid-conversation and on worn paths, not standing where they were placed.
  - **Calibrated**: the probe measures the full sim's rates (birth, death, food, building), the chronicle uses them,
    and a test checks that the chronicle and the full sim stay within tolerance over the same span of village time.
- **The recap** ("while you were away") plays while the catch-up finishes: who was born, who died, what was built,
  what they now say about you. **You can't act until it's done.**
- **Limits**: no cap on catch-up (see below). If the clock goes backwards, it's ignored. Moving the device clock
  forward only ages the village; it can't earn the player anything.

### Progress happens only while you're live

The village keeps living without you, **but you make no progress by ignoring it:**

- **Journal**: an entry needs you to witness the event on screen. Inherently live.
- **Biographies**: every bot's history is recorded, but a bot's biography is only *rich* if you met it live
  (inspected or watched it). Bots who lived and died while you were away get a short gravestone line. Being
  there is how you come to know them.
- **Milestones**: the village's milestones are recorded in its history whenever they happen. The player's own
  progress comes from the journal, from biographies, and from the influence and renown earned through acts, which
  are always live.
- **Favour** (the means to act, not progress): it builds up while you're away into a **capped reservoir** of
  about a day's worth. Staying away longer gains nothing extra.

### Reasons to come back regularly

The target is **a visit or two a day, 5–20 minutes**, driven by the world, not by guilt mechanics:

1. **The favour reservoir is full**: daily visits use everything you earn; longer gaps waste it.
2. **Prayers**: villagers make requests in response to real trouble (drought, a hard winter, sickness, a feud)
   with a deadline in village time. Answering is live-only. Unanswered prayers wear belief down, never below a floor.
3. **Seasons turn on a real-time rhythm**: something new to see each visit, and a reason to check whether the
   stores will last the winter.
4. **The bots you know are mortal**: with lifespans of real days to weeks, a regular visitor follows lives;
   someone who stays away comes back to strangers. That's how biographies reward regularity.
5. **Belief fades over generations** without new acts to retell. Long absences cost renown, but never wreck the
   village.

### The calendar ratio (decided 2026-10-06)

**1 real hour = 1 village day.** A season is 1 real day (24 village days), a year is 4 real days, and a lifespan
is about 2 real weeks. A week away is about 1.75 village years: some faces gone, still recognisable. Daily visitors
follow lives.

### No catch-up cap (decided 2026-10-06)

However long you're away, the village lived it all: six months is about 4,400 village days. So the chronicle model
must:
- **Stay fast at any length**: step per village hour for the recent past and per village day further back. The
  recap summarises long absences by era instead of by event.
- **Handle extinction**: a village can die out while you're away. The land and the graves remain. *Open:* how a
  new village begins (wanderers arriving on their own after a while, or the god calling settlers).
- **Stay stable**: runaway growth or collapse over thousands of steps is a calibration bug, and the long-run
  chronicle test should catch it.

Since nothing caps the catch-up, the rule that progress only happens live is what stops it from being exploited.

## What this means for existing tools

| Today | Becomes |
|---|---|
| drop berry / rock / fire (free, unlimited) | god acts that cost favour and are witnessed and interpreted |
| click empty grass → berry (accidental) | removed; clicking only selects or inspects |
| clear | probably a god act ("the land forgives"), or removed |
| reseed | "start a new village", behind a confirmation once saving exists |
| speed buttons (1×–8×), pause | removed: real time, can't be rushed |
| inspector | grows into the bot biography |
| event log | grows into the village history, feeding the recap, milestones and journal |

## Suggested order

Each step is a candidate significant change for a build:

1. **Generations**: births, inheritance, aging. Biographies, family lines and passed-down beliefs all need
   lives that begin and end. It also fixes the steady population decline every build shows.
2. **Real-time clock + village history + saving + time away**: a fixed-timestep activity clock, a calendar
   clock, a structured event history (not just log strings), autosave, the chronicle model with its calibration
   test, and the recap.
3. **Favour, belief and god acts**: witnesses, beliefs, favour income, the expanded set of acts, and
   removing the free tools.
4. **Journal, milestones, biographies UI.**
5. Scarcity and seasons, dynamic jobs and settlements get woven in after this, each making the god's
   choices matter more.
