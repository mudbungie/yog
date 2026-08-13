//! On-disk payload writers for the story fixtures — the files an agent's
//! surfaces are derived from, split out of [`super::world`] per the 300-line
//! cap: `messages/`, `steps/`, `inbox/`, the balls clone directory, and the two
//! low-level primitives (a backdated mtime, a blob in the object store) the
//! workspace builder needs.

#![allow(dead_code)]
#![allow(clippy::unwrap_used)]

use std::fs;
use std::io::Write as _;
use std::path::Path;

/// Backdate `path`'s modification time to `unix` seconds.
pub fn set_mtime(path: &Path, unix: i64) {
    let when = std::time::UNIX_EPOCH + std::time::Duration::from_secs(unix.unsigned_abs());
    let times = fs::FileTimes::new().set_modified(when);
    fs::File::options()
        .write(true)
        .open(path)
        .unwrap()
        .set_times(times)
        .unwrap();
}

/// Write `body` into the object store and return its oid — the hold mark's ref
/// target is a blob, so it cannot be pointed at a branch like the others.
pub fn hash_object(repo: &Path, body: &str) -> String {
    // Through `git_env::git()` like every other fixture fork: an inherited
    // `GIT_DIR` would write this blob into the outer repo's object store.
    let mut child = yog::git_env::git()
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_SYSTEM", "/dev/null")
        .arg("-C")
        .arg(repo)
        .args(["hash-object", "-w", "--stdin"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(body.as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success(), "hash-object: {}", out.status);
    String::from_utf8(out.stdout).unwrap().trim().to_owned()
}

/// Materialize the balls clone directory for `project` under `clones` — the
/// percent-encoded basename `projects::enumerate` decodes back (§5.1 #1). Only
/// `/` needs encoding for a temp-dir path, and the round trip is asserted here
/// rather than trusted, so a codec change fails loudly instead of silently
/// yielding a project yog never sees.
pub fn clone_dir(clones: &Path, project: &Path) -> std::path::PathBuf {
    let encoded = project.to_string_lossy().replace('/', "%2F");
    assert_eq!(
        yog::xdg::percent_decode(&encoded),
        project.to_string_lossy(),
        "the fixture's clone-dir encoding must round-trip"
    );
    let dir = clones.join(encoded);
    fs::create_dir_all(&dir).unwrap();
    dir
}

/// Write a transcript entry at `<ws>/agents/<agent>/messages/<name>` — inside
/// the agent's own worktree, not beside it — and return its path so a test can
/// read the bytes back for the Raw assertion. `name` is `NNN-<origin>.<ext>`:
/// the origin `tool` is reserved for tool results, `.md` is a delivered
/// deposit, any other `.json` is a model turn.
pub fn write_message(ws: &Path, agent: &str, name: &str, body: &str) -> std::path::PathBuf {
    let dir = ws.join("agents").join(agent).join("messages");
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join(name);
    fs::write(&path, body).unwrap();
    path
}

/// Write `<ws>/steps/<agent>/<step>/<name>` (ARCH §3.3) — `meta.json`,
/// `request.json`, `response.json`, `staging/`, `tools/<n>/input.json`.
pub fn write_step(
    ws: &Path,
    agent: &str,
    step: &str,
    name: &str,
    body: &str,
) -> std::path::PathBuf {
    let path = ws.join("steps").join(agent).join(step).join(name);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, body).unwrap();
    path
}

/// Write an undelivered deposit at `<ws>/inbox/<agent>/<name>.md` (§2.11) — the
/// `✉n` count's one source.
pub fn write_deposit(ws: &Path, agent: &str, name: &str, body: &str) -> std::path::PathBuf {
    let dir = ws.join("inbox").join(agent);
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join(format!("{name}.md"));
    fs::write(&path, body).unwrap();
    path
}
