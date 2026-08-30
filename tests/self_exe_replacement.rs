//! bl-f558 — **a live engine outlives its own inode.** Installing, updating or
//! rebuilding yog replaces the file at yog's pathname while the running process
//! keeps executing the unlinked image; from that instant Linux's
//! `/proc/self/exe` — the whole of `current_exe()` there — reads back `<path>
//! (deleted)`, an annotation naming a file that does not exist. The §8.6
//! capability control is the sharp case, because the start flow re-resolves its
//! shim on **every** Start (`world::tools::ensure_control`), so the first Start
//! after an install used to burn that annotation into the adjudicator litany
//! consults before every granted tool invocation — and every later tool call
//! failed closed at a boundary whose cause was hours in the past.
//!
//! No in-process test can reach this: replacing THIS binary's inode would move
//! `current_exe()` for every other test in the process. So the shape is the one
//! `sigterm_durability` already uses — a child arm, inert without its fixture
//! env, spawned from a **second pathname for this same executable** that the
//! child then replaces under itself. Nothing is copied: both pathnames are hard
//! links (the born-from one to this test binary, the replacement to the built
//! `yog`), which is also what makes the consult at the end a real one — after
//! the swap that pathname IS the yog binary, so the shim is exec'd for a real
//! verdict over real stdio, exactly as litany's seam consults it.

// The fixture helpers of an integration-test crate unwrap freely like any test.
#![allow(clippy::unwrap_used)]

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Stdio;

/// Names the fixture directory, and its presence is what makes the child arm
/// live — absent (every ordinary run of this binary) the arm returns at once.
const FIXTURE_ENV: &str = "YOG_TEST_REPLACED_EXE_DIR";
/// The child arm's test name, spawned `--exact`. Deliberately shares no
/// substring with any argv the shim is later exec'd under.
const CHILD_ARM: &str = "the_live_engine_that_lost_its_inode";

/// The built `yog` — the executable the "install" moves onto the pathname the
/// child was born from, so the shim it writes can actually be consulted.
fn installed_yog() -> &'static Path {
    Path::new(env!("CARGO_BIN_EXE_yog"))
}

/// The child arm: a live process that converges the world's tools at boot, has
/// its own pathname atomically replaced under it, and then does what a Start
/// does — re-resolve and rewrite the §8.6 control shim.
#[test]
fn the_live_engine_that_lost_its_inode() {
    let Ok(dir) = std::env::var(FIXTURE_ENV) else {
        return;
    };
    let dir = PathBuf::from(dir);
    let born = std::env::current_exe().unwrap();

    // Boot, as `main.rs` boots: the world's tool roster is converged before
    // any face exists, which is the ask that fixes this process's reading of
    // its own executable.
    yog::world::tools::ensure_tools(&dir.join("boot")).unwrap();

    // The install, in its real shape: a *different* inode renamed onto the
    // same pathname. `rename(2)` is atomic and refuses a same-inode move,
    // which is why the replacement is the built yog rather than another link
    // to this binary.
    let next = dir.join("yog.next");
    std::fs::hard_link(installed_yog(), &next).unwrap();
    std::fs::rename(&next, &born).unwrap();

    // On a platform that annotates the unlinked dentry — Linux, via procfs —
    // the live reading is now an impossible path. macOS reports the original
    // pathname with no annotation, so it never had the defect and the
    // assertion below would be false there.
    let raw = std::env::current_exe().unwrap();
    if cfg!(target_os = "linux") {
        assert_ne!(raw, born, "the reading moved when the inode was replaced");
        assert!(!raw.exists(), "and it names nothing on disk: {raw:?}");
    }

    // What a Start does on the live process, every time: re-resolve the
    // adjudicator and converge its shim.
    let shim = yog::world::tools::ensure_control(&dir.join("tools")).unwrap();
    let body = std::fs::read_to_string(&shim).unwrap();
    assert!(
        body.contains(&format!("exec '{}' ", born.display())),
        "the shim names the pathname this engine was born from: {body}"
    );
    assert!(!body.contains("(deleted)"), "{body}");
}

/// Consult the shim exactly as litany's seam does: no argv, the request on
/// stdin, the conversation's env, the workspace as cwd.
fn consult(shim: &Path, root: &Path, workspace: &Path, request: &str) -> (i32, String) {
    let mut child = yog::git_env::command(shim)
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
        out.status.code().unwrap_or_default(),
        String::from_utf8(out.stdout).unwrap(),
    )
}

#[test]
fn a_start_after_the_binary_is_replaced_seeds_a_control_that_still_runs() {
    // Same filesystem as the built yog, because every pathname here is a hard
    // link and a link cannot cross a mount.
    let dir = tempfile::Builder::new()
        .prefix("yog-inode-")
        .tempdir_in(installed_yog().parent().unwrap())
        .unwrap();
    let live = dir.path().join("yog");
    std::fs::hard_link(std::env::current_exe().unwrap(), &live).unwrap();

    let status = yog::git_env::command(&live)
        .args([CHILD_ARM, "--exact", "--nocapture", "--test-threads=1"])
        .env(FIXTURE_ENV, dir.path())
        .status()
        .unwrap();
    assert!(status.success(), "the live-engine arm failed: {status:?}");

    // The durable artifact the child left behind, read back from outside it.
    let shim = yog::world::tools::control_path(&dir.path().join("tools"));
    let body = std::fs::read_to_string(&shim).unwrap();
    assert!(!body.contains("(deleted)"), "{body}");
    assert!(
        body.contains(&format!("exec '{}' ", live.display())),
        "{body}"
    );

    // And it runs. This is the assertion the defect actually broke: the shim
    // is exec'd for a real verdict, where an annotated pathname would have
    // failed closed with a program that does not exist.
    let root = tempfile::tempdir().unwrap();
    let workspace = root.path().join("data/yog/workspaces/cobalt-gecko");
    std::fs::create_dir_all(workspace.join("agents/amber")).unwrap();
    let request = r#"{"id":"toolu_01","name":"bash","input":{"command":"cargo test"},"role":"worker","agent_id":"amber"}"#;
    let (code, out) = consult(&shim, root.path(), &workspace, request);
    assert_eq!(code, 0, "{out}");
    assert_eq!(out.trim(), r#"{"verdict":"pass"}"#);
}
