//! End-to-end proof of the config-branch editor shim (DESIGN §9.3 Y21)
//! **without litany installed**: a real fake-`litany` recorder script that
//! invokes `$EDITOR` byte-for-byte the way litany's `edit_in_editor` does
//! (`litany/src/bin/litany/cli.rs:22-27`, the task-0 finding) —
//! `sh -c 'exec {EDITOR} "$1"' sh <checkout>` — and the real `yog` binary
//! (`CARGO_BIN_EXE_yog`) as the `--editor-apply` shim `$EDITOR` resolves to.
//!
//! The round trip: [`edit::stage_files`] writes a draft → [`edit::drive`]
//! spawns fake-`litany` with `EDITOR`/`YOG_EDIT_SRC` → fake-`litany`
//! materializes a checkout, refreshes `descriptions/**` (as litany does at
//! §3.3), and execs `$EDITOR` → the yog shim copies **only** the staged file
//! over the checkout → fake-`litany` asserts the staged file landed and its
//! `descriptions/**` survived, exiting 0. `drive`'s recorded exit 0 is the
//! proof the whole chain held.

// clippy's `allow-unwrap-in-tests` reaches `#[test]` fns and `#[cfg(test)]`
// mods, but not the plain fixture helpers of an integration-test crate (they
// are neither); those unwrap freely like any test. Scoped to this test binary
// and out of the src-only `rules-audit`.
#![allow(clippy::unwrap_used)]

use std::path::Path;
use tempfile::tempdir;
use yog::cli_outbound::Cli;
use yog::config_edit::branch::edit::{self, DraftFile, EditOrigin, EditPlan};

/// Write an executable fake `litany config` that reproduces litany's checkout
/// materialization + `$EDITOR` hand-off shape, then verifies the shim's
/// effect. Non-zero exit codes name the failure for the assertion message.
fn write_fake_litany(dir: &Path) -> std::path::PathBuf {
    let path = dir.join("litany");
    let body = r#"#!/bin/sh
# args: config <ws> <name> [flags]
ws="$2"
checkout="$ws/.config-author"
mkdir -p "$checkout/descriptions"
printf 'litany-refreshed' > "$checkout/descriptions/pool.md"
# litany's exact $EDITOR hand-off (cli.rs:22-27): word-split editor, one
# quoted positional = the checkout dir.
sh -c 'exec '"$EDITOR"' "$1"' sh "$checkout"
ed=$?
[ "$ed" -eq 0 ] || { echo "editor exited $ed" >&2; exit 20; }
[ -f "$checkout/workflow.yaml" ] || { echo "workflow.yaml not copied" >&2; exit 21; }
[ "$(cat "$checkout/descriptions/pool.md")" = "litany-refreshed" ] \
  || { echo "descriptions/** clobbered" >&2; exit 22; }
exit 0
"#;
    // Never a bare `fs::write`: this script is exec'd, and a write fd on it in
    // this process is the ETXTBSY race peer test threads lose
    // (`tests/support/write_exec.rs`).
    crate::support::write_exec::write_exec(&path, body);
    path
}

#[test]
fn config_edit_round_trip_through_the_real_shim() {
    let dir = tempdir().unwrap();
    let staging = edit::stage_files(
        &dir.path().join("stage"),
        "e2e-0",
        &[DraftFile {
            rel_path: "workflow.yaml".into(),
            bytes: b"steps: []\n".to_vec(),
        }],
    )
    .unwrap();

    let litany = write_fake_litany(dir.path());
    let ws = dir.path().join("ws");
    std::fs::create_dir_all(&ws).unwrap();

    let yog_bin = env!("CARGO_BIN_EXE_yog");
    let plan = EditPlan::compose(
        Path::new(yog_bin),
        &ws,
        "default",
        &EditOrigin::Advance,
        &staging,
    );
    let entry = edit::drive(
        &Cli::new(&litany),
        &ws,
        &plan,
        "T0",
        &dir.path().join("state"),
        yog::opslog::Origin::World,
    );

    // exit 0 ⇒ every internal assertion in fake-litany held: the shim copied
    // the staged file AND litany's descriptions/** refresh survived.
    assert_eq!(entry.exit, 0, "fake-litany stderr: {}", entry.stderr);
    let checkout = ws.join(".config-author");
    assert_eq!(
        std::fs::read_to_string(checkout.join("workflow.yaml")).unwrap(),
        "steps: []\n"
    );
    assert_eq!(
        std::fs::read_to_string(checkout.join("descriptions/pool.md")).unwrap(),
        "litany-refreshed"
    );
}
