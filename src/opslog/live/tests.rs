//! Unit tests for the retirement projection and activity summary (live).

use super::super::{OpEntry, Origin};
use super::*;

/// One row: `argv` split on spaces, run in `cwd`, exiting `exit`.
fn row(argv: &str, cwd: &str, exit: i32) -> OpRow {
    OpRow::from(&OpEntry {
        ts: "TS".into(),
        argv: argv.split(' ').map(String::from).collect(),
        cwd: cwd.into(),
        exit,
        stdout: String::new(),
        stderr: if exit == 0 {
            String::new()
        } else {
            "boom".into()
        },
        origin: Origin::default(),
    })
}

use OpOutcome::{Clean, Detached, Failed, Retired};

/// A clean detached handoff: `-2` with empty stderr (`super::super::DETACHED_EXIT`
/// — the [`row`] helper's exit==0 stderr rule doesn't apply to sentinels, so this
/// is separate rather than reused).
fn handoff(argv: &str, cwd: &str) -> OpRow {
    OpRow::from(&OpEntry {
        ts: "TS".into(),
        argv: argv.split(' ').map(String::from).collect(),
        cwd: cwd.into(),
        exit: super::super::DETACHED_EXIT,
        stdout: String::new(),
        stderr: String::new(),
        origin: Origin::default(),
    })
}

#[test]
fn a_later_clean_run_of_the_same_verb_retires_the_failure() {
    let rows = [row("lernie prime", "/w", 2), row("lernie prime", "/w", 0)];
    assert_eq!(outcomes(&rows), vec![Retired, Clean]);
    assert_eq!(activity(&rows).errors, 0, "the chip counts no live failure");
}

#[test]
fn a_failure_with_no_later_clean_run_stays_live() {
    let rows = [row("lernie prime", "/w", 0), row("lernie prime", "/w", 2)];
    assert_eq!(outcomes(&rows), vec![Clean, Failed]);
    assert_eq!(activity(&rows).errors, 1);
}

#[test]
fn retirement_is_scoped_to_the_cwd_it_ran_in() {
    let rows = [row("lernie prime", "/a", 2), row("lernie prime", "/b", 0)];
    assert_eq!(
        outcomes(&rows),
        vec![Failed, Clean],
        "a clean run elsewhere leaves this project's failure live"
    );
}

#[test]
fn the_verb_key_is_binary_plus_subcommand_not_the_operands() {
    // Same verb, different ball id → the later clean close retires the failure.
    let same = [row("bl close bl-1", "/p", 1), row("bl close bl-2", "/p", 0)];
    assert_eq!(outcomes(&same), vec![Retired, Clean]);
    // A different subcommand is a different verb → nothing is retired.
    let other = [
        row("bl close bl-1", "/p", 1),
        row("bl list --json", "/p", 0),
    ];
    assert_eq!(outcomes(&other), vec![Failed, Clean]);
    // A bare one-token argv keys on itself.
    let bare = [row("bz", "/p", 1), row("bz", "/p", 0)];
    assert_eq!(outcomes(&bare), vec![Retired, Clean]);
}

#[test]
fn only_live_failures_reach_the_chip() {
    let rows = [
        row("lernie prime", "/w", 2), // retired below
        row("bl close bl-1", "/p", 1),
        row("lernie prime", "/w", 0),
    ];
    let a = activity(&rows);
    assert_eq!(
        a,
        Activity {
            total: 3,
            errors: 1,
            drifts: 0
        }
    );
    // §11: the count is said in words, glyph on top — and the word is the
    // badge mapping's, so chip and rows can never disagree.
    assert_eq!(a.chip(), "activity · 3 ops · 1 failed ⚠");
    assert!(a.chip().contains(crate::theme::op_badge(Failed).2));
    assert_eq!(activity(&rows[2..]).chip(), "activity · 1 ops");
    assert_eq!(activity(&[]).chip(), "activity · 0 ops");
    assert!(outcomes(&[]).is_empty());
}

/// bl-8433: a handed-off spawn is its own outcome — never the `Clean` badge
/// ("ran clean" for an exit nobody observed is a lie) and never `Failed`
/// (nothing has actually gone wrong).
#[test]
fn a_clean_handoff_is_its_own_outcome_not_clean_and_not_failed() {
    let rows = [handoff("lernie prompt", "/ws")];
    assert_eq!(outcomes(&rows), vec![Detached]);
}

/// bl-8433: the chip's ⚠ count must not inflate on a handoff — it is not a
/// failure — and the total still counts the row.
#[test]
fn a_handoff_does_not_move_the_chip_failed_count() {
    let rows = [handoff("lernie prompt", "/ws")];
    let a = activity(&rows);
    assert_eq!(
        a,
        Activity {
            total: 1,
            errors: 0,
            drifts: 0
        }
    );
    assert_eq!(a.chip(), "activity · 1 ops");
}

/// bl-8433 ruling: a handoff retires an earlier failure of the same verb
/// exactly like a clean run does (§6) — it is the newest fact about that verb,
/// and a stale failure under it is no longer live.
#[test]
fn a_handoff_retires_an_earlier_failure_of_the_same_verb() {
    let rows = [
        row("lernie prompt", "/ws", 2),
        handoff("lernie prompt", "/ws"),
    ];
    assert_eq!(outcomes(&rows), vec![Retired, Detached]);
    assert_eq!(activity(&rows).errors, 0, "the retired failure is not live");
}

/// A driver that died *after* launching (stderr folded from its sink) is a
/// failure, not a `Detached` outcome — `OpRow::failed` already covers it, and
/// the rollup must not double-classify the same row two ways.
#[test]
fn a_post_launch_death_reads_failed_not_detached() {
    let died = OpRow::from(&OpEntry {
        ts: "TS".into(),
        argv: vec!["lernie".into(), "prompt".into()],
        cwd: "/ws".into(),
        exit: super::super::DETACHED_EXIT,
        stdout: String::new(),
        stderr: "refusing: version skew\n".into(),
        origin: Origin::default(),
    });
    assert_eq!(outcomes(&[died]), vec![Failed]);
}

#[test]
fn a_drift_row_is_counted_on_its_own_axis_never_as_a_failure() {
    // A drift line accuses the watcher, not the operator's last verb: it
    // must not read as a failure (that would hijack the §7.3 banner), and it
    // must not retire or be retired by anything.
    let drift = OpRow::from(&OpEntry::drift(
        "TS".into(),
        "unannounced",
        "/state".into(),
        "/ws/a\n".into(),
    ));
    assert!(drift.drift());
    assert!(!drift.failed(), "a drift is not a failed action");
    let rows = [row("bl close bl-1", "/p", 1), drift];
    let a = activity(&rows);
    assert_eq!(
        a,
        Activity {
            total: 2,
            errors: 1,
            drifts: 1
        }
    );
    assert_eq!(a.chip(), "activity · 2 ops · 1 failed ⚠ · 1 drift");
    assert_eq!(
        activity(&rows[1..]).chip(),
        "activity · 1 ops · 1 drift",
        "drift alone still reaches the chip"
    );
}
