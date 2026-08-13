//! STORIES **S1-T3** agent-verbs: message / stop (± children) / scan spawn the
//! §8.2 argv with cwd = the workspace, and every outcome lands in `ops.jsonl`
//! (STORIES S1.3, DESIGN §8.2, §4.2, §15 M6 Z7). One fake `lernie` recorder,
//! the three verbs driven sequentially through the current `actions::verbs`
//! dispatch API.

#![allow(clippy::unwrap_used)]

use crate::support::Recorder;
use tempfile::tempdir;
use yog::actions::verbs;
use yog::cli_outbound::Cli;
use yog::opslog;

/// STORIES **S1-T3** agent-verbs.
#[test]
fn s1_t3_message_stop_scan_argv_and_ops_trail() {
    let bin = tempdir().unwrap();
    let state = tempdir().unwrap();
    let ws = tempdir().unwrap();
    let ws_s = ws.path().to_string_lossy().to_string();

    // Fake `lernie`: records every spawn; `scan` plays a summary line (§8.2).
    let rec = Recorder::new(bin.path(), "lernie").on("scan", "flushed 2 inboxes\n", 0);
    let lernie = Cli::new(rec.path());
    // The §8.2 lernie family takes a workspace-bound `Cli` and nothing else
    // (bl-bf79): the workspace is named once, here, and is the verbs' cwd and
    // their `<ws>` argv word both.
    let bound = verbs::Bound::at(
        &lernie,
        &yog::world::compose(&yog::xdg::Env::from_env()),
        ws.path(),
    );
    let m = verbs::message(&bound, state.path(), "T1", "c-001", "ping").unwrap();
    let s = verbs::stop(&bound, state.path(), "T2", "c-001", true).unwrap();
    let sc = verbs::scan(&bound, state.path(), "T3").unwrap();

    // Outcomes: all exit 0; scan's summary rides back on stdout.
    assert_eq!((m.exit, s.exit, sc.exit), (0, 0, 0));
    assert_eq!(sc.stdout, "flushed 2 inboxes\n", "scan summary surfaced");

    // Recorded argv per §8.2 (arg 0 = the verb; cwd = the workspace).
    let inv = rec.invocations();
    assert_eq!(inv.len(), 3, "one spawn per verb");
    assert_eq!(inv[0].argv, ["message", ws_s.as_str(), "c-001", "ping"]);
    assert_eq!(
        inv[1].argv,
        ["stop", ws_s.as_str(), "c-001", "--stop-children"]
    );
    assert_eq!(inv[2].argv, ["scan", ws_s.as_str()]);
    let ws_canon = crate::support::canon(ws.path());
    assert!(
        inv.iter().all(|i| i.cwd == ws_canon),
        "each verb runs cwd = ws"
    );
    // The recorder captures the child's environment (argv+env+cwd), inherited
    // here since a bare `Cli::new` stands nothing of its own.
    assert!(inv[0].env.contains_key("PATH"), "env is recorded");

    // The ops.jsonl trail: one line per verb — argv (binary + args), cwd, exit
    // (§4.2). scan's summary is the durable stdout.
    let ops = opslog::tail(state.path(), 16);
    assert_eq!(ops.len(), 3, "one ops row per verb");
    assert_eq!(
        &ops[0].argv[1..],
        &["message", ws_s.as_str(), "c-001", "ping"]
    );
    assert_eq!(
        &ops[1].argv[1..],
        &["stop", ws_s.as_str(), "c-001", "--stop-children"]
    );
    assert_eq!(&ops[2].argv[1..], &["scan", ws_s.as_str()]);
    assert_eq!(ops[2].stdout, "flushed 2 inboxes\n");
    assert!(ops.iter().all(|e| e.exit == 0 && e.cwd == ws_s));
}
