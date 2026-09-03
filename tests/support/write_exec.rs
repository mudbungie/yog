//! **Executable fixtures are written by a CHILD, never by this process** — the
//! `tests/` half of the discipline `src/test_support/fixture.rs` holds for the
//! lib binary, which the integration crate cannot see (`test_support` is
//! `#[cfg(test)] pub(crate)`).
//!
//! `exec` on a file some process still holds open for writing fails with
//! `ETXTBSY`, and a plain `fs::write` in a test thread hands exactly that fd to
//! any peer thread that happens to `fork` while it is open — the copy lives in
//! the child until its own `exec` clears it. `tests/integration/main.rs` runs
//! ~25 tests thread-parallel in one process and each of them forks, so the
//! window was hit at roughly **one run in eight**, as an ETXTBSY on a recorder
//! script another test had just written.
//!
//! **A lock was never available to fix it here**, which is the fact that
//! settles the design. The forks that matter are yog's own (`git_tree::cmd`,
//! `cli_outbound`), and yog is linked as a LIBRARY by an integration test — not
//! `cfg(test)` — so its spawn lock was compiled out of the very binary that
//! needed it. So the fd is removed from this process rather than scheduled
//! around: `sh` opens the file, `cat` fills it, `chmod` marks it, and all of it
//! dies with the child we wait on. A fork of *this* process copies *this* fd
//! table, which never held the descriptor. That answer generalized in bl-fd28 —
//! the lib binary took it too, and its spawn lock, having nothing left to
//! exclude, was measured out (`src/git_env.rs`'s module doc).
//!
//! `#[path]`-included by each binary that needs it, the way every other shared
//! `tests/` module is. `make rules-audit` scans `src` only, so
//! `rules/no-hand-chmod.yml` cannot reach here and the sweep is by hand.

use std::io::Write as _;
use std::path::Path;

/// Write `body` to `path` and mark it `0755`, entirely inside a child process —
/// the ONE way a test binary in this crate creates a file it is going to exec.
pub fn write_exec(path: &Path, body: &str) {
    let mut child = std::process::Command::new("sh")
        .args(["-c", r#"cat > "$1" && chmod 755 "$1""#, "sh"])
        .arg(path)
        .stdin(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    // Taken and dropped in one statement: `cat` sees EOF only once this end of
    // the pipe is closed, and the wait below would otherwise never return.
    child
        .stdin
        .take()
        .unwrap()
        .write_all(body.as_bytes())
        .unwrap();
    let status = child.wait().unwrap();
    assert!(status.success(), "authoring {}: {status}", path.display());
}
