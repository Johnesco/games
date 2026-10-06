//! The village calendar, the slow clock (docs/PLAYER.md, "Two clocks").
//!
//! Village time is derived purely from the tick count, so it stays
//! deterministic: the sim never reads the wall clock. The real-time contract
//! lives in the constants: at 1× the page runs `TICKS_PER_SEC` ticks per real
//! second, and 1 real hour = 1 village day. Anchoring a village to the wall
//! clock (time away, catch-up) comes later with saving.

/// Activity-clock rate at 1× (real time). The page reads this through
/// `Gridland::ticks_per_sec`, so the JS loop and the calendar never disagree.
/// 120 is the pace the sim was tuned at (the old "2×" default).
pub const TICKS_PER_SEC: u64 = 120;
/// 1 real hour = 1 village day.
pub const TICKS_PER_DAY: u64 = TICKS_PER_SEC * 3600;
pub const TICKS_PER_HOUR: u64 = TICKS_PER_DAY / 24;
pub const DAYS_PER_SEASON: u64 = 24; // a season = 1 real day
pub const SEASONS_PER_YEAR: u64 = 4; // a year = 4 real days
pub const DAYS_PER_YEAR: u64 = DAYS_PER_SEASON * SEASONS_PER_YEAR;

/// A new village wakes at dawn on the first day of spring.
pub const START_HOUR: u64 = 6;
/// Night is 21:00–05:00. Dusk and dawn each take the hour on the
/// other side of that boundary.
pub const NIGHT_FROM: f32 = 21.0;
pub const NIGHT_UNTIL: f32 = 5.0;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Season {
    Spring,
    Summer,
    Autumn,
    Winter,
}

impl Season {
    pub fn label(self) -> &'static str {
        match self {
            Season::Spring => "spring",
            Season::Summer => "summer",
            Season::Autumn => "autumn",
            Season::Winter => "winter",
        }
    }
}

/// A moment on the village calendar. All fields are 0-based except
/// `year` and `day`, which count from 1 the way villagers would.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct VillageTime {
    pub year: u64,
    pub season: Season,
    pub day: u64, // 1..=DAYS_PER_SEASON
    pub hour: u64,
    pub minute: u64,
    /// Days since the village began (0-based), for ages and history.
    pub day_index: u64,
}

/// Village clock offset: tick 0 is START_HOUR on day 1.
fn clock_ticks(tick: u64) -> u64 {
    tick + START_HOUR * TICKS_PER_HOUR
}

pub fn time_at(tick: u64) -> VillageTime {
    let t = clock_ticks(tick);
    let day_index = t / TICKS_PER_DAY;
    let in_day = t % TICKS_PER_DAY;
    let day_of_year = day_index % DAYS_PER_YEAR;
    let season = match day_of_year / DAYS_PER_SEASON {
        0 => Season::Spring,
        1 => Season::Summer,
        2 => Season::Autumn,
        _ => Season::Winter,
    };
    VillageTime {
        year: day_index / DAYS_PER_YEAR + 1,
        season,
        day: day_of_year % DAYS_PER_SEASON + 1,
        hour: in_day / TICKS_PER_HOUR,
        minute: (in_day % TICKS_PER_HOUR) * 60 / TICKS_PER_HOUR,
        day_index,
    }
}

/// Hour of the village day as a fraction, 0.0..24.0.
pub fn hour_f(tick: u64) -> f32 {
    (clock_ticks(tick) % TICKS_PER_DAY) as f32 / TICKS_PER_HOUR as f32
}

pub fn is_night(tick: u64) -> bool {
    let h = hour_f(tick);
    h >= NIGHT_FROM || h < NIGHT_UNTIL
}

/// Ambient daylight, 0.0 (deep night) to 1.0 (full day). Ramps linearly
/// through dawn (05–06) and dusk (20–21), so the light change at a 1-hour
/// day takes about 2.5 real minutes each way.
pub fn daylight(tick: u64) -> f32 {
    let h = hour_f(tick);
    if h >= NIGHT_UNTIL + 1.0 && h < NIGHT_FROM - 1.0 {
        1.0
    } else if h >= NIGHT_UNTIL && h < NIGHT_UNTIL + 1.0 {
        h - NIGHT_UNTIL // dawn
    } else if h >= NIGHT_FROM - 1.0 && h < NIGHT_FROM {
        NIGHT_FROM - h // dusk
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_at_dawn_on_spring_day_one() {
        let t = time_at(0);
        assert_eq!((t.year, t.season, t.day, t.hour, t.minute), (1, Season::Spring, 1, START_HOUR, 0));
        assert!(!is_night(0));
        assert_eq!(daylight(0), 1.0);
    }

    #[test]
    fn real_time_contract() {
        // 1 real hour at 1× is exactly one village day.
        let a = time_at(0);
        let b = time_at(TICKS_PER_SEC * 3600);
        assert_eq!(b.day_index, a.day_index + 1);
        assert_eq!((b.hour, b.minute), (a.hour, a.minute));
        // A season is a real day; a year is four.
        assert_eq!(time_at(TICKS_PER_SEC * 3600 * 24).season, Season::Summer);
        assert_eq!(time_at(TICKS_PER_SEC * 3600 * 96).year, 2);
    }

    #[test]
    fn night_and_light() {
        let at = |h: u64, m: u64| (h + 24 - START_HOUR) % 24 * TICKS_PER_HOUR + m * TICKS_PER_HOUR / 60;
        assert!(is_night(at(23, 0)) && is_night(at(4, 59)));
        assert!(!is_night(at(5, 0)) && !is_night(at(20, 59)));
        assert_eq!(daylight(at(2, 0)), 0.0);
        assert_eq!(daylight(at(12, 0)), 1.0);
        assert!((daylight(at(5, 30)) - 0.5).abs() < 0.01);
        assert!((daylight(at(20, 30)) - 0.5).abs() < 0.01);
    }
}
