//! S15-T1 — the capability control end to end (DESIGN §8.6, VISION §4.11): the
//! shim yog seeds into `world/tools/` **is** the executable litany's
//! tool-control seam consults, and it answers litany's wire contract over a real
//! process's stdin and stdout.
//!
//! Everything below the process edge is unit-tested beside its module; what this
//! binary proves is the part no in-process test can — that the generated shim
//! script execs a yog which reads one JSON request on stdin, writes one JSON
//! verdict on stdout, and exits 0 for every answer including a refusal (the seam
//! fails closed on a non-zero exit, so a decline must not look like a fault).

// The fixture helpers of an integration-test crate unwrap freely like any test.
#![allow(clippy::unwrap_used)]

use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::{Command, Stdio};

use yog::world::tools::TOOL_CONTROL;

/// Write the shim's own shape — a `/bin/sh` re-exec of yog under the
/// tool-control word — against the **built** `yog` binary rather than this test
/// process, which is the one substitution the harness has to make. The exact
/// bytes yog seeds are pinned beside `ensure_control`; what is under test here
/// is the chain the script starts.
fn seed_shim(tools: &Path) -> std::path::PathBuf {
    std::fs::create_dir_all(tools).unwrap();
    let path = tools.join(TOOL_CONTROL);
    let body = format!(
        "#!/bin/sh\nexec '{}' {TOOL_CONTROL} \"$@\"\n",
        env!("CARGO_BIN_EXE_yog"),
    );
    std::fs::write(&path, body).unwrap();
    let mut perms = std::fs::metadata(&path).unwrap().permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(&path, perms).unwrap();
    path
}

/// Consult the shim exactly as litany's seam does: no argv, the request on
/// stdin, `LITANY_CONV_REPO`/`LITANY_CONV_BRANCH` in the environment, and the
/// workspace root as cwd. Returns `(exit code, stdout)`.
fn consult(shim: &Path, root: &Path, workspace: &Path, request: &str) -> (i32, String) {
    let mut child = Command::new(shim)
        .current_dir(workspace)
        .env_clear()
        .env("HOME", root.join("home"))
        .env("XDG_DATA_HOME", root.join("data"))
        .env("XDG_STATE_HOME", root.join("state"))
        .env("LITANY_CONV_REPO", workspace)
        .env("LITANY_CONV_BRANCH", "amber")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(request.as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();
    (
        out.status.code().unwrap(),
        String::from_utf8(out.stdout).unwrap(),
    )
}

/// One `tool_use` block as the seam serializes it.
fn request(command: &str) -> String {
    format!(
        r#"{{"id":"toolu_01","name":"bash","input":{{"command":"{command}"}},"role":"worker","agent_id":"amber"}}"#
    )
}

/// Raise the monitor's per-conversation floor by writing the ops row the
/// boundary's own `/revoke` writes — the fold's only durable home.
fn floor(state_root: &Path, agent_id: &str) {
    yog::opslog::append(
        state_root,
        &yog::opslog::OpEntry {
            argv: ["yog-control", "floor", agent_id, "raise"]
                .map(str::to_owned)
                .to_vec(),
            ..yog::opslog::OpEntry::default()
        },
    )
    .unwrap();
}

#[test]
fn the_seeded_shim_answers_the_seam_over_real_stdio() {
    let root = tempfile::tempdir().unwrap();
    let workspace = root.path().join("data/yog/workspaces/cobalt-gecko");
    std::fs::create_dir_all(workspace.join("agents/amber")).unwrap();
    let shim = seed_shim(&root.path().join("data/yog/world/tools"));

    // Work inside the writable root passes, and a pass carries no reason —
    // litany's own parser rejects one.
    let (code, out) = consult(&shim, root.path(), &workspace, &request("cargo test"));
    assert_eq!(code, 0);
    assert_eq!(out.trim(), r#"{"verdict":"pass"}"#);

    // Leaving the world passes too — the shipped table parks nothing.
    let (code, out) = consult(&shim, root.path(), &workspace, &request("curl https://x"));
    assert_eq!(code, 0);
    assert_eq!(out.trim(), r#"{"verdict":"pass"}"#);

    // A park is imposed, not shipped: the monitor's floor over this
    // conversation, read back off the same trail the boundary writes it to.
    floor(
        &yog::world::layout_under(&root.path().join("data/yog"))
            .state
            .join("yog"),
        "amber",
    );
    let (code, out) = consult(&shim, root.path(), &workspace, &request("curl https://x"));
    assert_eq!(code, 0);
    assert!(out.contains(r#""verdict":"hold""#), "{out}");
    assert!(out.contains("open-world"), "{out}");

    // Loss is declined in band — still exit 0, because a decline is an answer
    // and a non-zero exit is what the seam reads as a fault.
    let (code, out) = consult(&shim, root.path(), &workspace, &request("rm -rf /etc"));
    assert_eq!(code, 0);
    assert!(out.contains(r#""verdict":"refuse""#), "{out}");

    // A request nobody could adjudicate exits non-zero, which fails closed at
    // the seam: the invocation never executes.
    let (code, out) = consult(&shim, root.path(), &workspace, "not json");
    assert_ne!(code, 0);
    assert!(out.is_empty(), "{out}");
}
