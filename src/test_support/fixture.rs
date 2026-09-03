//! **Executable fixtures are written by a CHILD, never by this process**
//! (bl-fd28) — the whole of the ETXTBSY discipline, which [`crate::git_env`]
//! used to hold at the fork and no longer does.
//!
//! `fs::write` on a fixture script holds a write fd for the length of the
//! write. A `fork` on ANY thread copies that fd into a child that keeps it
//! until its own `exec` completes, and an `exec` of the script inside that
//! window is **ETXTBSY**. `git_env` used to close that from the fork side with
//! a `cfg(test)` lock, but a lock there can only cover the forks yog itself
//! performs: the substrate crates yog links (`balls`, `litany`, `brazen`) fork
//! `git` on their own account and took none of it, so a lib beat driving one of
//! them in-process was an unlocked forker for as long as it ran (bl-6bf5
//! measured 8 failures, bl-fd28 another 2 in `fan`).
//!
//! So the exposure is closed on the side that owns it. Here the fd never exists
//! in this process at all: `sh -c 'cat > "$1" && chmod 755 "$1"'` holds it in a
//! child, and a peer fork copies only the fds this process has. That reverses
//! `git_env`'s older "the victim's own care cannot save it" — which was true of
//! a write-side *bracket* (a lock a peer's fork must also respect) and is false
//! of a write-side *relocation* (nothing left to copy, whoever forks).
//!
//! The body goes down a pipe rather than an argv word so nothing here has an
//! `ARG_MAX`; a fixture large enough to fill the 64 KiB pipe buffer before the
//! child drains it would deadlock, and none is remotely near that.
//!
//! With the descriptor gone the lock was measured out — 0/0/0 with it and
//! 0/0/0 without, over 3,360 runs of bl-6bf5's filter per side — so nothing
//! here is guarded and nothing needs to be. The spawn still goes through
//! [`crate::git_env::spawn`], which is the crate's one fork for its own
//! reasons (`rules/no-bare-fork.yml`).

use std::io::Write as _;
use std::path::Path;
use std::process::Stdio;

/// Write `body` to `path` and mark it `0755`, entirely inside a child process.
/// The one lawful way for a test in this crate to create an executable file —
/// `rules/no-hand-chmod.yml` refuses the hand-rolled spelling.
pub(crate) fn write_exec(path: &Path, body: &str) {
    let mut cmd = crate::git_env::command(Path::new("sh"));
    cmd.arg("-c")
        .arg(r#"cat > "$1" && chmod 755 "$1""#)
        .arg("sh")
        .arg(path)
        .stdin(Stdio::piped());
    let mut child = crate::git_env::spawn(&mut cmd).expect("fork the fixture writer");
    child
        .stdin
        .take()
        .expect("the writer's stdin")
        .write_all(body.as_bytes())
        .expect("feed the fixture body");
    let status = child.wait().expect("wait for the fixture writer");
    assert!(status.success(), "fixture writer failed: {status} {path:?}");
}

/// Take the owner's write bit off an existing file — the "this cannot be
/// recorded" fixture, and no part of the executable-bit hazard above. It lives
/// here because the mode-bit vocabulary lives here (see the rule), and it takes
/// no mode: an exec bit is not reachable through it.
pub(crate) fn read_only(path: &Path) {
    chmod(path, 0o444);
}

/// Undo [`read_only`] — restore an ordinary `0644`, so a `TempDir` can clean up
/// after a test that made one of its files unwritable.
pub(crate) fn writable(path: &Path) {
    chmod(path, 0o644);
}

fn chmod(path: &Path, mode: u32) {
    use std::os::unix::fs::PermissionsExt as _;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode)).expect("chmod");
}
