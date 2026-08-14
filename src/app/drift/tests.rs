//! The drift instrumentation's own beats (§7.2): the per-kind fold, the two
//! thresholds a pass and a snapshot are judged against, and the edge test that
//! keeps a permanently-late derivation one event rather than one row a sweep
//! (bl-4b28). Split from [`super`] at §12's per-file budget on the file's own
//! seam — that side states what drift *is*, this side drives it.

use super::*;
use crate::opslog::{DRIFT_EXIT, YOG_DRIFT};

fn p(s: &str) -> PathBuf {
    PathBuf::from(s)
}

#[test]
fn nothing_found_writes_nothing() {
    assert!(entries("TS", "/state", &[]).is_empty());
}

#[test]
fn one_line_per_kind_carries_every_root_it_names() {
    let found = [
        Drift::Unannounced(p("/ws/b")),
        Drift::Desync(p("/ws/a")),
        Drift::Unannounced(p("/ws/a")),
        Drift::Unannounced(p("/ws/b")), // a duplicate is one finding
    ];
    let out = entries("TS", "/state", &found);
    assert_eq!(out.len(), 2, "two kinds, two lines: {out:?}");
    assert_eq!(
        out[0].argv,
        vec![YOG_DRIFT.to_string(), "desync".to_string()]
    );
    assert_eq!(out[0].stderr, "/ws/a\n");
    assert_eq!(out[0].cwd, "/state");
    assert_eq!(out[0].ts, "TS");
    assert_eq!(out[0].exit, DRIFT_EXIT);
    assert_eq!(
        out[1].argv,
        vec![YOG_DRIFT.to_string(), "unannounced".to_string()]
    );
    assert_eq!(out[1].stderr, "/ws/a\n/ws/b\n", "both roots, deduped");
}

#[test]
fn every_kind_names_itself_and_its_root() {
    let all = [
        Drift::Desync(p("/a")),
        Drift::Unannounced(p("/b")),
        Drift::Unenumerated(p("/c")),
        Drift::Late(p("/state"), 7),
    ];
    let kinds: Vec<&str> = all.iter().map(Drift::kind).collect();
    assert_eq!(kinds, ["desync", "unannounced", "unenumerated", "late"]);
    let roots: Vec<&Path> = all.iter().map(Drift::root).collect();
    assert_eq!(
        roots,
        [
            Path::new("/a"),
            Path::new("/b"),
            Path::new("/c"),
            Path::new("/state")
        ]
    );
    assert_eq!(entries("TS", "/state", &all).len(), 4);
}

/// bl-4b28 — the edge, exhaustively: the trail takes the pass that *starts*
/// a late run and nothing else. 4447 restatements of one fact is not a
/// record, it is a log file with the record buried in it.
#[test]
fn only_the_pass_that_stops_keeping_cadence_is_a_finding() {
    assert_eq!(late_edge(Some(3), false), Some(3), "the edge into lateness");
    assert_eq!(
        late_edge(Some(3), true),
        None,
        "still late says nothing new"
    );
    assert_eq!(late_edge(None, true), None, "recovery is not a drift row");
    assert_eq!(late_edge(None, false), None, "and neither is keeping it");
}

#[test]
fn a_late_pass_carries_how_late_it_was() {
    let out = entries("TS", "/state", &[Drift::Late(p("/state"), 12)]);
    assert_eq!(out[0].argv, vec![YOG_DRIFT.to_string(), "late".to_string()]);
    assert_eq!(out[0].stderr, "/state (12 s pass)\n");
}

/// bl-4b28 — a pass is judged by the sweep it ran: the cheap period for a
/// cheap pass (and for one that swept nothing), the full period for a full
/// sweep, which is the pass that re-derives every workspace and is the
/// reason that period is longer in the first place.
#[test]
fn the_promise_a_pass_is_held_to_is_its_own_sweeps_period() {
    use crate::app::dirty::Sweep;
    let c = crate::app::Cadence::default();
    assert_eq!(c.late_pass(Sweep::Full), c.full_sweep);
    assert_eq!(c.late_pass(Sweep::Cheap), c.cheap_sweep);
    assert_eq!(c.late_pass(Sweep::None), c.cheap_sweep);
    // The pass the storm was made of: a full sweep taking longer than the
    // cheap period is the schedule working, not drift.
    let t0 = std::time::Instant::now();
    let three_seconds = t0 + Duration::from_secs(3);
    assert_eq!(
        lateness(t0, three_seconds, c.late_pass(Sweep::Cheap)),
        Some(3)
    );
    assert_eq!(lateness(t0, three_seconds, c.late_pass(Sweep::Full)), None);
}

#[test]
fn lateness_fires_only_at_the_cadence_it_promised() {
    let late_pass = crate::app::Cadence::default().late_pass(crate::app::dirty::Sweep::Cheap);
    let t0 = std::time::Instant::now();
    let a_hair_early = (t0 + late_pass)
        .checked_sub(Duration::from_millis(1))
        .unwrap();
    assert_eq!(lateness(t0, a_hair_early, late_pass), None);
    assert_eq!(
        lateness(t0, t0 + late_pass, late_pass),
        Some(late_pass.as_secs())
    );
    // A non-monotonic injected clock cannot underflow into a false alarm.
    assert_eq!(lateness(t0 + late_pass, t0, late_pass), None);
}

#[test]
fn the_staleness_line_is_silent_until_two_full_sweeps_are_missed() {
    let stale = crate::app::Cadence::default().stale_after();
    assert_eq!(stale_label(Duration::from_secs(0), stale), None);
    let a_hair_fresh = stale.checked_sub(Duration::from_millis(1)).unwrap();
    assert_eq!(stale_label(a_hair_fresh, stale), None);
    assert_eq!(
        stale_label(stale, stale),
        Some(format!("derivation {} s behind", stale.as_secs()))
    );
}
