//! The time seam's own tests, beside `ui_state::clock` — the system clock's two
//! readings, the crate's one ISO 8601 spelling and the calendar math both
//! directions of it ride on. Split from [`super`] at §12's budget on the seam
//! the production module was split on: nothing here touches a document.

use crate::ui_state::{Clock, SystemClock, epoch_from_iso8601, format_iso8601, iso8601_extended};

#[test]
fn system_clock_is_monotonic_and_stamps_unix_seconds() {
    let c = SystemClock;
    let a = c.now();
    assert!(c.now() >= a);
    // The §4.2 wall-clock field: unix seconds as a string, opaque to opslog.
    // Any real clock is well past the epoch, and the parse is the contract.
    let stamp: u64 = c.stamp().parse().expect("unix seconds");
    assert!(stamp > 1_700_000_000, "a plausible wall clock: {stamp}");
}

#[test]
fn format_iso8601_pads_every_field() {
    assert_eq!(
        format_iso8601(2026, 8, 2, 0, 24, 26),
        "2026-08-02 00:24:26Z"
    );
    assert_eq!(format_iso8601(1970, 1, 1, 0, 0, 0), "1970-01-01 00:00:00Z");
}

/// bl-61db: the activity row's raw epoch, rendered the same way the chat
/// header renders its id's stamp (bl-16da) — `date -u -d @1785630266` is the
/// independent oracle for this value.
#[test]
fn iso8601_extended_reads_the_activity_row_example() {
    assert_eq!(iso8601_extended(1_785_630_266), "2026-08-02 00:24:26Z");
}

#[test]
fn iso8601_extended_covers_the_epoch_and_a_leap_day() {
    assert_eq!(iso8601_extended(0), "1970-01-01 00:00:00Z");
    // 2000 is a leap year (divisible by 400) — `date -u -d @951868799`.
    assert_eq!(iso8601_extended(951_868_799), "2000-02-29 23:59:59Z");
    // year-end rollover — `date -u -d @1735689599`.
    assert_eq!(iso8601_extended(1_735_689_599), "2024-12-31 23:59:59Z");
}

/// The calendar routine's inverse (§3.9, bl-40ab): the one timestamp yog reads
/// back rather than prints, round-tripped against `iso8601_extended` at the
/// epoch, a leap day and a year-end rollover — the same three oracles the
/// forward direction is pinned on.
#[test]
fn epoch_from_iso8601_inverts_the_extended_rendering() {
    for secs in [0_i64, 951_868_799, 1_735_689_599, 1_785_630_266] {
        let rendered = iso8601_extended(secs).replace(' ', "T");
        assert_eq!(epoch_from_iso8601(&rendered), Some(secs), "{rendered}");
    }
}

/// It accepts exactly lernie's clock shape and nothing else — a tolerant parse
/// would invent an answer for bytes no lernie wrote.
#[test]
fn epoch_from_iso8601_refuses_every_other_shape() {
    for bad in [
        "",
        "2026-04-22T06:54:32",       // no zone
        "2026-04-22T06:54:32+00:00", // an offset, not Z
        "2026-04-22 06:54:32Z",      // a space where the T belongs
        "2026/04/22T06:54:32Z",      // the wrong date separators
        "2026-04-22T06.54.32Z",      // the wrong time separators
        "20xx-04-22T06:54:32Z",      // not digits
        "2026-04-22T06:54:3Z",       // short by one
    ] {
        assert_eq!(epoch_from_iso8601(bad), None, "{bad:?}");
    }
}
