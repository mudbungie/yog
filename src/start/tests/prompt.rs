//! The detached `lernie prompt` (§8.1, §3.3): the fire-time conversation mint
//! riding `--name`, the goal fired verbatim (bl-6920), the `YOG_NAME` layer, the
//! typed `--cwd` target binding, and the spawn-only ops line. A fifo rendezvous
//! proves the detached child ran and captures its argv + env.

use super::{World, write_exec};
use crate::cli_outbound::Cli;
use crate::opslog::{Origin, SYNTHETIC_EXIT};
use crate::start::{DETACHED_EXIT, Prepared, execute_prompt};
use std::path::Path;

/// The composer's fire-time params: workspace `name`, its path, and the §3.3
/// typed work-target binding (`None` = the bare rung's "bind nothing", the
/// shape most of these tests fire). There is no driver-cwd field since bl-6654
/// — the detached driver stands in the workspace it drives.
/// One fire, whole (§8.1): the located workspace, the prepared start and the
/// edited goal — said once here rather than as a struct literal per case.
pub(super) fn fire(
    ws: &Path,
    name: &str,
    binding: Option<&Path>,
    goal: &str,
) -> crate::start::Fire {
    crate::start::Fire {
        workspace: ws.to_path_buf(),
        prepared: prepared(name, binding),
        goal: goal.to_owned(),
    }
}

pub(super) fn prepared(name: &str, binding: Option<&Path>) -> Prepared {
    Prepared {
        workspace: name.to_owned(),
        binding: binding.map(Path::to_path_buf),
        goal: String::new(),
        origin: Origin::Conversation,
        lineage: None,
    }
}

/// The workspace the fire drives — created, because it is also the detached
/// driver's own cwd (bl-6654) and a spawn into a missing directory fails.
pub(super) fn workspace(w: &World) -> std::path::PathBuf {
    let ws = w.yog.path().join("ws");
    std::fs::create_dir_all(&ws).expect("workspace dir");
    ws
}

/// A blocking fifo — reading it rendezvous with the detached child's write.
pub(super) fn make_fifo(path: &Path) {
    let status = crate::git_env::status(
        crate::git_env::command(Path::new("mkfifo"))
            .args(["-m", "600"])
            .arg(path),
    )
    .expect("spawn mkfifo");
    assert!(status.success(), "mkfifo");
}

#[test]
fn prompt_fires_the_goal_verbatim_layers_yog_name_and_logs_the_sentinel() {
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
    let ws = workspace(&w);
    let conversation = execute_prompt(
        &lernie,
        w.state.path(),
        "TS",
        &fire(&ws, "cobalt-gecko", None, "do it"),
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
    assert_eq!(e.cwd, ws.display().to_string());
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

/// **The work target is typed, not prose** (§3.3, bl-6654 / VISION §4.10 item
/// 2): a bound rung's directory rides lernie's `--cwd` — the creation-time seed
/// for the working-directory mark every later tool step reads — and the goal
/// stays LAST so `clip_goal` still trims exactly the payload. The spawned argv
/// and the logged one are built from one list, so they cannot disagree about a
/// flag that rides conditionally.
#[test]
fn prompt_passes_the_typed_target_as_cwd_with_the_goal_still_last() {
    let w = World::new();
    let fifo = w.bin.path().join("report");
    make_fifo(&fifo);
    let body = format!(
        "#!/bin/sh\nprintf '%s\\037%s\\037%s\\037%s\\037%s\\037%s\\037%s' \"$1\" \"$2\" \"$3\" \"$4\" \"$5\" \"$6\" \"$7\" > '{}'\n",
        fifo.display()
    );
    let lernie = Cli::new(write_exec(w.bin.path(), "lernie", &body));
    let ws = workspace(&w);
    let target = w.balls.path().join("work");
    std::fs::create_dir_all(&target).unwrap();
    let conversation = execute_prompt(
        &lernie,
        w.state.path(),
        "TS",
        &fire(&ws, "cobalt-gecko", Some(&target), "do it"),
        &[],
        &super::rng(),
    )
    .unwrap();

    let recorded = std::fs::read_to_string(&fifo).unwrap();
    let fields: Vec<&str> = recorded.split('\u{1f}').collect();
    assert_eq!(fields[0], "prompt");
    assert_eq!(fields[1], "--name");
    assert_eq!(fields[2], conversation);
    assert_eq!(fields[3], "--cwd", "the binding is a parameter, not prose");
    assert_eq!(fields[4], target.to_string_lossy());
    assert_eq!(fields[5], ws.to_string_lossy());
    assert_eq!(fields[6], "do it", "the goal is still the last argument");

    let e = &w.ops()[0];
    assert_eq!(&e.argv[4..6], ["--cwd", target.to_string_lossy().as_ref()]);
    assert_eq!(e.argv[7], "do it");
    // The driver's own cwd is NOT the target: it is the workspace it drives, the
    // same for every rung. The binding is the whole work-target channel.
    assert_eq!(e.cwd, ws.display().to_string());
}

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
