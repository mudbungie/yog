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

use OpOutcome::{Clean, Detached, Failed, Notice, Retired};

/// A lernie notice line, verbatim (bl-1296) — the sink content that used to
/// make a `-2` row a rendered failure.
const NOTICE_LINE: &str = "lernie: exit launch for c-1: no such file \
     (accepted crash class, ARCH §2.11)\n";

/// A detached `-2` row with `stderr` already folded in from its §8.1 sink
/// (`super::super::DETACHED_EXIT` — the [`row`] helper's exit==0 stderr rule
/// doesn't apply to sentinels, so this is separate rather than reused).
fn detached(argv: &str, cwd: &str, stderr: &str) -> OpRow {
    OpRow::from(&OpEntry {
        ts: "TS".into(),
        argv: argv.split(' ').map(String::from).collect(),
        cwd: cwd.into(),
        exit: super::super::DETACHED_EXIT,
        stdout: String::new(),
        stderr: stderr.into(),
        origin: Origin::default(),
    })
}

/// A clean detached handoff: the sentinel with the driver still silent.
fn handoff(argv: &str, cwd: &str) -> OpRow {
    detached(argv, cwd, "")
}

/// A `-2` row whose sink holds only lernie notice lines (bl-1296) — [`handoff`]
/// with the driver's benign words in place of the silence.
fn noticed(argv: &str, cwd: &str) -> OpRow {
    detached(argv, cwd, NOTICE_LINE)
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
    let died = detached("lernie prompt", "/ws", "refusing: version skew\n");
    assert_eq!(outcomes(&[died]), vec![Failed]);
}

/// **THE BALL** (bl-1296): a driver notice is its own outcome — never `Failed`
/// (nothing went wrong), never `Detached` (the driver did speak), never `Clean`
/// (nobody observed an exit).
#[test]
fn a_driver_notice_is_its_own_outcome() {
    assert_eq!(outcomes(&[noticed("lernie prompt", "/ws")]), vec![Notice]);
}

/// And it never reaches the chip's ⚠ count — which is the whole complaint: the
/// sink is append-only for the driver's life, so one benign line kept saying
/// `1 failed ⚠` on every sweep until the operator acked it.
#[test]
fn a_notice_does_not_move_the_chip_failed_count() {
    let a = activity(&[noticed("lernie prompt", "/ws")]);
    assert_eq!(
        a,
        Activity {
            total: 1,
            errors: 0,
            drifts: 0
        }
    );
    assert_eq!(a.chip(), "activity · 1 ops");
    assert!(!a.alarming(), "and the pane offers no Dismiss for it");
}

/// It retires an earlier failure of the same verb exactly as `Clean` and
/// `Detached` do (§6): it is the newest fact about that verb in this `cwd`.
#[test]
fn a_notice_retires_an_earlier_failure_of_the_same_verb() {
    let rows = [
        row("lernie prompt", "/ws", 2),
        noticed("lernie prompt", "/ws"),
    ];
    assert_eq!(outcomes(&rows), vec![Retired, Notice]);
    assert_eq!(activity(&rows).errors, 0);
}

/// The narrowing did not weaken the rule it narrowed: a sink that mixes a
/// notice with words nothing recognizes is still a death, and still counted.
#[test]
fn a_sink_mixing_a_notice_with_unrecognized_words_stays_a_failure() {
    let mixed = detached(
        "lernie prompt",
        "/ws",
        &format!("{NOTICE_LINE}lernie: brazen 0.0.2 != 0.0.3\n"),
    );
    assert_eq!(outcomes(std::slice::from_ref(&mixed)), vec![Failed]);
    assert_eq!(activity(&[mixed]).errors, 1);
}
