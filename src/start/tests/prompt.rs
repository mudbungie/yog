//! The detached `lernie prompt` (§8.1, §3.3): the fire-time conversation mint
//! riding `--name`, the goal fired verbatim (bl-6920), the `YOG_NAME` layer, the
//! per-rung cwd, and the spawn-only ops line. A fifo rendezvous proves the
//! detached child ran and captures its argv + env.

use super::{World, write_exec};
use crate::cli_outbound::Cli;
use crate::opslog::{Origin, SYNTHETIC_EXIT};
use crate::start::{DETACHED_EXIT, Prepared, execute_prompt};
use crate::test_support::spawn_guard;
use std::path::Path;

/// The composer's fire-time params: workspace `name`, its path, the driver cwd.
fn prepared(name: &str, cwd: &Path, workspace: &Path) -> Prepared {
    Prepared {
        name: name.to_owned(),
        workspace: workspace.to_path_buf(),
        cwd: cwd.to_path_buf(),
        goal: String::new(),
        origin: Origin::Conversation,
    }
}

/// A blocking fifo — reading it rendezvous with the detached child's write.
fn make_fifo(path: &Path) {
    let status = crate::git_env::command(Path::new("mkfifo"))
        .args(["-m", "600"])
        .arg(path)
        .status()
        .expect("spawn mkfifo");
    assert!(status.success(), "mkfifo");
}

#[test]
fn prompt_fires_the_goal_verbatim_layers_yog_name_and_logs_the_sentinel() {
    let _g = spawn_guard();
    let w = World::new();
    let fifo = w.bin.path().join("report");
    make_fifo(&fifo);
    // The fake records the argv (verb/--name/name/ws/goal) and the layered
    // YOG_NAME.
    let body = format!(
        "#!/bin/sh\nprintf '%s\\037%s\\037%s\\037%s\\037%s\\037%s' \"$1\" \"$2\" \"$3\" \"$4\" \"$5\" \"$YOG_NAME\" > '{}'\n",
        fifo.display()
    );
    let lernie = Cli::new(write_exec(w.bin.path(), "lernie", &body));
    let ws = w.yog.path().join("ws");
    let cwd = w.balls.path(); // an existing dir → a valid detached cwd
    let conversation = execute_prompt(
        &lernie,
        w.state.path(),
        "TS",
        &prepared("cobalt-gecko", cwd, &ws),
        "do it",
        &[],
        &super::rng(),
    )
    .unwrap();

    let recorded = std::fs::read_to_string(&fifo).unwrap();
    let fields: Vec<&str> = recorded.split('\u{1f}').collect();
    assert_eq!(fields[0], "prompt");
    // The minted name rides `--name` — the lernie-stored fact home (§3.3 as
    // ruled by bl-50f3) — with the goal last so `clip_goal` still trims it.
    assert_eq!(fields[1], "--name");
    assert_eq!(fields[3], ws.to_string_lossy());
    // The goal reaches lernie exactly as the operator edited it (bl-6920):
    // no identity line, no mutation — `--name` is identity's only channel,
    // and lernie states the stored fact in its assembled context (lernie
    // bl-d55f). The workspace name rides `YOG_NAME` and nothing else.
    assert_eq!(fields[4], "do it", "the payload is unmutated");
    assert_eq!(fields[2], conversation, "--name carries the fired mint");
    assert_ne!(conversation, "cobalt-gecko");
    assert_eq!(fields[5], "cobalt-gecko", "YOG_NAME layered (§8)");

    let e = &w.ops()[0];
    assert_eq!(e.argv[1], "prompt");
    assert_eq!(e.exit, DETACHED_EXIT);
    assert_eq!(e.cwd, cwd.display().to_string());
    assert!(e.stderr.is_empty(), "a clean launch logs no error");
    // bl-afa9: and it *renders* as the handoff it is — never a numeric exit,
    // never the wording a spawn failure gets.
    let row = crate::opslog::OpRow::from(e);
    assert!(!row.failed());
    assert_eq!(
        row.exit_label(),
        "detached — handed off, no exit to observe"
    );
    // The logged argv mirrors the spawned one: `--name` + the mint, goal last.
    assert_eq!(&e.argv[2..4], ["--name", conversation.as_str()]);
    assert_eq!(
        e.argv[5], "do it",
        "the logged goal is the verbatim payload"
    );
}

#[test]
fn prompt_routes_the_drivers_stderr_to_its_per_spawn_sink() {
    let _g = spawn_guard();
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
    let ws = w.yog.path().join("ws");
    execute_prompt(
        &lernie,
        w.state.path(),
        "TS",
        &prepared("n", w.balls.path(), &ws),
        "g",
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
    let _g = spawn_guard();
    let w = World::new();
    let lernie = Cli::new(super::fake_lernie(w.bin.path()));
    let ws = w.yog.path().join("ws");
    // Each byte JSON-escapes to `\u00XX` (6 bytes): a raw-byte clip would let this
    // serialize to ~54 KB. The clip must hold POST-escape or the `ops.jsonl` line
    // blows past CAP/PIPE_BUF and loses its two-instance atomic append (§4.2).
    let big = "\u{1}".repeat(9000);
    execute_prompt(
        &lernie,
        w.state.path(),
        "TS",
        &prepared("n", w.balls.path(), &ws),
        &big,
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
    let _g = spawn_guard();
    let w = World::new();
    let lernie = Cli::new("/definitely/not/a/real/lernie-prompt");
    let ws = w.yog.path().join("ws");
    let err = execute_prompt(
        &lernie,
        w.state.path(),
        "TS",
        &prepared("n", w.balls.path(), &ws),
        "g",
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
    let _g = spawn_guard();
    let w = World::new();
    let lernie = Cli::new(super::fake_lernie(w.bin.path()));
    let ws = w.yog.path().join("ws");
    let missing = w.yog.path().join("no-such-dir");
    let err = execute_prompt(
        &lernie,
        w.state.path(),
        "TS",
        &prepared("n", &missing, &ws),
        "g",
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
