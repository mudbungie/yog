//! STORIES **S6-T5** activity-chip: an ops fixture with k failures gives a chip
//! carrying the op count and the live-failure count; an expanded row yields
//! argv / cwd / exit / stderr **verbatim** — including an action that never
//! spawned — and a line clipped by the §4.2 cap renders its `truncated` marker
//! rather than silently short bytes (STORIES S6.5, DESIGN §4.2, §11).
//!
//! **Two premises drifted.** The row says the chip carries `N ops · k ⚠`; since
//! the §11 glyph doctrine the chip has room to say the outcome outright and
//! takes both the word and the glyph from `theme::op_badge`, so it reads
//! `… · k failed ⚠`. And the ⚠ axis is **quieted by the operator's ack**
//! (bl-c417) while `total` is not — the chip may never claim fewer ops than the
//! pane below it lists.

#![allow(clippy::unwrap_used)]

use tempfile::tempdir;
use yog::opslog::{self, OpEntry, OpOutcome, OpRow, Origin};

/// A completed op row.
fn op(ts: &str, argv: &[&str], cwd: &str, exit: i32, stderr: &str) -> OpEntry {
    OpEntry {
        ts: ts.to_owned(),
        argv: argv.iter().map(|s| (*s).to_owned()).collect(),
        cwd: cwd.to_owned(),
        exit,
        stdout: String::new(),
        stderr: stderr.to_owned(),
        origin: Origin::Balls,
    }
}

/// STORIES **S6-T5** activity-chip.
#[test]
fn s6_t5_the_chip_counts_and_every_row_expands_verbatim() {
    let state = tempdir().unwrap();
    let gate = "pre-commit: coverage 92% < 100%\naborting close";
    for entry in [
        op("1000", &["bl", "list", "--json"], "/proj", 0, ""),
        op(
            "1001",
            &["bl", "close", "bl-1", "--as", "cobalt"],
            "/proj",
            1,
            gate,
        ),
        op(
            "1002",
            &["lernie", "scan", "/ws"],
            "/ws",
            2,
            "no such workspace",
        ),
    ] {
        opslog::append(state.path(), &entry).unwrap();
    }
    // An action that never spawned still leaves a rendered fact (§4.2 amended) —
    // a trail that hides *why* is not a trail.
    opslog::append(
        state.path(),
        &OpEntry::synthetic_failure(
            "1003".to_owned(),
            vec!["bl".to_owned(), "claim".to_owned(), "bl-2".to_owned()],
            "/proj".to_owned(),
            "no such file or directory (os error 2)".to_owned(),
            Origin::Balls,
        ),
    )
    .unwrap();

    let entries = opslog::tail(state.path(), 64);
    assert_eq!(entries.len(), 4);
    let rows: Vec<OpRow> = entries.iter().map(OpRow::from).collect();

    // --- The chip. Every op counted; the live failures counted on their own axis.
    let activity = opslog::activity(&rows);
    assert_eq!(activity.total, 4);
    assert_eq!(
        activity.errors, 3,
        "two non-zero exits and one that never ran"
    );
    assert_eq!(activity.drifts, 0);
    let (glyph, _, phrase) = yog::theme::op_badge(OpOutcome::Failed);
    let chip = activity.chip();
    assert_eq!(chip, format!("activity · 4 ops · 3 {phrase} {glyph}"));
    assert!(chip.starts_with("activity · 4 ops"));

    // --- An expanded row: argv, cwd, exit and stderr, byte-exact.
    let close = rows.iter().find(|r| r.argv.contains("close")).unwrap();
    assert_eq!(
        close.argv, "bl close bl-1 --as cobalt",
        "argv is byte-exact"
    );
    assert_eq!(close.cwd, "/proj");
    assert_eq!(close.exit, 1);
    assert_eq!(close.stderr, gate, "the gate's reason rides back verbatim");
    assert!(close.has_output(), "a row with output is worth opening");

    // --- Including the action that never spawned: it has a row, an argv, and a
    // reason, and it is a failure like any other.
    let never = rows.iter().find(|r| r.argv.contains("claim")).unwrap();
    assert_eq!(never.exit, opslog::SYNTHETIC_EXIT);
    assert_eq!(never.argv, "bl claim bl-2");
    assert!(never.stderr.contains("no such file"));
    assert!(
        never.failed(),
        "an attempt that never ran is a rendered failure"
    );

    // --- The §4.2 cap on the captured streams: a line too long to store says
    // so, rather than handing back short bytes that read as complete.
    let noisy = "e".repeat(opslog::CAP * 2);
    let long = op("1004", &["lernie", "scan", "/ws"], "/ws", 1, &noisy);
    let line = String::from_utf8(opslog::build_line(&long)).unwrap();
    assert!(
        line.contains(r#""truncated":true"#),
        "a clipped line marks itself"
    );
    assert!(line.len() <= opslog::CAP, "and it really was clipped");
    assert!(!line.contains(&noisy), "the full stderr is not in there");
    // An ordinary line carries no marker — the marker means something.
    let short = String::from_utf8(opslog::build_line(&op(
        "1005",
        &["bl", "list"],
        "/proj",
        0,
        "",
    )))
    .unwrap();
    assert!(
        !short.contains("truncated"),
        "no marker where nothing was cut"
    );

    // argv is never truncated inside the serializer — a pathological argv is the
    // one unavoidable overflow — so the deliberately-large field (a composed
    // `lernie prompt` goal, §8.1) is clipped at its source, and says how much it
    // dropped rather than trailing off.
    let goal = "g".repeat(opslog::CAP * 2);
    let prompt = op("1006", &["lernie", "prompt", &goal], "/ws", 0, "");
    let clipped = opslog::clip_goal(&prompt);
    let kept = clipped.argv.last().unwrap();
    assert!(kept.len() < goal.len(), "the goal was clipped");
    assert!(
        kept.contains("bytes elided"),
        "and it names what it dropped"
    );
    assert!(opslog::build_line(&clipped).len() <= opslog::CAP);
}
