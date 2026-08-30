//! STORIES **S5-T5** config-branch-shim: `litany config <ws> <name>` is spawned
//! with `EDITOR=<yog> --editor-apply` and `YOG_EDIT_SRC=<stage>`, the shim
//! copies **only** the drafted files, an empty diff surfaces verbatim, and the
//! staging directory has a bounded life (STORIES S5.3, DESIGN §9.3).
//!
//! `tests/integration/editor_roundtrip.rs` is this row's seed and proves the
//! copy half end-to-end through the **real** `yog` binary as `$EDITOR`. This
//! module takes the three beats that sit either side of it: the spawn contract,
//! the verbatim ride-back, and the staging lifecycle.
//!
//! **One premise drifted.** The row says "the staging dir is gone at exit". It
//! is not, and deliberately: `drive` returns the outcome and leaves the staged
//! bytes alone (a draft the operator may still be looking at), and
//! `sweep_staging` reclaims a directory untouched for 24 h at the next startup
//! (§5.2). Deleting at exit would race litany's own read of the checkout it was
//! just handed. The assertion below is that bounded life, not a same-breath
//! delete.

#![allow(clippy::unwrap_used)]

use crate::support::Recorder;
use std::path::Path;
use tempfile::tempdir;
use yog::cli_outbound::Cli;
use yog::config_edit::apply::{EDITOR_APPLY_FLAG, copy_staged};
use yog::config_edit::branch::edit::{self, DraftFile, EditOrigin, EditPlan};
use yog::opslog::{self, Origin};

/// What litany says when the operator's edit changed nothing.
const NO_CHANGE: &str = "no change to config/default — nothing committed\n";

/// STORIES **S5-T5** config-branch-shim.
#[test]
fn s5_t5_the_shim_contract_and_the_verbatim_ride_back() {
    let dir = tempdir().unwrap();
    let state = dir.path().join("state");
    let ws = dir.path().join("ws");
    std::fs::create_dir_all(&ws).unwrap();

    let staging = edit::stage_files(
        &dir.path().join("stage"),
        "nonce-0",
        &[DraftFile {
            rel_path: "workflow.yaml".into(),
            bytes: b"steps: []\n".to_vec(),
        }],
    )
    .unwrap();

    // --- The spawn contract (§9.3): yog re-enters ITSELF as the editor, and
    // names the staging dir in the one variable the shim reads.
    let yog_binary = Path::new("/opt/yog/bin/yog");
    let plan = EditPlan::compose(yog_binary, &ws, "default", &EditOrigin::Advance, &staging);
    let env = plan.env();
    assert_eq!(env[0].0, "EDITOR");
    assert!(
        env[0].1.contains(EDITOR_APPLY_FLAG),
        "EDITOR re-enters yog in shim mode: {}",
        env[0].1
    );
    assert!(env[0].1.contains("/opt/yog/bin/yog"));
    assert_eq!(
        env[1],
        ("YOG_EDIT_SRC", staging.display().to_string().as_str())
    );

    // --- The verb, and its words riding back. An empty diff is litany's own
    // sentence — yog owns no judgement about whether a config changed, so it
    // carries the verb's answer rather than deriving a second opinion.
    let litany = Recorder::new(dir.path(), "litany").on("config", NO_CHANGE, 0);
    let entry = edit::drive(
        &Cli::new(litany.path()),
        &ws,
        &plan,
        "T0",
        &state,
        Origin::World,
    );
    assert_eq!(entry.exit, 0);
    assert_eq!(entry.stdout, NO_CHANGE, "the empty diff surfaces verbatim");
    assert_eq!(
        &entry.argv[1..],
        &["config", &ws.display().to_string(), "default"],
        "config <ws> <name>, and no flags on the Advance lineage"
    );
    // The child observed both variables — asserted at the recorder, not at the
    // plan, so the env actually reached the process.
    let inv = litany.invocations();
    assert_eq!(inv.len(), 1);
    assert_eq!(
        inv[0].env.get("YOG_EDIT_SRC").map(String::as_str),
        Some(staging.display().to_string().as_str())
    );
    assert!(
        inv[0]
            .env
            .get("EDITOR")
            .is_some_and(|e| e.contains(EDITOR_APPLY_FLAG)),
        "the child's EDITOR is the shim: {:?}",
        inv[0].env.get("EDITOR")
    );
    // The outcome is durable (§4.2), whatever it was.
    let ops = opslog::tail(&state, 8);
    assert_eq!(ops.len(), 1);
    assert_eq!(ops[0].stdout, NO_CHANGE);

    // --- A fork names its source; the origin is argv, never a mode yog holds.
    let forked = EditPlan::compose(
        yog_binary,
        &ws,
        "spike",
        &EditOrigin::Fork {
            source: "config/default".to_owned(),
        },
        &staging,
    );
    let entry = edit::drive(
        &Cli::new(litany.path()),
        &ws,
        &forked,
        "T1",
        &state,
        Origin::World,
    );
    assert_eq!(
        &entry.argv[entry.argv.len() - 2..],
        &["--from", "config/default"]
    );
}

/// STORIES **S5-T5** config-branch-shim — the copy rule and the staging
/// lifetime. Split from the beat above only for the 100-line function cap.
#[test]
fn s5_t5_only_the_draft_is_copied_and_the_staging_dir_has_a_bounded_life() {
    let dir = tempdir().unwrap();
    let staging = edit::stage_files(
        &dir.path().join("stage"),
        "nonce-0",
        &[DraftFile {
            rel_path: "workflow.yaml".into(),
            bytes: b"steps: []\n".to_vec(),
        }],
    )
    .unwrap();

    // --- Only the drafted files are copied: whatever litany refreshed into the
    // checkout survives, because the shim writes and never deletes.
    let checkout = dir.path().join("checkout");
    std::fs::create_dir_all(checkout.join("descriptions")).unwrap();
    std::fs::write(checkout.join("descriptions/pool.md"), "litany-refreshed").unwrap();
    let written = copy_staged(&staging, &checkout).unwrap();
    assert_eq!(
        written,
        [std::path::PathBuf::from("workflow.yaml")],
        "only the draft"
    );
    assert_eq!(
        std::fs::read_to_string(checkout.join("descriptions/pool.md")).unwrap(),
        "litany-refreshed",
        "the checkout's own files are untouched"
    );

    // --- The staging life. It survives the drive (the operator's draft is
    // still theirs) …
    assert!(
        staging.exists(),
        "the drive does not delete the staging dir"
    );
    // … and is reclaimed by the startup sweep once it has gone cold. A fresh
    // one is kept; only a stale one is swept, and the decision is a pure
    // function of the clock the caller injects.
    let stage_root = dir.path().join("stage");
    assert!(
        edit::sweep_staging(&stage_root, 0).is_empty(),
        "a fresh staging dir is not swept"
    );
    assert!(staging.exists());
    let far_future = 2_000_000_000;
    let swept = edit::sweep_staging(&stage_root, far_future);
    assert_eq!(
        swept,
        std::slice::from_ref(&staging),
        "a cold one is reclaimed"
    );
    assert!(!staging.exists(), "and really deleted");
}
