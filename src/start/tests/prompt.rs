//! **What the detached `litany prompt` child is handed** (§8.1, §3.3): the
//! fire-time conversation mint riding `--name`, the goal fired verbatim
//! (bl-6920), the `YOG_NAME` layer and the typed `--cwd` target binding. A fifo
//! rendezvous proves the detached child ran and captures its argv + env.
//!
//! What the *fire itself* writes down — the §4.2 ops line, its stderr sink, the
//! clipped goal and the two spawn failures that are not handoffs — is
//! [`super::prompt_ops`]. One gesture, two questions: a launch that lands and a
//! launch that is only ever a row. The fixtures both read live here.

use super::{World, write_exec};
use crate::cli_outbound::Cli;
use crate::opslog::Origin;
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
    let litany = Cli::new(write_exec(w.bin.path(), "litany", &body));
    let ws = workspace(&w);
    let conversation = execute_prompt(
        &litany,
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
    // The minted name rides `--name` — the litany-stored fact home (§3.3 as
    // ruled by bl-50f3) — with the goal last so `clip_goal` still trims it.
    assert_eq!(fields[1], "--name");
    assert_eq!(fields[3], ws.to_string_lossy());
    // The goal reaches litany exactly as the operator edited it (bl-6920):
    // no identity line, no mutation — `--name` is identity's only channel,
    // and litany states the stored fact in its assembled context (litany
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
/// 2): a bound rung's directory rides litany's `--cwd` — the creation-time seed
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
    let litany = Cli::new(write_exec(w.bin.path(), "litany", &body));
    let ws = workspace(&w);
    let target = w.balls.path().join("work");
    std::fs::create_dir_all(&target).unwrap();
    let conversation = execute_prompt(
        &litany,
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
