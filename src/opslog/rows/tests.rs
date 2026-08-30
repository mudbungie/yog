//! [`super::OpRow`]'s own tables: the collapsed summary's newline fold and its
//! §11-scan elision (bl-0bf9, cut through the middle since bl-3aa1 so a column
//! of ops that share a long invariant path prefix stays readable), the human
//! timestamp column (bl-61db), and the [`super::SurfaceFailure`] projection.
//! Pure — no filesystem, no clock. Split from the module at the §12 line cap on
//! this directory's own seam (`line/tests.rs`, `detached/tests.rs`).

use super::*;

fn entry(exit: i32, stderr: &str) -> OpEntry {
    OpEntry {
        ts: "TS".into(),
        argv: vec!["bl".into(), "close".into(), "bl-4db6".into()],
        cwd: "/proj".into(),
        exit,
        stdout: "out".into(),
        stderr: stderr.into(),
        origin: Origin::default(),
    }
}

#[test]
fn op_row_carries_the_full_entry_and_joins_argv() {
    let row = OpRow::from(&entry(0, ""));
    assert_eq!(row.ts, "TS");
    assert_eq!(row.argv, "bl close bl-4db6");
    assert_eq!(row.cwd, "/proj");
    assert_eq!(row.exit, 0);
    assert_eq!(row.stdout, "out");
    assert_eq!(row.stderr, "");
}

/// bl-61db: the leading column reads `when()`, not the raw `ts`. An
/// unparseable `ts` (the "TS" fixture above; every real line is a decimal
/// epoch) falls back to itself rather than a made-up date.
#[test]
fn when_renders_iso8601_and_falls_back_on_a_non_numeric_ts() {
    assert_eq!(OpRow::from(&entry(0, "")).when(), "TS");
    let stamped = OpEntry {
        ts: "1785630266".into(),
        ..entry(0, "")
    };
    assert_eq!(OpRow::from(&stamped).when(), "2026-08-02 00:24:26Z");
}

#[test]
fn has_output_reflects_either_stream() {
    assert!(OpRow::from(&entry(0, "")).has_output()); // stdout "out"
    let neither = OpEntry {
        stdout: String::new(),
        ..entry(0, "")
    };
    assert!(!OpRow::from(&neither).has_output());
    let only_err = OpEntry {
        stdout: String::new(),
        ..entry(1, "boom")
    };
    assert!(OpRow::from(&only_err).has_output());
}

/// bl-0bf9: a short, single-line `argv` renders as itself — no elision, no
/// trailing `…`, when it already fits the cap.
#[test]
fn summary_leaves_a_short_argv_unchanged() {
    assert_eq!(OpRow::from(&entry(0, "")).summary(), "bl close bl-4db6");
}

/// bl-0bf9 acceptance: a 500-char multi-line goal (a prompt op's payload —
/// a ball body reaches this length routinely) renders as ONE elided
/// single-line summary of bounded length — the collapsed row's whole point
/// — while the row's own `argv` field stays byte-exact, because the
/// expansion (§4.2) must never lose bytes, only the collapsed summary
/// elides.
#[test]
fn summary_folds_newlines_and_elides_a_long_multiline_goal() {
    let goal = format!(
        "identity preamble\n{}\nball worktree preamble",
        "payload segment ".repeat(30)
    );
    assert!(goal.chars().count() >= 500, "fixture must exceed 500 chars");
    let e = OpEntry {
        argv: vec![goal.clone()],
        ..entry(0, "")
    };
    let row = OpRow::from(&e);

    let summary = row.summary();
    assert!(!summary.contains('\n') && !summary.contains('\r'));
    assert_eq!(summary.chars().count(), SUMMARY_ARGV_MAX);
    assert!(summary.contains('…'), "the cut is marked: {summary}");
    // bl-3aa1: the cut takes the MIDDLE. Both ends of the goal survive —
    // the head that says what the op is, and the tail that tells this row
    // from the next one. Asserting only `ends_with('…')` is what let the
    // wrong end be kept for as long as it was.
    assert!(summary.starts_with("identity preamble"), "{summary}");
    assert!(summary.ends_with("ball worktree preamble"), "{summary}");

    // The trail never loses bytes: only the collapsed summary elides.
    assert_eq!(row.argv, goal);
}

/// bl-0bf9: the elision boundary is exact — a flat argv of exactly the cap
/// passes through whole; one char over triggers the `…`.
#[test]
fn summary_elision_boundary_is_exact() {
    let at_cap = "a".repeat(SUMMARY_ARGV_MAX);
    let e = OpEntry {
        argv: vec![at_cap.clone()],
        ..entry(0, "")
    };
    assert_eq!(OpRow::from(&e).summary(), at_cap);

    let over_cap = "a".repeat(SUMMARY_ARGV_MAX + 1);
    let e = OpEntry {
        argv: vec![over_cap],
        ..entry(0, "")
    };
    let summary = OpRow::from(&e).summary();
    assert_eq!(summary.chars().count(), SUMMARY_ARGV_MAX);
    assert!(summary.contains('…'));
}

/// bl-3aa1, the defect itself: two ops that differ only in their TAIL —
/// the shape every activity row really has, a long invariant path prefix
/// and a distinguishing workspace/agent at the end — must not render as
/// the same string. Head-keeping elision collapsed them, which is what
/// made a column of different operations scan as one repeated line.
#[test]
fn two_ops_differing_only_in_their_tail_do_not_summarize_alike() {
    let prefix = "litany prompt --name growing \
         /home/u/.cache/yog-drive/quality-20260807T214407Z/data/yog/workspaces/";
    let summarize = |leaf: &str| {
        let e = OpEntry {
            argv: vec![format!("{prefix}{leaf}")],
            ..entry(0, "")
        };
        OpRow::from(&e).summary()
    };
    let home = summarize("home 20260807T214551Z-2a1181a3");
    let other = summarize("scratch 20260807T220107Z-c0ffeeba");
    assert!(
        home.chars().count() == SUMMARY_ARGV_MAX && other.chars().count() == SUMMARY_ARGV_MAX,
        "both rows really are over the cap, so this is not a vacuous pass"
    );
    assert_ne!(home, other, "the rows must stay distinguishable");
    assert!(home.ends_with("home 20260807T214551Z-2a1181a3"), "{home}");
    assert!(
        other.ends_with("scratch 20260807T220107Z-c0ffeeba"),
        "{other}"
    );
}

#[test]
fn surface_failure_carries_argv_and_stderr_tail() {
    let row = OpRow::from(&entry(2, "line1\nline2"));
    let f = SurfaceFailure::from(&row);
    assert_eq!(f.argv, "bl close bl-4db6");
    assert_eq!(f.stderr_tail, "line1\nline2");
}

#[test]
fn stderr_tail_keeps_only_the_last_lines() {
    assert_eq!(stderr_tail(""), "");
    assert_eq!(stderr_tail("only\n"), "only");
    assert_eq!(stderr_tail("a\nb\nc\nd\ne\n"), "c\nd\ne");
}
