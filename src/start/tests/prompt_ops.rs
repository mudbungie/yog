//! **What a detached fire writes down** (§4.2, §8.1): the ops line a clean
//! launch logs, the per-spawn stderr sink the driver's own words land in, the
//! clipped goal that keeps the line under `CAP`, and bl-afa9's two spawn
//! failures — a fork that never landed is a synthetic-failure row, never the
//! `-2` handoff.
//!
//! [`super::prompt`] owns the other half of the same gesture (what the child is
//! handed) and the fixtures both suites fire through.

use super::prompt::{fire, make_fifo, workspace};
use super::{World, write_exec};
use crate::cli_outbound::Cli;
use crate::opslog::SYNTHETIC_EXIT;
use crate::start::{DETACHED_EXIT, execute_prompt};

#[test]
fn prompt_routes_the_drivers_stderr_to_its_per_spawn_sink() {
    let w = World::new();
    let fifo = w.bin.path().join("report");
    make_fifo(&fifo);
    // A driver that refuses and dies the moment it launches — the §13.3 hole:
    // the spawn itself succeeds, so the ops line is a clean `-2`.
    let body = format!(
        "#!/bin/sh\nprintf 'refusing: version skew\\n' >&2\nprintf done > '{}'\n",
        fifo.display()
    );
    let lernie = Cli::new(write_exec(w.bin.path(), "lernie", &body));
    let ws = workspace(&w);
    execute_prompt(
        &lernie,
        w.state.path(),
        "TS",
        &fire(&ws, "n", None, "g"),
        &[],
        &super::rng(),
    )
    .unwrap();
    assert_eq!(std::fs::read_to_string(&fifo).unwrap(), "done");

    let e = &w.ops()[0];
    assert!(
        e.stderr.is_empty(),
        "the *logged* line records only the launch"
    );
    // The evidence lives in the sink, keyed by the same ts + workspace, and the
    // read-time fold turns the row into the failure the operator must see.
    let folded = crate::opslog::detached::fold(w.state.path(), e);
    assert_eq!(folded.stderr, "refusing: version skew\n");
    assert!(crate::opslog::OpRow::from(&folded).failed());
}

#[test]
fn prompt_clips_a_large_logged_goal() {
    let w = World::new();
    let lernie = Cli::new(super::fake_lernie(w.bin.path()));
    let ws = workspace(&w);
    // Each byte JSON-escapes to `\u00XX` (6 bytes): a raw-byte clip would let this
    // serialize to ~54 KB. The clip must hold POST-escape or the `ops.jsonl` line
    // blows past CAP/PIPE_BUF and loses its two-instance atomic append (§4.2).
    let big = "\u{1}".repeat(9000);
    execute_prompt(
        &lernie,
        w.state.path(),
        "TS",
        &fire(&ws, "n", None, &big),
        &[],
        &super::rng(),
    )
    .unwrap();
    let entry = &w.ops()[0];
    assert!(
        entry.argv[5].contains("bytes elided"),
        "the logged goal is clipped (§4.2)"
    );
    assert!(
        crate::opslog::build_line(entry).len() <= crate::opslog::CAP,
        "the serialized ops line stays ≤ CAP after JSON escaping"
    );
}

/// bl-afa9: a fork that never landed is NOT a detached handoff. It logs the
/// §4.2 synthetic-failure line, so the trail can tell "never started" from
/// "launched and running" — the two facts that used to share `-2`.
#[test]
fn prompt_logs_the_spawn_failure_and_returns_err() {
    let w = World::new();
    let lernie = Cli::new("/definitely/not/a/real/lernie-prompt");
    let ws = workspace(&w);
    let err = execute_prompt(
        &lernie,
        w.state.path(),
        "TS",
        &fire(&ws, "n", None, "g"),
        &[],
        &super::rng(),
    )
    .is_err();
    assert!(err, "a missing binary is surfaced");
    let e = &w.ops()[0];
    assert_eq!(e.exit, SYNTHETIC_EXIT);
    assert_ne!(e.exit, DETACHED_EXIT, "never started is not a handoff");
    assert!(!e.stderr.is_empty(), "the spawn error rides in stderr");
    let row = crate::opslog::OpRow::from(e);
    assert!(row.failed());
    assert_eq!(row.exit_label(), "failed to spawn — never started");
    // The intended argv is preserved, so the §6 retirement keys on the same
    // verb a later successful prompt writes.
    assert_eq!(e.argv[1], "prompt");
}

/// bl-afa9's second finding, the operator's actual case: Enter with a work
/// directory that is not there. The fork fails between fork and exec, and the
/// row must say so — not `-2`, which would claim the driver is running.
#[test]
fn a_nonexistent_work_directory_logs_failed_to_spawn_not_a_handoff() {
    let w = World::new();
    let lernie = Cli::new(super::fake_lernie(w.bin.path()));
    let missing = w.yog.path().join("no-such-dir");
    let err = execute_prompt(
        &lernie,
        w.state.path(),
        "TS",
        &fire(&missing, "n", None, "g"),
        &[],
        &super::rng(),
    )
    .is_err();
    assert!(err, "a bad cwd is surfaced");
    let e = &w.ops()[0];
    assert_eq!(e.exit, SYNTHETIC_EXIT);
    assert!(
        e.stderr.contains("work directory does not exist"),
        "the cwd is blamed, not the binary (bl-6191): {}",
        e.stderr
    );
    assert_eq!(
        crate::opslog::OpRow::from(e).exit_label(),
        "failed to spawn — never started"
    );
}
