//! The crate's **one** time seam and its **one** calendar routine — split from
//! [`super`] at §12's budget on the seam the two subjects already had: the
//! parent is `ui.json`'s schema, and none of this reads or writes a document.
//!
//! Both halves live here rather than apart because they are the same fact in
//! two directions: [`Clock`] is where a reading comes from, and the rest is the
//! one spelling that reading is rendered in and parsed back out of. Keeping
//! them together is what keeps yog free of a `chrono`/`time` dependency without
//! that freedom being spread over two files.

use std::time::Instant;

/// Injected time (§7.2: "all timing is clock-injected"). `ui.json` itself is
/// untimed (write-through, see [`super`]); the seam lives here as the crate's
/// **one** time injection, consumed by the §7.2 derivation worker, its sweep
/// schedule and the §10 probe TTL cache.
///
/// Two readings, one source. [`now`](Clock::now) is monotonic — only
/// differences between calls matter (debounce windows, sweep deadlines,
/// snapshot age). [`stamp`](Clock::stamp) is the wall-clock `ops.jsonl` field
/// (§4.2), opaque to `opslog`: it exists here because §7.2's worker writes its
/// own drift lines off the frame thread, and a second time seam for the string
/// would be a second thing to inject and fake.
///
/// `Send + Sync` because the worker thread holds the same `Arc<dyn Clock>` the
/// frame injected (§7.2) — the schedule it gates and the test that advances it
/// are on different threads by construction.
pub trait Clock: Send + Sync {
    fn now(&self) -> Instant;
    /// The wall-clock `ops.jsonl` timestamp (§4.2) — unix seconds as a string,
    /// the crate's timestamp convention.
    fn stamp(&self) -> String;
    /// The same wall clock as an integer — the unit every boundary derivation
    /// dates against (`now_unix`) and the one a snapshot stamps its completion
    /// in (bl-b4b5). A default method rather than a second implementation,
    /// because there is one clock reading and this is only how it is spelled:
    /// an unparsable stamp is epoch zero, which is what every reader of a
    /// missing timestamp already treats it as.
    fn unix(&self) -> i64 {
        self.stamp().parse().unwrap_or(0)
    }
}

#[derive(Clone, Copy)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> Instant {
        Instant::now()
    }
    fn stamp(&self) -> String {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_secs())
            .to_string()
    }
}

/// The crate's **one** human-timestamp spelling, ISO 8601 extended:
/// `YYYY-MM-DD HH:MM:SSZ`. Assembled from already-decomposed calendar fields
/// so every caller — the chat header's when-seat (bl-16da, whose id already
/// carries `y/mo/d/h/mi/s` as digit groups) and the activity row's leading
/// column (bl-61db, whose `ts` is raw epoch seconds) — renders through this
/// one line rather than two independently-written format strings that could
/// drift apart.
pub(crate) fn format_iso8601(
    year: i64,
    month: i64,
    day: i64,
    hour: i64,
    minute: i64,
    second: i64,
) -> String {
    format!("{year:04}-{month:02}-{day:02} {hour:02}:{minute:02}:{second:02}Z")
}

/// Unix epoch seconds → [`format_iso8601`] (bl-61db: the activity row's raw
/// `1785630266` rendered as `2026-08-02 00:24:26Z`). Proleptic Gregorian, UTC,
/// no leap seconds — Howard Hinnant's `civil_from_days`
/// (<https://howardhinnant.github.io/date_algorithms.html>), the crate's one
/// calendar routine so this stays free of a `chrono`/`time` dependency.
pub fn iso8601_extended(epoch_secs: i64) -> String {
    let days = epoch_secs.div_euclid(86_400);
    let secs_of_day = epoch_secs.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    format_iso8601(
        year,
        month,
        day,
        secs_of_day / 3600,
        (secs_of_day % 3600) / 60,
        secs_of_day % 60,
    )
}

/// Days since the Unix epoch (1970-01-01) → `(year, month, day)`, proleptic
/// Gregorian. Ported verbatim from Hinnant's `civil_from_days` (public
/// domain), which is exact for the whole `i64` range this crate ever sees.
fn civil_from_days(z: i64) -> (i64, i64, i64) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365; // [0, 399]
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let day = doy - (153 * mp + 2) / 5 + 1; // [1, 31]
    let month = if mp < 10 { mp + 3 } else { mp - 9 }; // [1, 12]
    let year = if month <= 2 { y + 1 } else { y };
    (year, month, day)
}

/// The inverse of [`iso8601_extended`], for the one timestamp yog reads back
/// rather than prints: lernie's step `meta.json` `started_at`/`ended_at`
/// (§3.9, bl-40ab). `2026-04-22T06:54:32Z` → epoch seconds.
///
/// **Deliberately not an RFC 3339 parser.** It accepts exactly the shape
/// lernie's clock writes (`prompt/clock.rs`) — four digits, `-`, two, `-`,
/// two, `T`, two, `:`, two, `:`, two, `Z`, and nothing else — because that
/// clock is the only writer this crate ever reads, and a tolerant parser would
/// invent an answer for bytes no lernie produced. Anything else is `None`, the
/// same honest unknown a missing `meta.json` gives.
pub fn epoch_from_iso8601(stamp: &str) -> Option<i64> {
    let b = stamp.as_bytes();
    if b.len() != 20 || b.last() != Some(&b'Z') {
        return None;
    }
    let at = |a: usize, z: usize| stamp.get(a..z)?.parse::<i64>().ok();
    let sep = |i: usize, c: u8| (b.get(i) == Some(&c)).then_some(());
    sep(4, b'-')?;
    sep(7, b'-')?;
    sep(10, b'T')?;
    sep(13, b':')?;
    sep(16, b':')?;
    let secs = at(11, 13)? * 3600 + at(14, 16)? * 60 + at(17, 19)?;
    Some(days_from_civil(at(0, 4)?, at(5, 7)?, at(8, 10)?) * 86_400 + secs)
}

/// `(year, month, day)` → days since the Unix epoch, proleptic Gregorian.
/// Hinnant's `days_from_civil` (public domain), the exact inverse of
/// [`civil_from_days`] and the second half of the crate's one calendar
/// routine — both directions here so neither grows a `chrono` dependency.
fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let y = if month <= 2 { year - 1 } else { year };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400; // [0, 399]
    let mp = if month > 2 { month - 3 } else { month + 9 }; // [0, 11]
    let doy = (153 * mp + 2) / 5 + day - 1; // [0, 365]
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy; // [0, 146096]
    era * 146_097 + doe - 719_468
}
